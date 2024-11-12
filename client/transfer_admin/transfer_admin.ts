import idl from "../../target/idl/pda_vesting.json";
import { PdaVesting } from "../../target/types/pda_vesting";
import { Program, Idl, AnchorProvider, setProvider, web3, Wallet } from "@coral-xyz/anchor";
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

// Load current admin's keypair from wallet file
const currentAdminKeypair = loadWalletKey("owner_signer_wallet.json");

// Create wallet instance from keypair
const currentAdminWallet = new Wallet(currentAdminKeypair);

// Create Anchor provider with connection and wallet
const provider = new AnchorProvider(connection, currentAdminWallet, {});
setProvider(provider);

// Parse IDL and create program instance
const idlString = JSON.parse(JSON.stringify(idl));  
const program = new Program<PdaVesting>(idlString, provider);

async function main() {
    // Derive PDA for BTB sale account
    const [btbSaleAccount] = await web3.PublicKey.findProgramAddress(
        [Buffer.from("btb-sale-account"), currentAdminKeypair.publicKey.toBuffer()],
        program.programId
    );
    
    // Log btbSaleAccount address
    console.log("BTB Sale Account (PDA):", btbSaleAccount.toString());

    // New admin's public key
    const newAdmin = new web3.PublicKey("ttZwVJp67UVCcDk7mAVzRVajEP2bozJJNUwQJ7KjEjN");

    try {
        // Execute transfer_admin instruction
        const tx = await program.methods.transferAdmin(newAdmin)
        .accounts({
            btbSaleAccount: btbSaleAccount,
            signer: currentAdminWallet.publicKey,
            systemProgram: web3.SystemProgram.programId,
        })
        .signers([currentAdminKeypair])
        .rpc();
       
        console.log("Admin transferred successfully. Transaction signature:", tx);

        // Fetch and log updated account info
        const accountInfo = await program.account.initializeDataAccount.fetch(btbSaleAccount);
        console.log("New admin address:", accountInfo.ownerInitializeWallet.toString());
        
    } catch (error) {
        console.error("Error during admin transfer:", error);
    }
}

// Execute the main function
main().catch((error) => {
    console.error("Program execution failed:", error);
    process.exit(1);
});