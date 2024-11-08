import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { 
    PublicKey, 
    Keypair, 
    Connection, 
    LAMPORTS_PER_SOL,
    SystemProgram,
    Transaction,
    sendAndConfirmTransaction
} from "@solana/web3.js";
import { 
    TOKEN_PROGRAM_ID, 
    MINT_SIZE,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    createInitializeMintInstruction,
    createAssociatedTokenAccountInstruction,
    createMintToInstruction,
    getAssociatedTokenAddress,
    getAccount,
    createSyncNativeInstruction,
    NATIVE_MINT
} from "@solana/spl-token";
import { assert } from "chai";
import * as idl from "../target/idl/raydium_liquidity.json";

describe("raydium-liquidity", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    // Your wallet
    const OWNER_PUBKEY = new PublicKey("b7doByN6f3VyrV26aVA3TeNUuMFJ7w22sbUDaK3QNx4");
    
    // Connection
    const connection = new Connection("https://api.devnet.solana.com", "confirmed");
    
    // Program IDs
    const programId = new PublicKey("82p44HjdpEBjebM2KBvP7seVB1RZFVEczBYaeSDUaa7v");
    const program = new Program(idl as anchor.Idl, programId, provider);
    const RAYDIUM_PROGRAM_ID = new PublicKey("devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH");

    // Pool Constants
    const POOL_AUTHORITY = new PublicKey("GH7E6vGvkEuzT7XEVza83xkpHfEgg1ioShGjUNTkCefa");
    const LP_MINT = new PublicKey("Cckf7GjdYLoYpyaHp9vRs7NRqYnQ4WQqjc47y1PAmNXg");
    const TOKEN_VAULT_0 = new PublicKey("DyB2bGiRGCsjK6NEvu7WEvRivJtt6Tg79275FwLMCzny");
    const TOKEN_VAULT_1 = new PublicKey("78c2facB98M7BEZpi8QgBvrSxwdKrMPcxh1XwDKnQ1H5");
    const USDC_MINT = new PublicKey("usdcdMjaaMPZyihyukvJNGTFKbJXPLWpMZQRKnuh51A");

    // Pool Authority Keypair from Secret/// pool wsol/usdc
    const POOL_AUTHORITY_KEYPAIR = Keypair.fromSecretKey(new Uint8Array([
        55,122,83,23,231,66,117,84,68,31,255,222,61,146,143,231,
        168,121,125,48,219,123,185,45,135,113,14,12,222,161,97,134,
        226,255,162,240,174,124,197,232,115,38,158,58,63,69,93,6,
        36,199,251,197,130,112,55,36,130,207,155,35,231,154,105,89
    ]));

    const TICK_ARRAYS = {
        lower: new PublicKey("5WjPy8M5xGpHYpXKMLfbeLo2FMqHxvVqd5EwHHJekEvF"),
        upper: new PublicKey("5WjPy8M5xGpHYpXKMLfbeLo2FMqHxvVqd5EwHHJekEvF")
    };

    let usdcAccount: PublicKey;
    let wsolAccount: PublicKey;
    let positionNftMint: Keypair;
    let positionNftAccount: PublicKey;

    before(async () => {
        try {
            console.log("Setting up test environment...");
            console.log("Owner:", OWNER_PUBKEY.toString());
            console.log("Program ID:", programId.toString());
            
            // First check USDC account
            usdcAccount = await getAssociatedTokenAddress(
                USDC_MINT,
                OWNER_PUBKEY
            );
            
            console.log("\nChecking USDC account...");
            try {
                const usdcAccountInfo = await getAccount(connection, usdcAccount);
                console.log("USDC Account:", usdcAccount.toString());
                console.log("USDC Balance:", Number(usdcAccountInfo.amount));
            } catch (e) {
                throw new Error("USDC account not found or has no balance");
            }

            // Setup WSOL Account
            console.log("\nSetting up WSOL account...");
            wsolAccount = await getAssociatedTokenAddress(
                NATIVE_MINT,
                OWNER_PUBKEY
            );
            
            // Create and fund WSOL account
            const wsolAmount = 0.2 * LAMPORTS_PER_SOL;
            const wsolTx = new Transaction();

            try {
                await getAccount(connection, wsolAccount);
                console.log("WSOL account exists");
            } catch (e) {
                console.log("Creating new WSOL account");
                wsolTx.add(
                    createAssociatedTokenAccountInstruction(
                        provider.wallet.publicKey,
                        wsolAccount,
                        OWNER_PUBKEY,
                        NATIVE_MINT
                    )
                );
            }

            wsolTx.add(
                SystemProgram.transfer({
                    fromPubkey: provider.wallet.publicKey,
                    toPubkey: wsolAccount,
                    lamports: wsolAmount,
                }),
                createSyncNativeInstruction(wsolAccount)
            );

            const wsolSig = await provider.sendAndConfirm(wsolTx);
            console.log("WSOL setup transaction:", wsolSig);

            const wsolAccountInfo = await getAccount(connection, wsolAccount);
            console.log("WSOL Account:", wsolAccount.toString());
            console.log("WSOL Balance:", Number(wsolAccountInfo.amount) / LAMPORTS_PER_SOL);

            // Create Position NFT
            console.log("\nCreating Position NFT...");
            positionNftMint = Keypair.generate();
            const mintRent = await connection.getMinimumBalanceForRentExemption(MINT_SIZE);

            const nftTx = new Transaction();
            nftTx.add(
                SystemProgram.createAccount({
                    fromPubkey: provider.wallet.publicKey,
                    newAccountPubkey: positionNftMint.publicKey,
                    space: MINT_SIZE,
                    lamports: mintRent,
                    programId: TOKEN_PROGRAM_ID,
                }),
                createInitializeMintInstruction(
                    positionNftMint.publicKey,
                    0,
                    OWNER_PUBKEY,
                    OWNER_PUBKEY
                )
            );

            positionNftAccount = await getAssociatedTokenAddress(
                positionNftMint.publicKey,
                OWNER_PUBKEY
            );

            nftTx.add(
                createAssociatedTokenAccountInstruction(
                    provider.wallet.publicKey,
                    positionNftAccount,
                    OWNER_PUBKEY,
                    positionNftMint.publicKey
                ),
                createMintToInstruction(
                    positionNftMint.publicKey,
                    positionNftAccount,
                    OWNER_PUBKEY,
                    1
                )
            );

            await provider.sendAndConfirm(nftTx, [positionNftMint]);
            console.log("Position NFT created:", positionNftMint.publicKey.toString());
            console.log("Position NFT Account:", positionNftAccount.toString());

            // Verify pool and vault accounts
            console.log("\nVerifying pool accounts...");
            const poolInfo = await connection.getAccountInfo(POOL_AUTHORITY);
            console.log("Pool exists:", !!poolInfo);

            const vault0Info = await connection.getAccountInfo(TOKEN_VAULT_0);
            const vault1Info = await connection.getAccountInfo(TOKEN_VAULT_1);
            console.log("Vault 0 exists:", !!vault0Info);
            console.log("Vault 1 exists:", !!vault1Info);

            console.log("\nSetup complete!");

        } catch (error) {
            console.error("Setup error:", error);
            throw error;
        }
    });

    // In the liquidity addition test:

it("Adds liquidity to pool", async () => {
  try {
      console.log("\nStarting liquidity addition...");
      console.log("Fee payer:", OWNER_PUBKEY.toString());
      
      const initialWsolAccount = await getAccount(connection, wsolAccount);
      const initialUsdcAccount = await getAccount(connection, usdcAccount);
      
      console.log("Initial WSOL balance:", Number(initialWsolAccount.amount) / LAMPORTS_PER_SOL);
      console.log("Initial USDC balance:", Number(initialUsdcAccount.amount));

      // Using smaller amounts
      const amount0 = new anchor.BN(0.05 * LAMPORTS_PER_SOL);
      const amount1 = new anchor.BN(50_000);
      
      const tickLower = -443632 - 64;
      const tickUpper = -443632 + 64;

      console.log("\nParameters:");
      console.log("Amount SOL:", amount0.toString());
      console.log("Amount USDC:", amount1.toString());
      console.log("Tick range:", tickLower, "to", tickUpper);

      // Get instruction data
      const addLiquidityAccounts = {
          clmmProgram: RAYDIUM_PROGRAM_ID,
          poolState: POOL_AUTHORITY,
          positionNftMint: positionNftMint.publicKey,
          positionNftAccount: positionNftAccount,
          tokenAccount0: wsolAccount,
          tokenAccount1: usdcAccount,
          tokenVault0: TOKEN_VAULT_0,
          tokenVault1: TOKEN_VAULT_1,
          tickArrayLower: TICK_ARRAYS.lower,
          tickArrayUpper: TICK_ARRAYS.upper,
          owner: OWNER_PUBKEY,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      };

      // Get the instruction
      const ix = await program.methods
          .addLiquidity(
              amount0,
              amount1,
              tickLower,
              tickUpper
          )
          .accounts(addLiquidityAccounts)
          .instruction();

      // Create transaction
      const latestBlockhash = await connection.getLatestBlockhash('confirmed');
      
      const tx = new anchor.web3.Transaction();
      tx.recentBlockhash = latestBlockhash.blockhash;
      tx.feePayer = OWNER_PUBKEY;  // Set your public key as fee payer
      tx.add(ix);

      console.log("\nSigning with accounts:");
      console.log("1. Fee Payer (owner):", OWNER_PUBKEY.toString());
      console.log("2. Pool Authority:", POOL_AUTHORITY.toString());

      // Let anchor provider sign (which has your wallet)
      try {
          await provider.wallet.signTransaction(tx);
          console.log("✓ Signed with fee payer");
      } catch (e) {
          console.error("Error signing with fee payer:", e);
          throw e;
      }

      // Sign with pool authority
      try {
          tx.partialSign(POOL_AUTHORITY_KEYPAIR);
          console.log("✓ Signed with pool authority");
      } catch (e) {
          console.error("Error signing with pool authority:", e);
          throw e;
      }

      // Verify signatures before sending
      console.log("\nTransaction signatures:");
      tx.signatures.forEach((sig, i) => {
          console.log(`${i + 1}. ${sig.publicKey.toString()}${sig.signature ? ' (signed)' : ''}`);
      });

      // Send transaction
      console.log("\nSending transaction...");
      const signature = await connection.sendRawTransaction(tx.serialize(), {
          skipPreflight: false,
          maxRetries: 3,
          preflightCommitment: 'confirmed',
      });

      console.log("Transaction sent:", signature);
      
      // Wait for confirmation
      const confirmationResponse = await connection.confirmTransaction({
          signature,
          blockhash: latestBlockhash.blockhash,
          lastValidBlockHeight: latestBlockhash.lastValidBlockHeight
      });

      if (confirmationResponse.value.err) {
          throw new Error(`Transaction failed: ${JSON.stringify(confirmationResponse.value.err)}`);
      }

      console.log("Transaction confirmed!");

      // Verify final balances
      const finalWsolAccount = await getAccount(connection, wsolAccount);
      const finalUsdcAccount = await getAccount(connection, usdcAccount);

      console.log("\nFinal balances:");
      console.log("WSOL:", Number(finalWsolAccount.amount) / LAMPORTS_PER_SOL);
      console.log("USDC:", Number(finalUsdcAccount.amount));

  } catch (error) {
      console.error("\nTest error:", error);
      
      throw error;
  }
});
});