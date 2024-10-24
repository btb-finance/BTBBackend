import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { assert } from "chai";
import { BtbTokenSale } from "../target/types/btb_token_sale";
import * as idl from "../target/idl/btb_token_sale.json";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID, 
  getAssociatedTokenAddress, 
  getAccount, 
  getMint 
} from "@solana/spl-token";

describe("btb_token_sale", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.BtbTokenSale as Program<BtbTokenSale>;
  const adminPublicKey = provider.wallet.publicKey;
  const btbMint = new PublicKey("BTBDnNJKCWpQJSy2KJhHq1CmRHDtvGQymqkhk3Kyivt4");
  
  let sale: PublicKey;
  let saleBtbAccount: PublicKey;

  before(async () => {
    [sale] = await PublicKey.findProgramAddress([Buffer.from("sale")], program.programId);
    console.log("Sale PDA:", sale.toBase58());

    saleBtbAccount = await getAssociatedTokenAddress(btbMint, sale, true);
    console.log("Sale BTB Associated Token Account:", saleBtbAccount.toBase58());
  });

  it("Fetch sale data", async () => {
    try {
      const saleAccount = await program.account.sale.fetch(sale);
      console.log("Fetched Sale Account Data:");
      console.log(JSON.stringify({
        pda: sale.toBase58(),
        btbTokenAddress: saleAccount.btbTokenAddress.toBase58(),
        btbTeamWallet: saleAccount.btbTeamWallet.toBase58(),
        tokenPrice: saleAccount.tokenPrice.toString(),
        tokenVestingPrice: saleAccount.tokenVestingPrice.toString(),
        owner: saleAccount.owner.toBase58(),
        isActive: saleAccount.isActive,
        paymentTokens: saleAccount.paymentTokens.map(token => ({
          mint: token.mint.toBase58(),
          isActive: token.isActive
        }))
      }, null, 2));

      assert.equal(saleAccount.btbTokenAddress.toBase58(), btbMint.toBase58());
      assert.equal(saleAccount.owner.toBase58(), adminPublicKey.toBase58());
    } catch (error) {
      console.error("Error in fetching sale data:", error);
      if (error instanceof anchor.AnchorError) {
        console.error("Error code:", error.error.errorCode.code);
        console.error("Error message:", error.error.errorMessage);
      }
      throw error;
    }
  });

  it("Check specific Associated Token Account", async () => {
    const specificATA = saleBtbAccount;
    try {
      const tokenAccountInfo = await getAccount(provider.connection, specificATA);
      
      console.log("Sale BTB Associated Token Account Details:");
      console.log("  Address:", specificATA.toBase58());
      console.log("  Mint:", tokenAccountInfo.mint.toBase58());
      console.log("  Owner:", tokenAccountInfo.owner.toBase58());
      console.log("  Amount:", tokenAccountInfo.amount.toString());

      const mintInfo = await getMint(provider.connection, tokenAccountInfo.mint);
      console.log("  Token Decimals:", mintInfo.decimals);
      
      const actualAmount = Number(tokenAccountInfo.amount) / Math.pow(10, mintInfo.decimals);
      console.log("  Actual Token Amount:", actualAmount);

    } catch (error) {
      if (error.name === "TokenAccountNotFoundError") {
        console.log("Token account not found:", specificATA.toBase58());
      } else {
        console.error("Error fetching token account details:", error);
      }
    }
  });

  it("Update sale status", async () => {
    try {
      await program.methods.updateSaleStatus(false)
        .accounts({
          sale,
          owner: adminPublicKey,
        })
        .rpc();

      const saleAccount = await program.account.sale.fetch(sale);
      assert.isFalse(saleAccount.isActive, "Sale should be inactive");
    } catch (error) {
      console.error("Error updating sale status:", error);
      throw error;
    }
  });

  it("Update payment token status", async () => {
    try {
      await program.methods.updateTokenStatus(0, false)
        .accounts({
          sale,
          owner: adminPublicKey,
        })
        .rpc();

      const saleAccount = await program.account.sale.fetch(sale);
      assert.isFalse(saleAccount.paymentTokens[0].isActive, "First payment token should be inactive");
    } catch (error) {
      console.error("Error updating token status:", error);
      throw error;
    }
  });
});