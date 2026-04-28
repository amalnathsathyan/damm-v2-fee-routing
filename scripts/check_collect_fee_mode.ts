import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Connection } from "@solana/web3.js";

async function checkCollectFeeMode() {
  const connection = new Connection('https://api.devnet.solana.com');
  const poolAddress = new PublicKey("EAKkJT2ik91rGftcbj9WAYRCpdQjqJuLsMTgb6onqnFV");

  const accountInfo = await connection.getAccountInfo(poolAddress);
  if (!accountInfo) {
    console.error('Error: Pool account not found');
    return;
  }

  const data = accountInfo.data;

  console.log('Pool data length:', data.length);

  const collectFeeModeOffsetRust = 232;
  const collectFeeModeOffsetTest = 72;

  console.log('collect_fee_mode at byte 72:', data[collectFeeModeOffsetTest]);
  console.log('collect_fee_mode at byte 232:', data[collectFeeModeOffsetRust]);

  console.log('Token A mint (bytes 8-40):', new PublicKey(data.slice(8, 40)).toBase58());
  console.log('Token B mint (bytes 40-72):', new PublicKey(data.slice(40, 72)).toBase58());
  console.log('Token A mint (bytes 168-200):', new PublicKey(data.slice(168, 200)).toBase58());
  console.log('Token B mint (bytes 200-232):', new PublicKey(data.slice(200, 232)).toBase58());
}

checkCollectFeeMode().then(() => process.exit(0)).catch(console.error);
