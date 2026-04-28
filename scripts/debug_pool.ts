import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Connection } from "@solana/web3.js";

async function debugPool() {
    
const connection = new Connection('https://devnet.helius-rpc.com/?api-key=2cf1ece5-ba9a-42d0-8bdd-674430c89ea8',{commitment:'confirmed'});

  const poolAddress = new PublicKey("EAKkJT2ik91rGftcbj9WAYRCpdQjqJuLsMTgb6onqnFV");
  const expectedBaseMint = new PublicKey("5sqqeX61xURcpyxFbY5HDfgKKMHQGfD3ZwmxzJ8zwfmF");
  const expectedQuoteMint = new PublicKey("So11111111111111111111111111111111111111112");
  
  console.log("\n🔍 DEBUGGING POOL DATA");
  console.log("=".repeat(80));
  console.log(`\nPool: ${poolAddress.toBase58()}`);
  console.log(`Expected Base:  ${expectedBaseMint.toBase58()}`);
  console.log(`Expected Quote: ${expectedQuoteMint.toBase58()}\n`);
  
  const poolAccountInfo = await connection.getAccountInfo(poolAddress);
  
  if (!poolAccountInfo) {
    console.log("❌ Pool account not found!");
    return;
  }
  
  console.log(`✅ Pool account exists (${poolAccountInfo.data.length} bytes)\n`);
  
  const poolData = poolAccountInfo.data;
  
  // Show discriminator
  const discriminator = poolData.slice(0, 8);
  console.log("Discriminator (0-8):", Array.from(discriminator));
  console.log("Expected:          ", [241, 154, 109, 4, 17, 177, 109, 188]);
  console.log("");
  
  // Try reading at standard offsets
  console.log("ATTEMPTING TO READ AT STANDARD OFFSETS:");
  console.log("-".repeat(80));
  
  try {
    const tokenABytes = poolData.slice(8, 40);
    const tokenA = new PublicKey(tokenABytes);
    console.log(`token_a_mint (8-40):  ${tokenA.toBase58()}`);
    console.log(`  Matches base?  ${tokenA.equals(expectedBaseMint) ? "✅ YES" : "❌ NO"}`);
    console.log(`  Matches quote? ${tokenA.equals(expectedQuoteMint) ? "✅ YES" : "❌ NO"}`);
  } catch (e) {
    console.log("❌ Error reading token_a_mint:", e.message);
  }
  
  console.log("");
  
  try {
    const tokenBBytes = poolData.slice(40, 72);
    const tokenB = new PublicKey(tokenBBytes);
    console.log(`token_b_mint (40-72): ${tokenB.toBase58()}`);
    console.log(`  Matches base?  ${tokenB.equals(expectedBaseMint) ? "✅ YES" : "❌ NO"}`);
    console.log(`  Matches quote? ${tokenB.equals(expectedQuoteMint) ? "✅ YES" : "❌ NO"}`);
  } catch (e) {
    console.log("❌ Error reading token_b_mint:", e.message);
  }
  
  console.log("");
  console.log(`collect_fee_mode (72): ${poolData[72]}`);
  console.log("");
  
  // Try scanning the entire pool data for our mints
  console.log("SCANNING ENTIRE POOL FOR TOKEN MINTS:");
  console.log("-".repeat(80));
  
  const baseMintBytes = expectedBaseMint.toBytes();
  const quoteMintBytes = expectedQuoteMint.toBytes();
  
  for (let i = 0; i <= poolData.length - 32; i++) {
    const chunk = poolData.slice(i, i + 32);
    
    if (Buffer.compare(chunk, Buffer.from(baseMintBytes)) === 0) {
      console.log(`\n✅ Found BASE mint at offset ${i}:`);
      console.log(`   ${expectedBaseMint.toBase58()}`);
    }
    
    if (Buffer.compare(chunk, Buffer.from(quoteMintBytes)) === 0) {
      console.log(`\n✅ Found QUOTE mint at offset ${i}:`);
      console.log(`   ${expectedQuoteMint.toBase58()}`);
    }
  }
  
  console.log("\n" + "=".repeat(80));
}

debugPool().then(() => process.exit(0)).catch(console.error);
