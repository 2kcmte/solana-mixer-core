use anchor_lang::prelude::*;

declare_id!("AQW933TrdFxE5q7982Vb57crHjZe3B7EZaHotdXnaQYQ");

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
