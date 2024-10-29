"use client";
import { useState, useEffect } from "react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { useAnchorWallet } from "@solana/wallet-adapter-react";
import { Connection, LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";
import { AnchorProvider, BN, Program } from "@coral-xyz/anchor";
import idl from "./client/pda_vesting.json";
import type { PdaVesting } from "@/pda_vesting";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { ToastContainer, toast } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";

export default function Home() {
  const wallet = useAnchorWallet();
  const [mounted, setMounted] = useState(false);
  const [inputValue, setInputValue] = useState('');
  
  // Initialize connection and provider only after mounting
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const provider = wallet ? new AnchorProvider(connection, wallet as any, {
    commitment: "confirmed",
  }) : null;

  useEffect(() => {
    setMounted(true);
  }, []);

  // Constants
  const btbMint = new PublicKey("btbjSLvBfKFf94VTYbze6TtCXYaeBgCadTcLfvoZp9d");
  const usdtMint = new PublicKey("usddpqpxr3LAu2HL95YJ4JJ4LFGFumAv7iaUhHYbmiQ");
  const ownerInitializeWallet = new PublicKey("sibxc42SdHMtovWeFzHihDMyENg9Dzf3vLWjxpt1xHo");
  const ownerTokenReceiveWallet = new PublicKey("te6eqhHuXFuhP1bjBfPs17VS84dR1M725FR9txASuCS");

  const handleSum = async () => {
    if (!provider || !wallet) {
      toast.error("Please connect your wallet first!");
      return;
    }

    try {
      const idlString = JSON.parse(JSON.stringify(idl));
      const program = new Program<PdaVesting>(idlString, provider);

      const numericValue = parseFloat(inputValue) || 0;
      const amount = new BN(numericValue * LAMPORTS_PER_SOL);
      const tokenType = 1;

      const [btbSaleAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("btb-sale-account"), ownerInitializeWallet.toBuffer()],
        program.programId
      );

      const userUsdtAccount = await getAssociatedTokenAddress(
        usdtMint,
        provider.wallet.publicKey
      );

      const ownerUsdtAccount = await getAssociatedTokenAddress(
        usdtMint,
        ownerTokenReceiveWallet
      );

      const btbSaleTokenAccount = await getAssociatedTokenAddress(
        btbMint,
        btbSaleAccount,
        true
      );

      const userBtbAccount = await getAssociatedTokenAddress(
        btbMint,
        provider.wallet.publicKey
      );

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
        user: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      }).rpc();

      console.log("Buy Token Transaction Signature:", tx);
      toast.success("Transaction completed!");
    } catch (error) {
      console.error("Transaction failed:", error);
      toast.error("Transaction failed: " + (error as Error).message);
    }
  };

  // Prevent hydration issues by not rendering until mounted
  if (!mounted) {
    return null;
  }

  return (
    <main className="flex flex-col h-screen p-4">
      <div className="flex items-center justify-between w-full p-1">
        <img src="/btb-logo.png" className="w-14 h-14" alt="Logo" />
        <div className="p-0 rounded-xl relative" style={{ background: "#ed1e28" }}>
          <div className="wallet-adapter-button-trigger">
            <WalletMultiButton className="absolute top-0 left-0 w-full h-full" style={{ backgroundColor: "#ed1e28", border: "none" }} />
      
          </div>
        </div>
      </div>

      <div className="flex-grow flex flex-col items-center justify-center">
        <div className="text-center mt-4">
          <h1 className="text-white text-3xl font-bold">BTB Finance</h1>
          <p className="text-white text-lg mt-1">The Future of Finance is Decentralized!</p>
        </div>
        
        <input
          type="number"
          placeholder="How much USD would you like to invest?"
          className="input input-bordered mt-4 mb-4 w-full max-w-md h-12 text-center text-gray-900 border-2 border-gray-300 rounded-md focus:outline-none focus:border-red-500 focus:ring-0"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
        />
        
        <button
          className="btn lg:btn-md py-2 px-4 text-white rounded shadow hover:bg-red-700 transition duration-300 ease-in-out w-48 mt-2"
          style={{ background: "#ed1e28" }}
          onClick={handleSum}
          disabled={!wallet}
        >
          {wallet ? "Buy Token" : "Connect Wallet First"}
        </button>
        
        <ToastContainer 
          position="bottom-left"
          theme="dark"
          autoClose={5000}
        />
      </div>
    </main>
  );
}