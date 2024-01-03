'use client';
import Image from 'next/image';
import styles from './page.module.css';
import Wallet from './Wallet';

export default function Home() {
  return (
    <main className={styles.main}>
      <div className={styles.description}>
        <Wallet />
      </div>
    </main>
  );
}
