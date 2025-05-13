use anchor_lang::prelude::*;

declare_id!("B7odahygLXdwCYmJteVyBFXXe9qEW5hyvCXieRGBoTTz");

#[program]
pub mod solana_mixer {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
