use anchor_lang::prelude::*;
pub mod state;
pub mod instructions;
mod errors;
use instructions::*;

declare_id!("4XVYzTX6F9wXfgYK9CnEKkLpQh3ptuthKkCixEBWHXa6");

#[program]
pub mod clobby {
    use super::*;

    pub fn create_market(ctx: Context<CreateMarket>, args: CreateMarketArgs) -> Result<()> {
        instructions::create_market(ctx, args)?;
        Ok(())
    }

    pub fn create_bookside_accounts(ctx: Context<CreateBookSide>) -> Result<()> {
        instructions::create_book_side(ctx)?;
        Ok(())
    }

    
}