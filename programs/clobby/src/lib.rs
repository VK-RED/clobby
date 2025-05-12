use anchor_lang::prelude::*;
pub mod state;
pub mod instructions;
mod errors;

use instructions::*;

declare_id!("4XVYzTX6F9wXfgYK9CnEKkLpQh3ptuthKkCixEBWHXa6");

#[program]
pub mod clobby {
    use super::*;

    /// this is also a separate instruction, due to the reason mention below
    pub fn init_market_authority_and_event(ctx:Context<InitMarketAuthorityAndEvent>) -> Result<()> {
        instructions::init_market_authority_and_event(ctx)?;
        Ok(())
    }

    pub fn create_market(ctx: Context<CreateMarket>, args: CreateMarketArgs) -> Result<()> {
        instructions::create_market(ctx, args)?;
        Ok(())
    }

    /// as the runtime limits the amount of data that can be loaded in the stack
    /// we create this separate instruction, separate from the create_market
    pub fn create_bookside_accounts(ctx: Context<CreateBookSide>) -> Result<()> {
        instructions::create_book_side(ctx)?;
        Ok(())
    }

    // pub fn create_market_events_account() -> Result<()> {
    //     Ok(())
    // }

    /// This is specially useful when matching the orders, we can directly increase or decrease the tokens
    /// we can settle the final amount, when the user requests for it.
    pub fn create_user_balance_account(ctx:Context<CreateUserBalanceAccount>) -> Result<()> {
        instructions::create_user_balance_account(ctx)?;
        Ok(())
    }

    pub fn settle_user_balance(ctx:Context<SettleUserBalance>) -> Result<()> {
        instructions::settle_user_balance(ctx)?;
        Ok(())
    }

    
    
}