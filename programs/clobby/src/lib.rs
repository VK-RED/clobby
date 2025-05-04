use anchor_lang::prelude::*;

declare_id!("4XVYzTX6F9wXfgYK9CnEKkLpQh3ptuthKkCixEBWHXa6");

#[program]
pub mod clobby {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
