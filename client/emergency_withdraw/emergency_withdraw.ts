import idl from "../../target/idl/pda_vesting.json";
import { PdaVesting } from "../../target/types/pda_vesting";
import { Program, Idl, AnchorProvider, setProvider, web3, Wallet } from "@coral-xyz/anchor";
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

// Load owner's keypair from wallet file
const ownerKeypair = loadWalletKey("owner_signer_wallet.json");

// Create wallet instance from keypair
const ownerWallet = new Wallet(ownerKeypair);

// Create Anchor provider with connection and wallet
const provider = new AnchorProvider(connection, ownerWallet, {});
setProvider(provider);

// Parse IDL and create program instance
const idlString = JSON.parse(JSON.stringify(idl));  
const program = new Program<PdaVesting>(idlString, provider);

async function main() {
    // BTB token mint address on devnet
    const btbMint = new web3.PublicKey("btbVv5dmAjutpRRSr6DKwBPyPyfKiJw4eXU11BPuTCK");
    
    // Derive PDA for BTB sale account
    const [btbSaleAccount] = await web3.PublicKey.findProgramAddress(
        [Buffer.from("btb-sale-account"), ownerKeypair.publicKey.toBuffer()],
        program.programId
    );
    
    // Get BTB token account for sale account (PDA)
    const btbSaleTokenAccount = await getAssociatedTokenAddress(
        btbMint,
        btbSaleAccount,
        true
    );

    // Get owner's BTB token account
    const ownerBtbAccount = await getAssociatedTokenAddress(
        btbMint,
        ownerKeypair.publicKey
    );
    
    console.log("BTB Sale Account (PDA):", btbSaleAccount.toString());
    console.log("BTB Sale Token Account:", btbSaleTokenAccount.toString());
    console.log("Owner BTB Account:", ownerBtbAccount.toString());

    try {
        // Get current balance before withdrawal
        const saleTokenAccount = await connection.getTokenAccountBalance(btbSaleTokenAccount);
        console.log("Current BTB balance in sale account:", saleTokenAccount.value.uiAmount);

        // Execute emergency_withdraw instruction
        const tx = await program.methods.emergencyWithdraw()
        .accounts({
            btbSaleAccount: btbSaleAccount,
            btbSaleTokenAccount: btbSaleTokenAccount,
            ownerBtbAccount: ownerBtbAccount,
            btbMintAccount: btbMint,
            signer: ownerWallet.publicKey,
            systemProgram: web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID
        })
        .signers([ownerKeypair])
        .rpc();
       
        console.log("Emergency withdrawal successful. Transaction signature:", tx);

        // Get new balance after withdrawal
        const newSaleBalance = await connection.getTokenAccountBalance(btbSaleTokenAccount);
        const ownerBalance = await connection.getTokenAccountBalance(ownerBtbAccount);
        
        console.log("New BTB balance in sale account:", newSaleBalance.value.uiAmount);
        console.log("Owner BTB balance:", ownerBalance.value.uiAmount);
        
    } catch (error) {
        console.error("Error during emergency withdrawal:", error);
    }
}

// Execute the main function
main().catch((error) => {
    console.error("Program execution failed:", error);
    process.exit(1);
});