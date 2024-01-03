// 'use client';
import { useState } from 'react';
import { connectToWallet } from './CcdClient';

export default function Wallet() {
  const [account, setAccount] = useState('');

  const handleClick = async () => {
    const { account, provider } = await connectToWallet();
    console.log({ account });
    console.log({ provider });
    setAccount(account);
  };

  return (
    <div>
      <div>{`Address ${account}`}</div>
      <button onClick={() => handleClick()}>Connect ccd wallet</button>
    </div>
  );
}
