import { Connection, Keypair } from '@solana/web3.js';
import * as fs from 'fs';

const connection = new Connection('https://devnet.helius-rpc.com/?api-key=2cf1ece5-ba9a-42d0-8bdd-674430c89ea8', 'confirmed');

async function deploy() {
  const keypairData = JSON.parse(fs.readFileSync('/Users/amalnathsathyan/.config/solana/id.json', 'utf-8'));
  const keypair = Keypair.fromSecretKey(Uint8Array.from(keypairData));
  
  const balance = await connection.getBalance(keypair.publicKey);
  console.log('Balance:', balance / 1e9, 'SOL');
}

deploy();
