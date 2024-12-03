use shielder_relayer::{RELAYER_PORT_ENV, RELAYER_SIGNING_KEYS_ENV};
use shielder_rust_sdk::alloy_primitives::address;

use super::*;

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    CLIConfig::command().debug_assert()
}

#[test]
fn config_resolution() {
    // ---- Target configuration. --------------------------------------------------------------
    let logging_format = LoggingFormat::Json;
    let host = DEFAULT_HOST.to_string();
    let port = 1234;
    let metrics_port = 5678;
    let balance_monitor_interval_secs = 60;
    let node_rpc_url = "http://localhost:8545".to_string();
    let shielder_contract_address = address!("0000000000000000000000000000000000000000");
    let fee_destination_key = "key0".to_string();
    let key1 = "key1".to_string();
    let key2 = "key2".to_string();
    let nonce_policy = DEFAULT_NONCE_POLICY;
    let dry_running = DryRunning::Always;
    let relay_count_for_recharge = DEFAULT_RELAY_COUNT_FOR_RECHARGE;
    let relay_fee = DEFAULT_RELAY_FEE.to_string();

    let expected_config = ServerConfig {
        logging_format, // from CLI
        network: NetworkConfig {
            host,         // default
            port,         // from env
            metrics_port, // from CLI
        },
        chain: ChainConfig {
            node_rpc_url: node_rpc_url.clone(),               // from CLI
            shielder_contract_address,                        // from CLI
            fee_destination_key: fee_destination_key.clone(), // from env
            signing_keys: vec![key1.clone(), key2.clone()],   // from env
            relay_fee: U256::from_str(&relay_fee).unwrap(),   // from CLI
        },
        operations: OperationalConfig {
            balance_monitor_interval_secs, // from env
            nonce_policy,                  // default
            dry_running,                   // from CLI
            relay_count_for_recharge,      // default
        },
    };

    // ---- CLI configuration. -----------------------------------------------------------------
    let cli_config = CLIConfig {
        logging_format: Some(logging_format),
        host: None,
        port: None,
        metrics_port: Some(metrics_port),
        balance_monitor_interval_secs: None,
        node_rpc_url: Some(node_rpc_url),
        shielder_contract_address: Some(shielder_contract_address.to_string()),
        fee_destination_key: None,
        signing_keys: None,
        nonce_policy: None,
        dry_running: Some(dry_running),
        relay_count_for_recharge: None,
        relay_fee: Some(relay_fee),
    };

    // ---- Environment variables. -----------------------------------------------------------
    unsafe {
        std::env::set_var(RELAYER_PORT_ENV, port.to_string());
        std::env::set_var(
            BALANCE_MONITOR_INTERVAL_SECS_ENV,
            balance_monitor_interval_secs.to_string(),
        );
        std::env::set_var(FEE_DESTINATION_KEY_ENV, fee_destination_key);
        std::env::set_var(RELAYER_SIGNING_KEYS_ENV, format!("{key1},{key2}"));
    }

    // ---- Test. ------------------------------------------------------------------------------
    let resolved_config = resolve_config_from_cli_config(cli_config);
    assert_eq!(resolved_config, expected_config);
}