use std::{env, io, str::FromStr, sync::Arc};

use alloy_provider::Provider;
use alloy_signer_local::PrivateKeySigner;
use anyhow::{anyhow, Result};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use shielder_rust_sdk::{
    alloy_primitives::{Address, U256},
    contract::{
        providers::create_provider_with_nonce_caching_signer, ConnectionPolicy, ShielderUser,
    },
};
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::{
    config::{resolve_config, ChainConfig, LoggingFormat, NoncePolicy, ServerConfig},
    metrics::{prometheus_endpoint, setup_metrics_handle},
    monitor::{balance_monitor::balance_monitor, Balances},
    recharge::start_recharging_worker,
    relay::Taskmaster,
};

mod config;
mod metrics;
mod monitor;
mod recharge;
mod relay;

#[derive(Clone)]
pub struct AppState {
    pub node_rpc_url: String,
    pub fee_destination: Address,
    pub relay_fee: U256,
    pub signer_addresses: Vec<Address>,
    pub taskmaster: Taskmaster,
    pub balances: Balances,
}

struct Signers {
    signers: Vec<PrivateKeySigner>,
    addresses: Vec<Address>,
    balances: Balances,
}

#[tokio::main]
async fn main() -> Result<()> {
    let server_config = resolve_config();
    init_logging(server_config.logging_format)?;

    let signers = get_signer_info(&server_config.chain)?;

    tokio::try_join!(
        balance_monitor(
            &server_config.chain.node_rpc_url,
            server_config.operations.balance_monitor_interval_secs,
            signers.balances.clone(),
        ),
        start_metrics_server(&server_config, signers.balances.clone()),
        start_main_server(&server_config, signers),
    )?;

    Ok(())
}

fn get_signer_info(config: &ChainConfig) -> Result<Signers> {
    let (signers, addresses) = parse_keys(&config.signing_keys)?;
    let balances = Arc::new(
        addresses
            .iter()
            .map(|address| (*address, Default::default()))
            .collect(),
    );
    Ok(Signers {
        signers,
        addresses,
        balances,
    })
}

async fn start_metrics_server(config: &ServerConfig, balances: Balances) -> Result<()> {
    let address = config.network.metrics_address();
    let listener = tokio::net::TcpListener::bind(address.clone()).await?;
    info!("Exposing metrics on {address}");

    let metrics_handle = setup_metrics_handle()?;
    let node_rpc_url = config.chain.node_rpc_url.clone();

    let app = Router::new()
        .route(
            "/metrics",
            get(move || prometheus_endpoint(metrics_handle, node_rpc_url, balances)),
        )
        .layer(CorsLayer::permissive());
    Ok(axum::serve(listener, app).await?)
}

async fn start_main_server(config: &ServerConfig, signers: Signers) -> Result<()> {
    let address = config.network.main_address();
    let listener = tokio::net::TcpListener::bind(address.clone()).await?;
    info!("Listening on {address}");

    let fee_destination = signer(&config.chain.fee_destination_key)?;
    let fee_destination_address = fee_destination.address();

    let report_for_recharge = start_recharging_worker(
        config.chain.node_rpc_url.clone(),
        fee_destination,
        signers.addresses.len(),
        config.operations.relay_count_for_recharge,
        config.chain.relay_fee,
    );

    let state = AppState {
        node_rpc_url: config.chain.node_rpc_url.clone(),
        fee_destination: fee_destination_address,
        relay_fee: config.chain.relay_fee,
        balances: signers.balances,
        signer_addresses: signers.addresses,
        taskmaster: Taskmaster::new(
            build_shielder_users(
                signers.signers,
                &config.chain,
                config.operations.nonce_policy,
            )
            .await?,
            config.operations.dry_running,
            report_for_recharge,
        ),
    };

    let app = Router::new()
        .route("/health", get(monitor::endpoints::health_endpoint))
        .route("/relay", post(relay::relay))
        .with_state(state.clone())
        .route_layer(middleware::from_fn(metrics::request_metrics))
        .layer(CorsLayer::permissive());
    Ok(axum::serve(listener, app).await?)
}

fn init_logging(format: LoggingFormat) -> Result<()> {
    const LOG_CONFIGURATION_ENVVAR: &str = "RUST_LOG";

    let filter = EnvFilter::new(
        env::var(LOG_CONFIGURATION_ENVVAR)
            .as_deref()
            .unwrap_or("info"),
    );

    let subscriber = tracing_subscriber::fmt()
        .with_writer(io::stdout)
        .with_target(true)
        .with_env_filter(filter);

    match format {
        LoggingFormat::Json => subscriber.json().try_init(),
        LoggingFormat::Text => subscriber.try_init(),
    }
    .map_err(|err| anyhow!(err))
}

fn parse_keys(keys: &[String]) -> Result<(Vec<PrivateKeySigner>, Vec<Address>)> {
    let signers = keys
        .iter()
        .map(|key| signer(key))
        .collect::<Result<Vec<_>>>()?;
    let addresses = signers.iter().map(|signer| signer.address()).collect();
    Ok((signers, addresses))
}

fn signer(signing_key: &str) -> Result<PrivateKeySigner> {
    PrivateKeySigner::from_str(signing_key)
        .map_err(|err| anyhow!("Failed to create signer - invalid signing key: {err:?}"))
}

async fn build_shielder_users(
    signers: Vec<PrivateKeySigner>,
    config: &ChainConfig,
    nonce_policy: NoncePolicy,
) -> Result<Vec<ShielderUser<impl Provider + Clone>>> {
    let mut shielder_users = vec![];
    for signer in signers {
        let policy = match nonce_policy {
            NoncePolicy::Caching => ConnectionPolicy::Keep {
                caller_address: signer.address(),
                provider: create_provider_with_nonce_caching_signer(&config.node_rpc_url, signer)
                    .await?,
            },
            NoncePolicy::Stateless => ConnectionPolicy::OnDemand {
                signer,
                rpc_url: config.node_rpc_url.clone(),
            },
        };
        shielder_users.push(ShielderUser::new(config.shielder_contract_address, policy));
    }
    Ok(shielder_users)
}