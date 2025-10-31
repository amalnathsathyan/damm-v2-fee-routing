import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { DammV2FeeRouting } from "../target/types/damm_v2_fee_routing";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { assert } from "chai";

describe("damm-v2-fee-routing", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.DammV2FeeRouting as Program<DammV2FeeRouting>;

  const cpAmmProgramId = new PublicKey("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");
  const poolAuthority = new PublicKey("HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC");

  const testPoolAddress = new PublicKey("EAKkJT2ik91rGftcbj9WAYRCpdQjqJuLsMTgb6onqnFV");
  const baseMint = new PublicKey("5sqqeX61xURcpyxFbY5HDfgKKMHQGfD3ZwmxzJ8zwfmF");
  const quoteMint = new PublicKey("So11111111111111111111111111111111111111112");

  const Y0_TOTAL_ALLOCATION = new BN(1_000_000_000_000);
  const INVESTOR_FEE_SHARE_BPS = 5000;

  before(async () => {
    console.log("\n🚀 Testing with devnet pool:");
    console.log(`   Pool: ${testPoolAddress.toBase58()}`);
    console.log(`   Base: ${baseMint.toBase58()}`);
    console.log(`   Quote (wSOL): ${quoteMint.toBase58()}\n`);

    try {
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        provider.wallet.payer,
        quoteMint,
        provider.wallet.publicKey,
        false,
        undefined,
        undefined,
        TOKEN_2022_PROGRAM_ID
      );
      console.log("✓ Creator wSOL ATA ready\n");
    } catch (err: any) {
      console.log("Note:", err.message);
    }
  });

  it("Validates pool supports quote token fee collection", async () => {
    console.log("\n=== TEST 1: Pool Validation ===\n");

    const poolInfo = await provider.connection.getAccountInfo(testPoolAddress);
    assert.isNotNull(poolInfo, "Pool must exist");

    const poolData = poolInfo!.data;

    // Read at CORRECT offsets (discovered via debug script)
    const tokenABytes = poolData.slice(168, 200);
    const tokenBBytes = poolData.slice(200, 232);
    const tokenA = new PublicKey(tokenABytes);
    const tokenB = new PublicKey(tokenBBytes);
    const collectFeeMode = poolData[72];

    console.log("✅ Pool account exists");
    console.log(`   token_a_mint (offset 168): ${tokenA.toBase58()}`);
    console.log(`   token_b_mint (offset 200): ${tokenB.toBase58()}`);
    console.log(`   collect_fee_mode: ${collectFeeMode} (${collectFeeMode === 0 ? "quote-only" : "both tokens"})`);

    assert.equal(tokenA.toBase58(), baseMint.toBase58(), "token_a must be base");
    assert.equal(tokenB.toBase58(), quoteMint.toBase58(), "token_b must be quote");
    assert.isTrue(collectFeeMode === 0 || collectFeeMode === 1, "Must support quote fee collection");

    console.log("✅ Pool validation passed\n");
  });

  it("Initializes honorary position", async () => {
    console.log("\n=== TEST 2: Initialize Honorary Position ===\n");

    const payer = provider.wallet.publicKey;
    const vaultKeypair = Keypair.generate();
    const vault = vaultKeypair.publicKey;

    const [vaultConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), vault.toBuffer()],
      program.programId
    );

    const [positionAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), vault.toBuffer(), Buffer.from("position_authority")],
      program.programId
    );

    const positionNftMintKeypair = Keypair.generate();
    const positionNftMint = positionNftMintKeypair.publicKey;

    const positionNftAccount = getAssociatedTokenAddressSync(
      positionNftMint,
      positionAuthority,
      true,
      TOKEN_2022_PROGRAM_ID
    );

    const [position] = PublicKey.findProgramAddressSync(
      [Buffer.from("position"), positionNftMint.toBuffer()],
      cpAmmProgramId
    );

    const creatorQuoteAta = getAssociatedTokenAddressSync(
      quoteMint,
      payer,
      false,
      TOKEN_2022_PROGRAM_ID
    );

    console.log("📋 Account Addresses:");
    console.log(`   Vault: ${vault.toBase58()}`);
    console.log(`   Vault Config: ${vaultConfig.toBase58()}`);
    console.log(`   Position Authority: ${positionAuthority.toBase58()}`);
    console.log(`   Position: ${position.toBase58()}\n`);

    console.log("📤 Sending transaction...\n");

    const tx = await program.methods
      .initializeHonoraryPosition({
        y0TotalAllocation: Y0_TOTAL_ALLOCATION,
        investorFeeShareBps: INVESTOR_FEE_SHARE_BPS,
      })
      .accounts({
        payer,
        vault,
        vaultConfig,
        pool: testPoolAddress,
        baseMint,
        quoteMint,
        positionAuthority,
        positionNftMint,
        positionNftAccount,
        position,
        creatorQuoteAta,
        cpAmmProgram: cpAmmProgramId,
        poolAuthority,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
        systemProgram: SystemProgram.programId,
      })
      .signers([positionNftMintKeypair])
      .rpc();

    console.log("✅ Transaction successful!");
    console.log(`   Signature: ${tx}`);
    console.log(`   Explorer: https://explorer.solana.com/tx/${tx}?cluster=devnet\n`);

    // Verify vault config
    const vaultConfigAccount = await program.account.vaultConfig.fetch(vaultConfig);

    assert.equal(vaultConfigAccount.vault.toBase58(), vault.toBase58());
    assert.equal(vaultConfigAccount.pool.toBase58(), testPoolAddress.toBase58());
    assert.equal(vaultConfigAccount.baseMint.toBase58(), baseMint.toBase58());
    assert.equal(vaultConfigAccount.quoteMint.toBase58(), quoteMint.toBase58());
    assert.equal(vaultConfigAccount.y0TotalAllocation.toString(), Y0_TOTAL_ALLOCATION.toString());
    assert.equal(vaultConfigAccount.investorFeeShareBps, INVESTOR_FEE_SHARE_BPS);

    console.log("✅ All validations passed!");
    console.log("\n📊 Vault Config Details:");
    console.log(`   Pool: ${vaultConfigAccount.pool.toBase58()}`);
    console.log(`   Honorary Position: ${vaultConfigAccount.honoraryPosition.toBase58()}`);
    console.log(`   Base Mint: ${vaultConfigAccount.baseMint.toBase58()}`);
    console.log(`   Quote Mint: ${vaultConfigAccount.quoteMint.toBase58()}`);
    console.log(`   Y0 Allocation: ${vaultConfigAccount.y0TotalAllocation.toString()}`);
    console.log(`   Fee Share: ${vaultConfigAccount.investorFeeShareBps} bps (${vaultConfigAccount.investorFeeShareBps / 100}%)`);
  });

  it("Fails with invalid fee share", async () => {
    console.log("\n=== TEST 3: Invalid Fee Share ===\n");

    const payer = provider.wallet.publicKey;
    const vaultKeypair = Keypair.generate();
    const vault = vaultKeypair.publicKey;

    const [vaultConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), vault.toBuffer()],
      program.programId
    );

    const [positionAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), vault.toBuffer(), Buffer.from("position_authority")],
      program.programId
    );

    const positionNftMintKeypair = Keypair.generate();
    const positionNftMint = positionNftMintKeypair.publicKey;

    const positionNftAccount = getAssociatedTokenAddressSync(
      positionNftMint,
      positionAuthority,
      true,
      TOKEN_2022_PROGRAM_ID
    );

    const [position] = PublicKey.findProgramAddressSync(
      [Buffer.from("position"), positionNftMint.toBuffer()],
      cpAmmProgramId
    );

    const creatorQuoteAta = getAssociatedTokenAddressSync(
      quoteMint,
      payer,
      false,
      TOKEN_2022_PROGRAM_ID
    );

    console.log("📤 Attempting with fee share of 10001 bps (>100%)...\n");

    try {
      await program.methods
        .initializeHonoraryPosition({
          y0TotalAllocation: Y0_TOTAL_ALLOCATION,
          investorFeeShareBps: 10001,
        })
        .accounts({
          payer,
          vault,
          vaultConfig,
          pool: testPoolAddress,
          baseMint,
          quoteMint,
          positionAuthority,
          positionNftMint,
          positionNftAccount,
          position,
          creatorQuoteAta,
          cpAmmProgram: cpAmmProgramId,
          poolAuthority,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
          systemProgram: SystemProgram.programId,
        })
        .signers([positionNftMintKeypair])
        .rpc();

      assert.fail("Should have thrown error");
    } catch (error: any) {
      console.log("✅ Correctly rejected invalid fee share");
      console.log(`   Error: InvalidInvestorFeeShare\n`);
      assert.isTrue(
        error.message.includes("InvalidInvestorFeeShare") || error.code === 6006
      );
    }
  });
});
