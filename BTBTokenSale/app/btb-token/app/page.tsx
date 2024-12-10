"use client";
import { useState, useEffect } from "react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";
import { useAnchorWallet } from "@solana/wallet-adapter-react";
import { Connection, PublicKey, SystemProgram } from "@solana/web3.js";
import { AnchorProvider, BN, Program } from "@coral-xyz/anchor";
import idl from "./client/pda_vesting.json";
import type { PdaVesting } from "@/pda_vesting";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { ToastContainer, toast } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";

type PaymentMethod = 'usdt' | 'usdc' | 'paypal';

type PaymentMethodConfig = {
  [K in PaymentMethod]: {
    mint: PublicKey;
    tokenType: number;
    label: string;
  }
};

export default function Home() {
  const wallet = useAnchorWallet();
  const [mounted, setMounted] = useState(false);
  const [inputValue, setInputValue] = useState('');
  const [selectedPaymentMethod, setSelectedPaymentMethod] = useState<PaymentMethod>('usdt');
  const [btbPrice, setBtbPrice] = useState<number | null>(null);
  
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");
  const provider = wallet ? new AnchorProvider(connection, wallet as any, {
    commitment: "confirmed",
  }) : null;

  // Constants
  const btbMint = new PublicKey("btbVv5dmAjutpRRSr6DKwBPyPyfKiJw4eXU11BPuTCK");
  const usdtMint = new PublicKey("utK7s5CmT6vvkd3JpTg5CfMaqAS8uVMwnqZjPZvcLkD");
  const usdcMint = new PublicKey("ucKymLwwPxrPaUDMtYGx5uoho91SfE3Qs2VuXf9dDZB");
  const paypalMint = new PublicKey("pa3x7zKXd2yvPNM8NxJUp1tu1j8xeXyRb6Y65yqPvtQ");
  const ownerInitializeWallet = new PublicKey("kk4JSSv7f5GX3ePkB9GKvTEP1n59ZrX1oVxLtXuodC4");
  const ownerTokenReceiveWallet = new PublicKey("te6eqhHuXFuhP1bjBfPs17VS84dR1M725FR9txASuCS");

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    async function setupPriceUpdates() {
      if (!provider || !wallet) return;

      try {
        const program = new Program<PdaVesting>(JSON.parse(JSON.stringify(idl)), provider);
        const [btbSaleAccount] = await PublicKey.findProgramAddress(
          [Buffer.from("btb-sale-account"), ownerInitializeWallet.toBuffer()],
          program.programId
        );

        // Initial price fetch
        const accountInfo = await program.account.initializeDataAccount.fetch(btbSaleAccount);
        setBtbPrice(accountInfo.btbPrice.toNumber() / 1_000_000);

        // Listen for price updates
        const subscriptionId = connection.onAccountChange(
          btbSaleAccount,
          async () => {
            const updatedInfo = await program.account.initializeDataAccount.fetch(btbSaleAccount);
            setBtbPrice(updatedInfo.btbPrice.toNumber() / 1_000_000);
          },
          'confirmed'
        );

        return () => connection.removeAccountChangeListener(subscriptionId);
      } catch (error) {
        console.error("Error setting up price updates:", error);
      }
    }

    setupPriceUpdates();
  }, [provider, wallet, connection]);

  // Payment method configurations
  const paymentMethods: PaymentMethodConfig = {
    usdt: { mint: usdtMint, tokenType: 1, label: 'USDT' },
    usdc: { mint: usdcMint, tokenType: 2, label: 'USDC' },
    paypal: { mint: paypalMint, tokenType: 3, label: 'PayPal USD' }
  };

  const handleSum = async () => {
    if (!provider || !wallet) {
      toast.error("Please connect your wallet first!");
      return;
    }

    try {
      const idlString = JSON.parse(JSON.stringify(idl));
      const program = new Program<PdaVesting>(idlString, provider);

      const numericValue = parseFloat(inputValue) || 0;
      const amount = new BN(numericValue * 1_000_000);
      const selectedMethod = paymentMethods[selectedPaymentMethod];

      const [btbSaleAccount] = await PublicKey.findProgramAddress(
        [Buffer.from("btb-sale-account"), ownerInitializeWallet.toBuffer()],
        program.programId
      );

      const userTokenAccount = await getAssociatedTokenAddress(
        selectedMethod.mint,
        provider.wallet.publicKey
      );

      const ownerTokenAccount = await getAssociatedTokenAddress(
        selectedMethod.mint,
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
        selectedMethod.tokenType
      )
      .accounts({
        btbSaleAccount: btbSaleAccount,
        userTokenAccount: userTokenAccount,
        ownerTokenAccount: ownerTokenAccount,
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
          <p className="text-white text-xl mt-2">
            Current Price: {btbPrice !== null ? `${btbPrice} USD` : 'Loading...'}
          </p>
        </div>
        
        <div className="flex flex-col gap-4 w-full max-w-md mt-4">
          <div className="flex gap-2">
            <select
              value={selectedPaymentMethod}
              onChange={(e) => setSelectedPaymentMethod(e.target.value as PaymentMethod)}
              className="select select-bordered h-12 text-center text-gray-900 border-2 border-gray-300 rounded-md focus:outline-none focus:border-red-500 focus:ring-0 w-1/3"
            >
              {Object.entries(paymentMethods).map(([key, method]) => (
                <option key={key} value={key}>
                  {method.label}
                </option>
              ))}
            </select>
            
            <input
              type="number"
              placeholder="How much USD would you like to invest?"
              className="input input-bordered h-12 text-center text-gray-900 border-2 border-gray-300 rounded-md focus:outline-none focus:border-red-500 focus:ring-0 w-2/3"
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
            />
          </div>
        </div>
        
        <button
          className="btn lg:btn-md py-2 px-4 text-white rounded shadow hover:bg-red-700 transition duration-300 ease-in-out w-48 mt-4"
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