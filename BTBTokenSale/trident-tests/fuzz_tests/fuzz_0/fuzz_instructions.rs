use trident_client::fuzzing::*;
use solana_sdk::pubkey::Pubkey;
use trident_client::fuzzing::{AccountsStorage, PdaStore, KeypairStore};



/// FuzzInstruction contains all available Instructions.
/// Below, the instruction arguments (accounts and data) are defined.
#[derive(Arbitrary, DisplayIx, FuzzTestExecutor)]
pub enum FuzzInstruction {
    BuyToken(BuyToken),
    EmergencyWithdraw(EmergencyWithdraw),
    Initialize(Initialize),
    ToggleSale(ToggleSale),
    UpdateInitialize(UpdateInitialize),
}

#[derive(Arbitrary, Debug)]
pub struct BuyToken {
    pub accounts: BuyTokenAccounts,
    pub data: BuyTokenData,
}
#[derive(Arbitrary, Debug)]
pub struct BuyTokenAccounts {
    pub btb_sale_account: AccountId,
    pub user_token_account: AccountId,
    pub owner_token_account: AccountId,
    pub btb_sale_token_account: AccountId,
    pub user_btb_account: AccountId,
    pub btb_mint_account: AccountId,
    pub user: AccountId,
    pub system_program: AccountId,
    pub token_program: AccountId,
    pub associated_token_program: AccountId,
}

/// Custom data types must derive `Debug` and `Arbitrary`.
/// To do this, redefine the type in the fuzz test and implement the `From`
/// trait
/// to convert it into the type defined in the program.
/// For more details, see: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#custom-data-types
#[derive(Arbitrary, Debug)]
pub struct BuyTokenData {
    pub amount: u64,
    pub token_type: u8,
}

#[derive(Arbitrary, Debug)]
pub struct EmergencyWithdraw {
    pub accounts: EmergencyWithdrawAccounts,
    pub data: EmergencyWithdrawData,
}

#[derive(Arbitrary, Debug)]
pub struct EmergencyWithdrawAccounts {
    pub btb_sale_account: AccountId,
    pub btb_sale_token_account: AccountId,
    pub owner_btb_account: AccountId,
    pub btb_mint_account: AccountId,
    pub signer: AccountId,
    pub system_program: AccountId,
    pub token_program: AccountId,
}

/// Custom data types must derive `Debug` and `Arbitrary`.
/// To do this, redefine the type in the fuzz test and implement the `From`
/// trait
/// to convert it into the type defined in the program.
/// For more details, see: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#custom-data-types
#[derive(Arbitrary, Debug)]
pub struct EmergencyWithdrawData {}

#[derive(Arbitrary, Debug)]
pub struct Initialize {
    pub accounts: InitializeAccounts,
    pub data: InitializeData,
}

#[derive(Arbitrary, Debug)]
pub struct InitializeAccounts {
    pub btb_sale_account: AccountId,
    pub btb_sale_token_account: AccountId,
    pub btb_mint_account: AccountId,
    pub signer: AccountId,
    pub system_program: AccountId,
    pub token_program: AccountId,
    pub associated_token_program: AccountId,
}

/// Custom data types must derive `Debug` and `Arbitrary`.
/// To do this, redefine the type in the fuzz test and implement the `From`
/// trait
/// to convert it into the type defined in the program.
/// For more details, see: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#custom-data-types
#[derive(Arbitrary, Debug)]
pub struct InitializeData {
    pub btb: AccountId,
    pub usdt: AccountId,
    pub usdc: AccountId,
    pub paypal_usd: AccountId,
    pub owner_token_receive_wallet: AccountId,
    pub btb_price: u64,
    pub vesting_price: u64,
}

#[derive(Arbitrary, Debug)]
pub struct ToggleSale {
    pub accounts: ToggleSaleAccounts,
    pub data: ToggleSaleData,
}

#[derive(Arbitrary, Debug)]
pub struct ToggleSaleAccounts {
    pub btb_sale_account: AccountId,
    pub signer: AccountId,
    pub system_program: AccountId,
}

/// Custom data types must derive `Debug` and `Arbitrary`.
/// To do this, redefine the type in the fuzz test and implement the `From`
/// trait
/// to convert it into the type defined in the program.
/// For more details, see: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#custom-data-types
#[derive(Arbitrary, Debug)]
pub struct ToggleSaleData {}


