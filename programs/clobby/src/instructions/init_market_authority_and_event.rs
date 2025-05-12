use anchor_lang::prelude::*;

use crate::state::{Market, MarketEvents};


pub fn init_market_authority_and_event(ctx:Context<InitMarketAuthorityAndEvent>) -> Result<()> {
    let accounts = ctx.accounts;

    let mut market_event = accounts.market_event.load_init()?;

    market_event.market = accounts.market.key();
    market_event.events_to_process = 0;
    

    msg!("Initializing market authority and market event");
    Ok(())
}


#[derive(Accounts)]
pub struct InitMarketAuthorityAndEvent<'info>{
    #[account(
        mut,
        signer,
    )]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 130,
        seeds = [b"market", market.key().as_ref()],
        bump
    )]
    /// CHECK: THIS IS PDA OF THE MARKET, THAT CAN SEND TOKENS
    pub market_authority: UncheckedAccount<'info>,

    #[account(
        zero
    )]
    pub market_event: AccountLoader<'info, MarketEvents>,

    pub market: Account<'info, Market>,

    pub system_program: Program<'info, System>,
}