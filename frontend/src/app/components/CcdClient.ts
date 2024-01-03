import { Buffer } from 'buffer/';

import { detectConcordiumProvider, WalletApi } from '@concordium/browser-wallet-api-helpers';
import { ModuleReference } from '@concordium/web-sdk';

export type ContractName = 'project_token' | 'carbon_credits' | 'carbon_credit_market';
export interface ContractInfo {
  schemaBuffer: Buffer;
  contractName: ContractName;
  moduleRef?: ModuleReference.Type;
}

export async function connectToWallet(): Promise<{ provider: WalletApi; account: string }> {
  const provider = await detectConcordiumProvider();
  let account = await provider.getMostRecentlySelectedAccount();
  if (!account) {
    account = await provider.connect();
  }

  if (!account) {
    throw new Error('Could not connect to the Wallet Account');
  }

  return { provider, account };
}
