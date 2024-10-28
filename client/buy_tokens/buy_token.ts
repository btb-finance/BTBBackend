import idl from "../../target/idl/pda_vesting.json";
import { PdaVesting } from "../../target/types/pda_vesting";
import { Program, Idl, AnchorProvider, setProvider, web3, Wallet, BN } from "@coral-xyz/anchor";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token";
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

// Load user's keypair from wallet file
const userKeypair = loadWalletKey("user_wallet.json");

// Create wallet instance from keypair
const userWallet = new Wallet(userKeypair);

// Create Anchor provider with connection and wallet
const provider = new AnchorProvider(connection, userWallet, {});
setProvider(provider);

// Parse IDL and create program instance
const idlString = JSON.parse(JSON.stringify(idl));  
const program = new Program<PdaVesting>(idlString, provider);

async function main() {
    // Token addresses
    const btbMint = new web3.PublicKey("btbjSLvBfKFf94VTYbze6TtCXYaeBgCadTcLfvoZp9d");
    const usdtMint = new web3.PublicKey("usddpqpxr3LAu2HL95YJ4JJ4LFGFumAv7iaUhHYbmiQ");
    
    // Owner's initialize wallet (from your initialized state)
    const ownerInitializeWallet = new web3.PublicKey("sibxc42SdHMtovWeFzHihDMyENg9Dzf3vLWjxpt1xHo");
    
    // Derive PDA for BTB sale account
    const [btbSaleAccount] = await web3.PublicKey.findProgramAddress(
        [Buffer.from("btb-sale-account"), ownerInitializeWallet.toBuffer()],
        program.programId
    );

    try {
        // Get user's USDT token account
        const userUsdtAccount = await getAssociatedTokenAddress(
            usdtMint,
            userKeypair.publicKey
        );

        // Get owner's USDT receiving account
        const ownerTokenReceiveWallet = new web3.PublicKey("te6eqhHuXFuhP1bjBfPs17VS84dR1M725FR9txASuCS");
        const ownerUsdtAccount = await getAssociatedTokenAddress(
            usdtMint,
            ownerTokenReceiveWallet
        );

        // Get BTB sale token account (PDA's token account)
        const btbSaleTokenAccount = await getAssociatedTokenAddress(
            btbMint,
            btbSaleAccount,
            true
        );

        // Get or create user's BTB token account
        const userBtbAccount = await getAssociatedTokenAddress(
            btbMint,
            userKeypair.publicKey
        );

        // Amount of BTB tokens to buy
        const amount = new BN(1000000); // Buying 1000 BTB tokens
        const tokenType = 1; // 1 for USDT

        const tx = await program.methods.buyToken(
            amount,
            tokenType
        )
        .accounts({
            btbSaleAccount: btbSaleAccount,
            userTokenAccount: userUsdtAccount,
            ownerTokenAccount: ownerUsdtAccount,
            btbSaleTokenAccount: btbSaleTokenAccount,
            userBtbAccount: userBtbAccount,
            btbMintAccount: btbMint,
            user: userKeypair.publicKey,
            systemProgram: web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        })
        .signers([userKeypair])
        .rpc();
        
        console.log("Buy Token Transaction Signature:", tx);
        
    } catch (error) {
        console.error("Error during buy token transaction:", error);
    }
}

// Execute the main function with error handling
main().catch((error) => {
    console.error("Program execution failed:", error);
    process.exit(1);
});