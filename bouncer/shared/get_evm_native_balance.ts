import { Chain, assetDecimals } from '@chainflip-io/cli';
import Web3 from 'web3';
import { fineAmountToAmount, getEvmEndpoint } from './utils';

export async function getEvmNativeBalance(chain: Chain, address: string): Promise<string> {
  const web3 = new Web3(getEvmEndpoint(chain));

  const weiBalance: string = await web3.eth.getBalance(address);
  return fineAmountToAmount(weiBalance, assetDecimals.ETH);
}
