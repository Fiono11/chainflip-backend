import fs from 'fs/promises';

async function loadContract(abiPath: string): Promise<JSON> {
  const abi = await fs.readFile(abiPath, 'utf-8');
  return JSON.parse(abi);
}

function loadContractCached(abiPath: string) {
  let cached: JSON | undefined;
  return async () => {
    if (!cached) {
      cached = await loadContract(abiPath);
    }
    return cached;
  };
}
const CF_ETH_CONTRACT_ABI_TAG = 'v1.0.0';
export const getErc20abi = loadContractCached(
  '../contract-interfaces/eth-contract-abis/IERC20.json',
);
export const getGatewayAbi = loadContractCached(
  `../contract-interfaces/eth-contract-abis/${CF_ETH_CONTRACT_ABI_TAG}/IStateChainGateway.json`,
);
export const getCFTesterAbi = loadContractCached(
  `../contract-interfaces/eth-contract-abis/${CF_ETH_CONTRACT_ABI_TAG}/CFTester.json`,
);
