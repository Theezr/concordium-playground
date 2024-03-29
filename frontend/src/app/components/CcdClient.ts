import { detectConcordiumProvider, WalletApi } from '@concordium/browser-wallet-api-helpers';

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
