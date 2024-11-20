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

    const [programDataAddress] = web3.PublicKey.findProgramAddressSync(
        [program.programId.toBuffer()],
        new web3.PublicKey('BPFLoaderUpgradeab1e11111111111111111111111')
    );

    console.log("Program Data Account:", programDataAddress.toString());

    // Derive PDA for BTB sale account
    const [btbSaleAccount] = await web3.PublicKey.findProgramAddress(
        [Buffer.from("btb-sale-account"), ownerKeypair.publicKey.toBuffer()],
        program.programId
    );
    
    // Log btbSaleAccount address
    console.log("BTB Sale Account (PDA):", btbSaleAccount.toString());

    try {
        // Get current sale status before toggle
        const accountInfoBefore = await program.account.initializeDataAccount.fetch(btbSaleAccount);
        console.log("Current sale status:", accountInfoBefore.isSaleActive);

        // Execute toggle_sale instruction
        const tx = await program.methods.toggleSale()
        .accounts({
            btbSaleAccount: btbSaleAccount,
            programData: programDataAddress,
            signer: ownerWallet.publicKey,
            systemProgram: web3.SystemProgram.programId,
        })
        .signers([ownerKeypair])
        .rpc();
       
        console.log("Sale status toggled successfully. Transaction signature:", tx);

        // Get new sale status after toggle
        const accountInfoAfter = await program.account.initializeDataAccount.fetch(btbSaleAccount);
        console.log("New sale status:", accountInfoAfter.isSaleActive);
        
    } catch (error) {
        console.error("Error during sale toggle:", error);
    }
}

// Execute the main function
main().catch((error) => {
    console.error("Program execution failed:", error);
    process.exit(1);
});