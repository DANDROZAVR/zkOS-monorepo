export const shielderContractAddress =
  process.env.SHIELDER_CONTRACT_ADDRESS ??
  (() => {
    throw new Error("SHIELDER_CONTRACT_ADDRESS env not defined");
  })();
export const rpcHttpEndpoint =
  process.env.RPC_HTTP_ENDPOINT ??
  (() => {
    throw new Error("RPC_HTTP_ENDPOINT env not defined");
  })();
export const relayerFeeAddress =
  process.env.RELAYER_FEE_ADDRESS ??
  (() => {
    throw new Error("RELAYER_FEE_ADDRESS env not defined");
  })();
export const relayerUrl =
  process.env.RELAYER_URL ??
  (() => {
    throw new Error("RELAYER_URL env not defined");
  })();
export const chainId =
  process.env.CHAIN_ID ??
  (() => {
    throw new Error("CHAIN_ID env not defined");
  })();
export const testnetPrivateKey =
  process.env.TESTNET_PRIVATE_KEY ??
  (() => {
    throw new Error("TESTNET_PRIVATE_KEY env not defined");
  })();

export const getChainConfig = () => {
  return {
    chainId: parseInt(chainId),
    rpcHttpEndpoint: rpcHttpEndpoint,
    contractAddress: shielderContractAddress as `0x${string}`,
    testnetPrivateKey: testnetPrivateKey as `0x${string}`,
  };
};

export const getRelayerConfig = () => {
  return {
    address: relayerFeeAddress as `0x${string}`,
    url: relayerUrl,
  };
};