#[derive(Arbitrary, Debug)]
pub struct UpdateInitialize {
    pub accounts: UpdateInitializeAccounts,
    pub data: UpdateInitializeData,
}

#[derive(Arbitrary, Debug)]
pub struct UpdateInitializeAccounts {
    pub btb_sale_account: AccountId,
    pub signer: AccountId,
    pub system_program: AccountId,
}

/// Custom data types must derive `Debug` and `Arbitrary`.
/// To do this, redefine the type in the fuzz test and implement the `From`
/// trait
/// to convert it into the type defined in the program.
/// For more details, see: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#custom-data-types
#[derive(Arbitrary, Debug)]
pub struct UpdateInitializeData {
    pub btb: AccountId,
    pub usdt: AccountId,
    pub usdc: AccountId,
    pub paypal_usd: AccountId,
    pub owner_token_receive_wallet: AccountId,
    pub btb_price: u64,
    pub vesting_price: u64,
}

///IxOps implementation for `BuyToken` with all required functions.
impl IxOps for BuyToken {
    type IxData = pda_vesting::instruction::BuyToken;
    type IxAccounts = FuzzAccounts;
    /// Definition of the program ID that the Instruction is associated with.
    fn get_program_id(&self) -> solana_sdk::pubkey::Pubkey {
        pda_vesting::ID
    }
    
