use anchor_lang::prelude::*;

declare_id!("Ch6gqgXUC7TsVroSGDF2pLfv4Bd4bM3cvpEJXZf3WYyC");

#[program]
pub mod btb_project {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
