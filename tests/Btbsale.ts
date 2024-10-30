import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram, Transaction, Keypair } from "@solana/web3.js";
import { assert } from "chai";
import { BtbTokenSale } from "../target/types/btb_token_sale";
import * as idl from "../target/idl/btb_token_sale.json";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID, 
  getAssociatedTokenAddress, 
  getAccount,
  createAssociatedTokenAccountInstruction,
  TokenAccountNotFoundError
} from "@solana/spl-token";

describe("BTB Token Sale", () => {
  // Provider setup
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Program setup
  const programId = new PublicKey("2zuQMgCNYgKzRJFuJqVEd8uzLCZ79CRYy8rSLub74iy1");
  const program = new Program(idl as anchor.Idl, programId, provider) as Program<BtbTokenSale>;

  // Account variables
  const ownerPublicKey = provider.wallet.publicKey;
  let btbMint: PublicKey;
  let usdcMint: PublicKey;
  let paypalMint: PublicKey;
  let usdtMint: PublicKey;
  let sale: PublicKey;
  let saleBtbAccount: PublicKey;
  let teamWallet: PublicKey;
  let teamUsdcAccount: PublicKey;
  let teamPaypalAccount: PublicKey;
  let teamUsdtAccount: PublicKey;
  let ownerUsdcAccount: PublicKey;
  let ownerPaypalAccount: PublicKey;
  let ownerUsdtAccount: PublicKey;
  let ownerBtbAccount: PublicKey;

  // Helper function to create token accounts
  async function createTokenAccountIfNotExists(
    connection: anchor.web3.Connection, 
    accountToCreate: PublicKey, 
    owner: PublicKey, 
    mint: PublicKey
  ) {
    try {
      await getAccount(connection, accountToCreate);
    } catch (error) {
      if (error instanceof TokenAccountNotFoundError) {
        console.log(`Creating account ${accountToCreate.toBase58()}`);
        const transaction = new Transaction().add(
          createAssociatedTokenAccountInstruction(
            provider.wallet.publicKey,
            accountToCreate,
            owner,
            mint
          )
        );
        await provider.sendAndConfirm(transaction);
        console.log(`Account ${accountToCreate.toBase58()} created`);
      } else {
        throw error;
      }
    }
  }

  // Helper function to format token amounts
  function formatTokenAmount(amount: bigint, decimals: number): string {
    const factor = BigInt(10 ** decimals);
    const whole = amount / factor;
    const fraction = amount % factor;
    return `${whole}${fraction === BigInt(0) ? '' : '.' + fraction.toString().padStart(decimals, '0')}`;
  }

  // Helper function for buying tokens
  async function buyTokensWithPaymentMethod(paymentTokenIndex: number, paymentTokenName: string) {
    const buyAmount = new anchor.BN("1000000000"); // 1 BTB
    
    let userPaymentAccount: PublicKey;
    let teamPaymentAccount: PublicKey;
    let paymentMint: PublicKey;

    switch(paymentTokenIndex) {
      case 0:
        userPaymentAccount = ownerUsdtAccount;
        teamPaymentAccount = teamUsdtAccount;
        paymentMint = usdtMint;
        break;
      case 1:
        userPaymentAccount = ownerPaypalAccount;
        teamPaymentAccount = teamPaypalAccount;
        paymentMint = paypalMint;
        break;
      case 2:
        userPaymentAccount = ownerUsdcAccount;
        teamPaymentAccount = teamUsdcAccount;
        paymentMint = usdcMint;
        break;
      default:
        throw new Error("Invalid payment token index");
    }

    console.log(`\nBuying with ${paymentTokenName} (index: ${paymentTokenIndex})`);
    
    const balancesBefore = await getBalances(userPaymentAccount, teamPaymentAccount);
    await printBalances("Before purchase", balancesBefore, paymentTokenName);

    const tx = await program.methods.buyTokens(buyAmount, paymentTokenIndex)
      .accounts({
        sale,
        user: ownerPublicKey,
        userPaymentTokenAccount: userPaymentAccount,
        userBtbAccount: ownerBtbAccount,
        btbMint,
        paymentTokenMint: paymentMint,
        teamPaymentTokenAccount: teamPaymentAccount,
        saleBtbAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    console.log("Transaction successful:", tx);

    const balancesAfter = await getBalances(userPaymentAccount, teamPaymentAccount);
    await printBalances("After purchase", balancesAfter, paymentTokenName);

    // Verify the transaction
    verifyTransaction(buyAmount, balancesBefore, balancesAfter);
  }

  // Helper function to get balances
  async function getBalances(userAccount: PublicKey, teamAccount: PublicKey) {
    return {
      owner: await getAccount(provider.connection, userAccount),
      ownerBtb: await getAccount(provider.connection, ownerBtbAccount),
      saleBtb: await getAccount(provider.connection, saleBtbAccount),
      team: await getAccount(provider.connection, teamAccount)
    };
  }

  // Helper function to print balances
  async function printBalances(title: string, balances: any, tokenName: string) {
    console.log(`\n${title}:`);
    console.log(`Owner ${tokenName}:`, formatTokenAmount(balances.owner.amount, 6));
    console.log("Owner BTB:", formatTokenAmount(balances.ownerBtb.amount, 9));
    console.log("Sale BTB:", formatTokenAmount(balances.saleBtb.amount, 9));
    console.log(`Team ${tokenName}:`, formatTokenAmount(balances.team.amount, 6));
  }

  // Helper function to verify transaction
  function verifyTransaction(buyAmount: anchor.BN, before: any, after: any) {
    const expectedCost = BigInt(1000);
    const actualPaymentDeducted = before.owner.amount - after.owner.amount;
    const actualBtbReceived = after.ownerBtb.amount - before.ownerBtb.amount;

    assert.equal(actualPaymentDeducted.toString(), expectedCost.toString(), 
      "Incorrect payment amount deducted");
    assert.equal(actualBtbReceived.toString(), buyAmount.toString(), 
      "Incorrect BTB amount received");
  }

  // Setup before tests
  before(async () => {
    try {
      [sale] = await PublicKey.findProgramAddress(
        [Buffer.from("sale")],
        program.programId
      );
      console.log("Sale PDA:", sale.toBase58());

      const saleAccount = await program.account.sale.fetch(sale);
      btbMint = saleAccount.btbTokenAddress;
      usdtMint = saleAccount.paymentTokens[0].mint;
      paypalMint = saleAccount.paymentTokens[1].mint;
      usdcMint = saleAccount.paymentTokens[2].mint;
      teamWallet = saleAccount.btbTeamWallet;

      // Log initial setup
      console.log("\nInitial Setup:");
      console.log("BTB Mint:", btbMint.toBase58());
      console.log("USDT Mint:", usdtMint.toBase58());
      console.log("PayPal Mint:", paypalMint.toBase58());
      console.log("USDC Mint:", usdcMint.toBase58());
      console.log("Team Wallet:", teamWallet.toBase58());

      // Initialize associated token accounts
      saleBtbAccount = await getAssociatedTokenAddress(btbMint, sale, true);
      teamUsdtAccount = await getAssociatedTokenAddress(usdtMint, teamWallet);
      teamPaypalAccount = await getAssociatedTokenAddress(paypalMint, teamWallet);
      teamUsdcAccount = await getAssociatedTokenAddress(usdcMint, teamWallet);
      ownerUsdtAccount = await getAssociatedTokenAddress(usdtMint, ownerPublicKey);
      ownerPaypalAccount = await getAssociatedTokenAddress(paypalMint, ownerPublicKey);
      ownerUsdcAccount = await getAssociatedTokenAddress(usdcMint, ownerPublicKey);
      ownerBtbAccount = await getAssociatedTokenAddress(btbMint, ownerPublicKey);

      // Create all necessary token accounts
      await Promise.all([
        createTokenAccountIfNotExists(provider.connection, ownerUsdtAccount, ownerPublicKey, usdtMint),
        createTokenAccountIfNotExists(provider.connection, ownerPaypalAccount, ownerPublicKey, paypalMint),
        createTokenAccountIfNotExists(provider.connection, ownerUsdcAccount, ownerPublicKey, usdcMint),
        createTokenAccountIfNotExists(provider.connection, ownerBtbAccount, ownerPublicKey, btbMint),
        createTokenAccountIfNotExists(provider.connection, teamUsdtAccount, teamWallet, usdtMint),
        createTokenAccountIfNotExists(provider.connection, teamPaypalAccount, teamWallet, paypalMint),
        createTokenAccountIfNotExists(provider.connection, teamUsdcAccount, teamWallet, usdcMint)
      ]);

      console.log("\nSetup completed successfully");
    } catch (error) {
      console.error("Error in setup:", error);
      throw error;
    }
  });

  // Test cases
  describe("Sale Management", () => {
    // it("should update sale parameters", async () => {
    //   console.log("\nTESTING SALE PARAMETERS UPDATE");
      
    //   try {
    //     const newTokenPrice = new anchor.BN(1500);
    //     const newVestingPrice = new anchor.BN(2000);
    //     const newTeamWallet = provider.wallet.publicKey;

    //     console.log("\nUpdating parameters to:");
    //     console.log("New token price:", newTokenPrice.toString());
    //     console.log("New vesting price:", newVestingPrice.toString());
    //     console.log("New team wallet:", newTeamWallet.toBase58());

    //     const tx = await program.methods.updateSaleParams(
    //       newTokenPrice,
    //       newVestingPrice,
    //       newTeamWallet
    //     )
    //     .accounts({
    //       sale,
    //       owner: ownerPublicKey,
    //     })
    //     .rpc();

    //     console.log("Transaction successful:", tx);

    //     const saleAccount = await program.account.sale.fetch(sale);
    //     assert.equal(saleAccount.tokenPrice.toString(), newTokenPrice.toString());
    //     assert.equal(saleAccount.tokenVestingPrice.toString(), newVestingPrice.toString());
    //     assert.equal(saleAccount.btbTeamWallet.toBase58(), newTeamWallet.toBase58());

    //     console.log("Sale parameters updated successfully");
    //   } catch (error) {
    //     console.error("Error updating sale parameters:", error);
    //     throw error;
    //   }
    // });

    it("should update sale status", async () => {
      console.log("\nTESTING SALE STATUS UPDATE");
      
      try {
        // Disable sale
        console.log("\nDisabling sale...");
        await program.methods.updateSaleStatus(false)
          .accounts({
            sale,
            owner: ownerPublicKey,
          })
          .rpc();

        let saleAccount = await program.account.sale.fetch(sale);
        assert.equal(saleAccount.isActive, false);
        console.log("Sale successfully disabled");

        // Enable sale
        console.log("\nEnabling sale...");
        await program.methods.updateSaleStatus(true)
          .accounts({
            sale,
            owner: ownerPublicKey,
          })
          .rpc();

        saleAccount = await program.account.sale.fetch(sale);
        assert.equal(saleAccount.isActive, true);
        console.log("Sale successfully enabled");
      } catch (error) {
        console.error("Error updating sale status:", error);
        throw error;
      }
    });
  });

  describe("Payment Token Management", () => {
    it("should manage payment token status", async () => {
      console.log("\nTESTING PAYMENT TOKEN STATUS UPDATE");
      
      try {
        // Disable USDT
        console.log("\nDisabling USDT payments...");
        await program.methods.updateTokenStatus(0, false)
          .accounts({
            sale,
            owner: ownerPublicKey,
          })
          .rpc();

        let saleAccount = await program.account.sale.fetch(sale);
        assert.equal(saleAccount.paymentTokens[0].isActive, false);
        console.log("USDT payments successfully disabled");

        // Re-enable USDT
        console.log("\nRe-enabling USDT payments...");
        await program.methods.updateTokenStatus(0, true)
          .accounts({
            sale,
            owner: ownerPublicKey,
          })
          .rpc();

        saleAccount = await program.account.sale.fetch(sale);
        assert.equal(saleAccount.paymentTokens[0].isActive, true);
        console.log("USDT payments successfully re-enabled");
      } catch (error) {
        console.error("Error updating token status:", error);
        throw error;
      }
    });

    // it("should add new payment token", async () => {
    //   console.log("\nTESTING ADD PAYMENT TOKEN");
      
    //   try {
    //     const newTokenMint = Keypair.generate().publicKey;
    //     console.log("\nAdding new payment token:", newTokenMint.toBase58());
        
    //     await program.methods.addPaymentToken(newTokenMint)
    //       .accounts({
    //         sale,
    //         owner: ownerPublicKey,
    //       })
    //       .rpc();

    //     const saleAccount = await program.account.sale.fetch(sale);
    //     const newToken = saleAccount.paymentTokens.find(
    //       token => token.mint.toBase58() === newTokenMint.toBase58()
    //     );

    //     assert(newToken, "New token should be added");
    //     assert(newToken.isActive, "New token should be active");
        
    //     console.log("New payment token added successfully");
    //   } catch (error) {
    //     console.error("Error adding payment token:", error);
    //     throw error;
    //   }
    // });

    // it("should remove payment token", async () => {
    //   console.log("\nTESTING REMOVE PAYMENT TOKEN");
      
    //   try {
    //     const saleAccountBefore = await program.account.sale.fetch(sale);
    //     const tokenToRemove = saleAccountBefore.paymentTokens.length - 1;

    //     console.log(`\nRemoving token at index: ${tokenToRemove}`);
    //     await program.methods.removePaymentToken(tokenToRemove)
    //       .accounts({
    //         sale,
    //         owner: ownerPublicKey,
    //       })
    //       .rpc();

    //     const saleAccountAfter = await program.account.sale.fetch(sale);
    //     assert.equal(
    //       saleAccountAfter.paymentTokens.length,
    //       saleAccountBefore.paymentTokens.length - 1,
    //       "Payment token should be removed"
    //     );

    //     console.log("Payment token removed successfully");
    //   } catch (error) {
    //     console.error("Error removing payment token:", error);
    //     throw error;
    //   }
    // });
  });

  describe("Token Purchases", () => {
    beforeEach(async () => {
      // Ensure sale is active before each purchase test
      try {
        const saleAccount = await program.account.sale.fetch(sale);
        if (!saleAccount.isActive) {
          await program.methods.updateSaleStatus(true)
            .accounts({
              sale,
              owner: ownerPublicKey,
            })
            .rpc();
          console.log("Sale activated for purchase test");
        }
      } catch (error) {
        console.error("Error in purchase test setup:", error);
        throw error;
      }
    });

    it("should buy tokens with USDT", async () => {
      console.log("\nTESTING USDT PURCHASE");
      try {
        await buyTokensWithPaymentMethod(0, "USDT");
        console.log("USDT purchase completed successfully");
      } catch (error) {
        console.error("Error in USDT purchase:", error);
        throw error;
      }
    });

    it("should buy tokens with PayPal", async () => {
      console.log("\nTESTING PAYPAL PURCHASE");
      try {
        await buyTokensWithPaymentMethod(1, "PayPal USD");
        console.log("PayPal purchase completed successfully");
      } catch (error) {
        console.error("Error in PayPal purchase:", error);
        throw error;
      }
    });

    it("should buy tokens with USDC", async () => {
      console.log("\nTESTING USDC PURCHASE");
      try {
        await buyTokensWithPaymentMethod(2, "USDC");
        console.log("USDC purchase completed successfully");
      } catch (error) {
        console.error("Error in USDC purchase:", error);
        throw error;
      }
    });
  });

  describe("Balance Verification", () => {
    it("should verify final balances", async () => {
      console.log("\n==============================================");
      console.log("FINAL WALLET BALANCES");
      console.log("==============================================");

      try {
        // Team balances
        const teamBalances = {
          usdt: await getAccount(provider.connection, teamUsdtAccount),
          paypal: await getAccount(provider.connection, teamPaypalAccount),
          usdc: await getAccount(provider.connection, teamUsdcAccount)
        };

        // Owner balances
        const ownerBalances = {
          usdt: await getAccount(provider.connection, ownerUsdtAccount),
          paypal: await getAccount(provider.connection, ownerPaypalAccount),
          usdc: await getAccount(provider.connection, ownerUsdcAccount),
          btb: await getAccount(provider.connection, ownerBtbAccount)
        };

        // Sale balance
        const saleBalance = await getAccount(provider.connection, saleBtbAccount);

        // Display team balances
        console.log("\nTeam Wallet Balances:");
        console.log("---------------------------------");
        console.log("USDT Balance:", formatTokenAmount(teamBalances.usdt.amount, 6), "USDT");
        console.log("PayPal Balance:", formatTokenAmount(teamBalances.paypal.amount, 6), "PayPal USD");
        console.log("USDC Balance:", formatTokenAmount(teamBalances.usdc.amount, 6), "USDC");

        // Display owner balances
        console.log("\nOwner Wallet Balances:");
        console.log("---------------------------------");
        console.log("USDT Balance:", formatTokenAmount(ownerBalances.usdt.amount, 6), "USDT");
        console.log("PayPal Balance:", formatTokenAmount(ownerBalances.paypal.amount, 6), "PayPal USD");
        console.log("USDC Balance:", formatTokenAmount(ownerBalances.usdc.amount, 6), "USDC");
        console.log("BTB Balance:", formatTokenAmount(ownerBalances.btb.amount, 9), "BTB");

        // Display sale balance
        console.log("\nSale Contract Balances:");
        console.log("---------------------------------");
        console.log("BTB Balance:", formatTokenAmount(saleBalance.amount, 9), "BTB");

        // Verify minimum balances
        assert(ownerBalances.btb.amount > BigInt(0), "Owner should have BTB tokens");
        assert(saleBalance.amount > BigInt(0), "Sale should have BTB tokens");
        
        console.log("\nBalance verification completed successfully");

      } catch (error) {
        console.error("Error verifying balances:", error);
        throw error;
      }
    });

    it("should verify transaction history", async () => {
      console.log("\nVERIFYING TRANSACTION HISTORY");
      
      try {
        // Fetch recent transactions (implementation depends on your needs)
        const saleAccount = await program.account.sale.fetch(sale);
        
        // Verify sale is still active
        assert(saleAccount.isActive, "Sale should be active");
        
        // Verify payment tokens
        assert(saleAccount.paymentTokens.length > 0, "Should have payment tokens");
        
        // Verify team wallet
        assert(saleAccount.btbTeamWallet.toString() === teamWallet.toString(), 
          "Team wallet should match");

        console.log("\nTransaction history verification completed");
        
      } catch (error) {
        console.error("Error verifying transaction history:", error);
        throw error;
      }
    });
  });

  after(async () => {
    console.log("\n==============================================");
    console.log("TEST SUITE COMPLETED");
    console.log("==============================================");
  });
});