    /// Definition of the Instruction data.
    /// Use randomly generated data from the fuzzer using `self.data.arg_name`
    /// or customize the data as needed.
    /// For more details, visit: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#get-data
    fn get_data(
        &self,
        _client: &mut impl FuzzClient,
        _fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<Self::IxData, FuzzingError> {
        let data = pda_vesting::instruction::BuyToken {
            amount: self.data.amount,
            token_type: self.data.token_type,
        };
        Ok(data)
    }
        
    /// Definition of of the accounts required by the Instruction.
    /// To utilize accounts stored in `FuzzAccounts`, use
    /// `fuzz_accounts.account_name.get_or_create_account()`.
    /// If no signers are required, leave the vector empty.
    /// For AccountMetas use <program>::accounts::<corresponding_metas>
    /// For more details, see: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#get-accounts
    fn get_accounts(
        &self,
        client: &mut impl FuzzClient,
        fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<(Vec<Keypair>, Vec<AccountMeta>), FuzzingError> {
        let signers = vec![todo!()];
        let acc_meta = todo!();
        Ok((signers, acc_meta))
    }

    /*
    fn get_accounts(
        &self,
        client: &mut impl FuzzClient,
        fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<(Vec<Keypair>, Vec<AccountMeta>), FuzzingError> {
        let signer = fuzz_accounts.user.get_or_create_account(client)?;
        let account_meta = vec![
            AccountMeta::new(fuzz_accounts.btb_sale_account.get_or_create_account(client)?, false),
            AccountMeta::new(fuzz_accounts.user_token_account.get_or_create_account(client)?, false),
            AccountMeta::new(fuzz_accounts.owner_token_account.get_or_create_account(client)?, true),
            AccountMeta::new(fuzz_accounts.btb_sale_token_account.get_or_create_account(client)?, false),
        ];
        Ok((vec![signer], account_meta))
    }
    
    fn get_accounts(
        &self,
        client: &mut impl FuzzClient,
        fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<(Vec<Keypair>, Vec<AccountMeta>), FuzzingError> {
        // Fetch required accounts using `get_or_create_account` and handle the results
        let btb_sale_token_account = fuzz_accounts
            .btb_sale_token_account
            .get_or_create_account(client)?;
        let btb_sale_account = fuzz_accounts
            .btb_sale_account
            .get_or_create_account(client)?;
        let user_btb_account = fuzz_accounts
            .user_btb_account
            .get_or_create_account(client)?;
    
        // Construct the account meta list
        let acc_meta = vec![
            AccountMeta::new(btb_sale_token_account, false),
            AccountMeta::new(btb_sale_account, false),
            AccountMeta::new(user_btb_account, false),
        ];
    
        let signer = fuzz_accounts.user.get_or_create_account(client)?;
        // Add any required signers
        let signers = vec![signer];
    
        Ok((signers, acc_meta))
    }
    */
}

///IxOps implementation for `EmergencyWithdraw` with all required functions.
impl IxOps for EmergencyWithdraw {
    type IxData = pda_vesting::instruction::EmergencyWithdraw;
    type IxAccounts = FuzzAccounts;
    /// Definition of the program ID that the Instruction is associated with.
    fn get_program_id(&self) -> solana_sdk::pubkey::Pubkey {
        pda_vesting::ID
    }
    /// Definition of the Instruction data.
    /// Use randomly generated data from the fuzzer using `self.data.arg_name`
    /// or customize the data as needed.
    /// For more details, visit: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#get-data
    fn get_data(
        &self,
        _client: &mut impl FuzzClient,
        _fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<Self::IxData, FuzzingError> {
        let data = pda_vesting::instruction::EmergencyWithdraw {};
        Ok(data)
    }
    /// Definition of of the accounts required by the Instruction.
    /// To utilize accounts stored in `FuzzAccounts`, use
    /// `fuzz_accounts.account_name.get_or_create_account()`.
    /// If no signers are required, leave the vector empty.
    /// For AccountMetas use <program>::accounts::<corresponding_metas>
    /// For more details, see: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#get-accounts
    fn get_accounts(
        &self,
        client: &mut impl FuzzClient,
        fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<(Vec<Keypair>, Vec<AccountMeta>), FuzzingError> {
        let signers = vec![todo!()];
        let acc_meta = todo!();
        Ok((signers, acc_meta))
    }
}

///IxOps implementation for `Initialize` with all required functions.
impl IxOps for Initialize {
    type IxData = pda_vesting::instruction::Initialize;
    type IxAccounts = FuzzAccounts;
    /// Definition of the program ID that the Instruction is associated with.
    fn get_program_id(&self) -> solana_sdk::pubkey::Pubkey {
        pda_vesting::ID
    }
    /// Definition of the Instruction data.
    /// Use randomly generated data from the fuzzer using `self.data.arg_name`
    /// or customize the data as needed.
    /// For more details, visit: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#get-data
    fn get_data(
        &self,
        _client: &mut impl FuzzClient,
        _fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<Self::IxData, FuzzingError> {
        let data = pda_vesting::instruction::Initialize {
            btb: todo!(),
            usdt: todo!(),
            usdc: todo!(),
            paypal_usd: todo!(),
            owner_token_receive_wallet: todo!(),
            btb_price: self.data.btb_price,
            vesting_price: self.data.vesting_price,
        };
        Ok(data)
    }
    /// Definition of of the accounts required by the Instruction.
    /// To utilize accounts stored in `FuzzAccounts`, use
    /// `fuzz_accounts.account_name.get_or_create_account()`.
    /// If no signers are required, leave the vector empty.
    /// For AccountMetas use <program>::accounts::<corresponding_metas>
    /// For more details, see: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#get-accounts
    fn get_accounts(
        &self,
        client: &mut impl FuzzClient,
        fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<(Vec<Keypair>, Vec<AccountMeta>), FuzzingError> {
        let signers = vec![todo!()];
        let acc_meta = todo!();
        Ok((signers, acc_meta))
    }
}

///IxOps implementation for `ToggleSale` with all required functions.
impl IxOps for ToggleSale {
    type IxData = pda_vesting::instruction::ToggleSale;
    type IxAccounts = FuzzAccounts;
    /// Definition of the program ID that the Instruction is associated with.
    fn get_program_id(&self) -> solana_sdk::pubkey::Pubkey {
        pda_vesting::ID
    }
    /// Definition of the Instruction data.
    /// Use randomly generated data from the fuzzer using `self.data.arg_name`
    /// or customize the data as needed.
    /// For more details, visit: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#get-data
    fn get_data(
        &self,
        _client: &mut impl FuzzClient,
        _fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<Self::IxData, FuzzingError> {
        let data = pda_vesting::instruction::ToggleSale {};
        Ok(data)
    }
    /// Definition of of the accounts required by the Instruction.
    /// To utilize accounts stored in `FuzzAccounts`, use
    /// `fuzz_accounts.account_name.get_or_create_account()`.
    /// If no signers are required, leave the vector empty.
    /// For AccountMetas use <program>::accounts::<corresponding_metas>
    /// For more details, see: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#get-accounts
    fn get_accounts(
        &self,
        client: &mut impl FuzzClient,
        fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<(Vec<Keypair>, Vec<AccountMeta>), FuzzingError> {
        let signers = vec![todo!()];
        let acc_meta = todo!();
        Ok((signers, acc_meta))
    }
}

///IxOps implementation for `UpdateInitialize` with all required functions.
impl IxOps for UpdateInitialize {
    type IxData = pda_vesting::instruction::UpdateInitialize;
    type IxAccounts = FuzzAccounts;
    /// Definition of the program ID that the Instruction is associated with.
    fn get_program_id(&self) -> solana_sdk::pubkey::Pubkey {
        pda_vesting::ID
    }
    /// Definition of the Instruction data.
    /// Use randomly generated data from the fuzzer using `self.data.arg_name`
    /// or customize the data as needed.
    /// For more details, visit: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#get-data
    fn get_data(
        &self,
        _client: &mut impl FuzzClient,
        _fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<Self::IxData, FuzzingError> {
        let data = pda_vesting::instruction::UpdateInitialize {
            btb: todo!(),
            usdt: todo!(),
            usdc: todo!(),
            paypal_usd: todo!(),
            owner_token_receive_wallet: todo!(),
            btb_price: self.data.btb_price,
            vesting_price: self.data.vesting_price,
        };
        Ok(data)
    }
    /// Definition of of the accounts required by the Instruction.
    /// To utilize accounts stored in `FuzzAccounts`, use
    /// `fuzz_accounts.account_name.get_or_create_account()`.
    /// If no signers are required, leave the vector empty.
    /// For AccountMetas use <program>::accounts::<corresponding_metas>
    /// For more details, see: https://ackee.xyz/trident/docs/latest/features/fuzz-instructions/#get-accounts
    fn get_accounts(
        &self,
        client: &mut impl FuzzClient,
        fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<(Vec<Keypair>, Vec<AccountMeta>), FuzzingError> {
        let signers = vec![todo!()];
        let acc_meta = todo!();
        Ok((signers, acc_meta))
    }
}

/*
/// Check supported AccountsStorages at
/// https://ackee.xyz/trident/docs/latest/features/account-storages/
#[derive(Default)]
pub struct FuzzAccounts {
    associated_token_program: AccountsStorage<todo!()>,
    btb_mint_account: AccountsStorage<todo!()>,
    btb_sale_account: AccountsStorage<PdaStore>,
    btb_sale_token_account: AccountsStorage<PdaStore>,
    owner_btb_account: AccountsStorage<todo!()>,
    owner_token_account: AccountsStorage<todo!()>,
    signer: AccountsStorage<todo!()>,
    system_program: AccountsStorage<todo!()>,
    token_program: AccountsStorage<todo!()>,
    user: AccountsStorage<todo!()>,
    user_btb_account: AccountsStorage<PdaStore>,
    user_token_account: AccountsStorage<todo!()>,
}
*/

#[derive(Default)]
pub struct FuzzAccounts {
    associated_token_program: AccountsStorage<solana_sdk::pubkey::Pubkey>, // Public key for the associated token program
    btb_mint_account: AccountsStorage<PdaStore>, // PDA for BTB mint
    btb_sale_account: AccountsStorage<PdaStore>, // PDA for the BTB sale account
    btb_sale_token_account: AccountsStorage<PdaStore>, // PDA for BTB sale token
    owner_btb_account: AccountsStorage<KeypairStore>, // Owner BTB account is a signer
    owner_token_account: AccountsStorage<KeypairStore>, // Owner token account is a signer
    signer: AccountsStorage<KeypairStore>, // Signer is a KeypairStore
    system_program: AccountsStorage<solana_sdk::pubkey::Pubkey>, // System program public key
    token_program: AccountsStorage<solana_sdk::pubkey::Pubkey>, // Token program public key
    user: AccountsStorage<KeypairStore>, // User is a signer
    user_btb_account: AccountsStorage<PdaStore>, // User BTB account is PDA
    user_token_account: AccountsStorage<KeypairStore>, // User token account is a signer
}
