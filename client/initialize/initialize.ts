import idl from "../../target/idl/pda_vesting.json";
import { PdaVesting } from "../../target/types/pda_vesting";
import { Program, Idl, AnchorProvider, setProvider, web3, Wallet, BN } from "@coral-xyz/anchor";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import fs from 'fs';

// Helper function to load wallet keypair from file system
export function loadWalletKey(keypairFile: string): web3.Keypair {
    const loaded = web3.Keypair.fromSecretKey(
        new Uint8Array(JSON.parse(fs.readFileSync(keypairFile).toString())),
    );
    return loaded;
}

// Initialize Solana connection to devnet
const connection = new web3.Connection("https://api.devnet.solana.com");

// Load initializer's keypair from wallet file
const initializerKeypair = loadWalletKey("owner_signer_wallet.json");

// Create wallet instance from keypair
const initializerWallet = new Wallet(initializerKeypair);

// Create Anchor provider with connection and wallet
const provider = new AnchorProvider(connection, initializerWallet, {});
setProvider(provider);

// Parse IDL and create program instance
const idlString = JSON.parse(JSON.stringify(idl));  
const program = new Program<PdaVesting>(idlString, provider);

async function main() {
    // Get program data account address
    const [programDataAddress] = web3.PublicKey.findProgramAddressSync(
        [program.programId.toBuffer()],
        new web3.PublicKey('BPFLoaderUpgradeab1e11111111111111111111111')
    );

    console.log("Program Data Account:", programDataAddress.toString());
    
    const programDataAccountInfo = await connection.getAccountInfo(programDataAddress);
    if (programDataAccountInfo) {
            const upgradeAuthorityAddress = new web3.PublicKey(programDataAccountInfo.data.slice(13, 45));
            console.log("Upgrade Authority:", upgradeAuthorityAddress.toString());
    }

    console.log("Loaded wallet public key:", initializerKeypair.publicKey.toString());

    // Derive PDA (Program Derived Address) for BTB sale account using seed "btb-sale-account"
    const [btbSaleAccount] = await web3.PublicKey.findProgramAddress(
        [Buffer.from("btb-sale-account"), initializerKeypair.publicKey.toBuffer()],
        program.programId
    );
    
    // Log btbSaleAccount address
    console.log("BTB Sale Account (PDA):", btbSaleAccount.toString());


    // BTB token mint address on devnet
    const btbMint = new web3.PublicKey("btbVv5dmAjutpRRSr6DKwBPyPyfKiJw4eXU11BPuTCK");

    // Create associated token account for PDA to hold BTB tokens
    const btbSaleTokenAccount = await getAssociatedTokenAddress(btbMint, btbSaleAccount, true);
    
    // Log btbSaleTokenAccount address
    console.log("BTB Sale Token Account:", btbSaleTokenAccount.toString());

    // Define addresses for supported payment tokens on devnet
    const btb = new web3.PublicKey("btbVv5dmAjutpRRSr6DKwBPyPyfKiJw4eXU11BPuTCK");      // BTB
    const usdt = new web3.PublicKey("utK7s5CmT6vvkd3JpTg5CfMaqAS8uVMwnqZjPZvcLkD");      // USDT
    const usdc = new web3.PublicKey("ucKymLwwPxrPaUDMtYGx5uoho91SfE3Qs2VuXf9dDZB");      // USDC
    const paypal_usd = new web3.PublicKey("pa3x7zKXd2yvPNM8NxJUp1tu1j8xeXyRb6Y65yqPvtQ"); // PayPal USD

    // Wallet that will receive tokens
    const owner_token_receive_wallet = new web3.PublicKey("te6eqhHuXFuhP1bjBfPs17VS84dR1M725FR9txASuCS");

    // Set BTB token and vesting prices
    const btb_price = new BN(1_000);      // BTB token price
    const vesting_price = new BN(8.08);   // Vesting price

    try {
        // Initialize the BTB sale program with configuration
        const tx = await program.methods.initialize(
            btb,                        // btb
            usdt,                       // USDT mint
            usdc,                       // USDC mint
            paypal_usd,                // PayPal USD mint
            owner_token_receive_wallet, // Token receiver
            btb_price,                 // BTB price
            vesting_price              // Vesting price
        )
        .accounts({
            btbSaleAccount: btbSaleAccount,                   // PDA account
            btbSaleTokenAccount: btbSaleTokenAccount,         // Token account for PDA
            btbMintAccount: btbMint,                         // BTB mint
            programData: programDataAddress,                  // Program data account
            signer: initializerWallet.publicKey,             // Owner
            systemProgram: web3.SystemProgram.programId,     // System program ID
            tokenProgram: TOKEN_PROGRAM_ID,                  // Token program ID
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID, // Associated token program ID
        })
        .signers([initializerKeypair])  // Add initializer as transaction signer
        .rpc();
       
        console.log("PDA initialized. Transaction signature:", tx);

        const accountInfo = await program.account.initializeDataAccount.fetch(btbSaleAccount);
        console.log("\n=== BTB Sale Account Data ===");
        console.log("\nToken Addresses:");
        console.log("BTB Address:", accountInfo.btb.toString());
        console.log("USDT Address:", accountInfo.usdt.toString());
        console.log("USDC Address:", accountInfo.usdc.toString());
        console.log("PayPal USD Address:", accountInfo.paypalUsd.toString());
        
        console.log("\nWallet Addresses:");
        console.log("Owner Token Receive Wallet:", accountInfo.ownerTokenReceiveWallet.toString());
        console.log("Owner Initialize Wallet:", accountInfo.ownerInitializeWallet.toString());
        
        console.log("\nPrices:");
        console.log("BTB Price (raw):", accountInfo.btbPrice.toString());
        console.log("BTB Price (formatted):", accountInfo.btbPrice.toNumber() / 1_000_000, "USDT");
        console.log("Vesting Price (raw):", accountInfo.vestingPrice.toString());
        console.log("Vesting Price (formatted):", accountInfo.vestingPrice.toNumber() / 1_000_000, "USDT");

        console.log("\nSales:");
        console.log("Sales Status:", accountInfo.isSaleActive);
        
    } catch (error) {
        console.error("Error during initialization:", error);
    }
}

// Execute the main function with error handling
main().catch((error) => {
    console.error("Program execution failed:", error);
    process.exit(1);
});