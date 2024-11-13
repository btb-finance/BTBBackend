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
const currentAdminKeypair = loadWalletKey("ttZwVJp67UVCcDk7mAVzRVajEP2bozJJNUwQJ7KjEjN.json");

// Create wallet instance from keypair
const currentAdminWallet = new Wallet(currentAdminKeypair);

// Create Anchor provider with connection and wallet
const provider = new AnchorProvider(connection, currentAdminWallet, {});
setProvider(provider);

// Parse IDL and create program instance
const program = new Program<PdaVesting>(idl as Idl, provider);

async function findInitializedAccount() {
    // Get all program accounts
    const accounts = await program.account.initializeDataAccount.all();
    
    // Find the account where the current wallet is the owner
    const ourAccount = accounts.find(acc => 
        acc.account.ownerInitializeWallet.equals(currentAdminKeypair.publicKey)
    );
    
    if (!ourAccount) {
        throw new Error("No initialized account found for this wallet");
    }
    
    return ourAccount.publicKey;
}

async function main() {
    try {
        // Find the currently initialized account
        const btbSaleAccount = await findInitializedAccount();
        console.log("Found initialized BTB Sale Account:", btbSaleAccount.toString());

        // New admin's public key - replace with actual public key
        const newAdmin = new web3.PublicKey("kk4JSSv7f5GX3ePkB9GKvTEP1n59ZrX1oVxLtXuodC4");

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

        // Wait for confirmation
        await connection.confirmTransaction(tx);

        // Try to fetch the updated account info
        try {
            const updatedAccountInfo = await program.account.initializeDataAccount.fetch(btbSaleAccount);
            console.log("New admin address:", updatedAccountInfo.ownerInitializeWallet.toString());
        } catch (e) {
            console.log("Note: Account data fetch after transfer may fail - this is expected");
        }

    } catch (error) {
        console.error("Error during admin transfer:", error);
    }
}

// Execute the main function
main().catch((error) => {
    console.error("Program execution failed:", error);
    process.exit(1);
});