

use anchor_lang::prelude::*;

use crate::state::Market;


pub fn init_market_authority(_ctx:Context<InitMarketAuthority>) -> Result<()> {
    msg!("Initializing market authority");
    Ok(())
}




#[derive(Accounts)]
pub struct InitMarketAuthority<'info>{
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
    ///CHECK: THIS IS PDA OF THE MARKET, THAT CAN SEND TOKENS
    pub market_authority: UncheckedAccount<'info>,

    pub market: Account<'info, Market>,

    pub system_program: Program<'info, System>,
}