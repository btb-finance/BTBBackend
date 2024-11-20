use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};

pub struct TestContext {
    pub program: Program,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
}

pub async fn setup_test_context() -> TestContext {
    let program_test = ProgramTest::new(
        "btb_finance_clmm",
        crate::ID,
        processor!(crate::entry),
    );

    let mut context = program_test.start_with_context().await;
    let payer = context.payer.clone();
    let recent_blockhash = context.last_blockhash;

    TestContext {
        program: context.program,
        payer,
        recent_blockhash,
    }
}

pub async fn create_mint(
    context: &mut TestContext,
    authority: &Keypair,
    decimals: u8,
) -> Pubkey {
    let mint = Keypair::new();
    let rent = context.program.get_rent().await.unwrap();
    let space = Mint::LEN;
    let lamports = rent.minimum_balance(space);

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                lamports,
                space as u64,
                &Token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                &authority.pubkey(),
                None,
                decimals,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint],
        context.recent_blockhash,
    );

    context.program.process_transaction(tx).await.unwrap();
    mint.pubkey()
}

pub async fn create_token_account(
    context: &mut TestContext,
    mint: &Pubkey,
    owner: &Pubkey,
) -> Pubkey {
    let token_account = Keypair::new();
    let rent = context.program.get_rent().await.unwrap();
    let space = TokenAccount::LEN;
    let lamports = rent.minimum_balance(space);

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &token_account.pubkey(),
                lamports,
                space as u64,
                &Token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &token_account.pubkey(),
                mint,
                owner,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &token_account],
        context.recent_blockhash,
    );

    context.program.process_transaction(tx).await.unwrap();
    token_account.pubkey()
}

pub async fn mint_tokens(
    context: &mut TestContext,
    mint: &Pubkey,
    token_account: &Pubkey,
    authority: &Keypair,
    amount: u64,
) {
    let tx = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            token_account,
            &authority.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&context.payer.pubkey()),
        &[&context.payer, authority],
        context.recent_blockhash,
    );

    context.program.process_transaction(tx).await.unwrap();
}
