  "use client";
  import { useState } from "react";
  import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
  import { useAnchorWallet } from "@solana/wallet-adapter-react";
  import { Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
  import { AnchorProvider, BN, Idl, Program, setProvider } from "@coral-xyz/anchor";
  import idl from "./owner.json";
  import type { Owner } from "@/owner";
  import { getAssociatedTokenAddressSync, } from "@solana/spl-token";
  import { ToastContainer, toast } from "react-toastify";
  import "react-toastify/dist/ReactToastify.css";



  export default function Home() {
    const wallet = useAnchorWallet();

    const connection = new Connection("https://api.devnet.solana.com", "confirmed");
    const provider = new AnchorProvider(connection, wallet as any,{
      commitment: "confirmed",
    });
    setProvider(provider);

    const secretkey = Uint8Array.from([236,105,200,30,182,138,46,171,35,169,21,37,241,212,15,131,158,114,232,115,183,249,252,22,200,229,109,241,8,7,97,24,12,6,60,222,37,94,241,78,202,204,184,37,134,184,122,53,8,128,24,197,96,74,188,108,94,250,214,226,146,40,78,193]);
    const myKeypair = Keypair.fromSecretKey(secretkey);
  
    const ownerAccount = new PublicKey("owQjAAczpK3Mk9euh8Ax2vbiPt9BDh4wWiHMz9TV22c");

    const from = new PublicKey("BTBoZDhtMcpTt8dRMcVD8xMu7E9tTCGQYns3K9g46tPY");
    const ata = getAssociatedTokenAddressSync(from, myKeypair.publicKey );

    const to = new PublicKey("7jb92oGHRZBwggsanUmwMqmnfirnScnk7kTNfCxZj6Yu");
    const ata_2 = getAssociatedTokenAddressSync(from, to );

    const [inputValue, setInputValue] = useState('');
    
    const handleSum = async () => {

      const sol = new BN(Number(inputValue) * LAMPORTS_PER_SOL);
      const btb = new BN(Number(inputValue) * 30 * LAMPORTS_PER_SOL);

      console.log(sol);
    
      const idlString = JSON.parse(JSON.stringify(idl));
      
      const program = new Program<Owner>(idlString ,  provider);
  
      const tx = await program.methods.transferSolAndReceiveToken(sol ,btb).accounts({
        user: provider.wallet.publicKey,
        owner: ownerAccount,
        userTokenAccount: ata_2,
        ownerTokenAccount: ata,
    }).signers([ myKeypair]).rpc(); 

    console.log("Your transaction signature", tx);
    
    toast.success("Transaction completed!", {
      position: "bottom-left", 
      style: {
        backgroundColor: "#333", 
        color: "#fff",
      },
    });



    };

    return (
      
      <main className="flex flex-col h-screen p-4">
      <div className="flex items-center justify-between w-full p-1">

        <img src="/btb-logo.png" className="w-14 h-14" alt="Logo" />       
        
        <div style={{
            background: "#ed1e28",
          }} className="p-0 rounded-xl relative"> 
          <WalletMultiButton className="absolute top-0 left-0 w-full h-full" style={{ backgroundColor: "transparent", border: "none" }} />
        </div>
      </div>
      
      <div className="flex-grow flex flex-col items-center justify-center">
       
      <div className="text-center mt-4">
        <h1 className="text-white text-3xl font-bold">BTB Finance</h1>
        <p className="text-white text-lg mt-1">The Future of Finance is Decentralized!</p>
      </div>
        <input
          type="number"
          placeholder="How much SOL would you like to invest?"
          className="input input-bordered mt-4 mb-4 w-1/3 h-12 text-center text-gray-900 border-2 border-gray-300 rounded-md focus:outline-none focus:border-red-500 focus:ring-0"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
        />
        <ToastContainer />
        <button
          className="btn lg:btn-md py-2 px-4 text-white rounded shadow hover:bg-blue-600 transition duration-300 ease-in-out w-1/7 mt-2"
          style={{
            background: "red",
          }}
          onClick={handleSum}
          disabled={!wallet}
        >
          {wallet ? "Buy Token" : "Connect Wallet"}
        </button>
      </div>
    </main>
    
    

    );
  }



