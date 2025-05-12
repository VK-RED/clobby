use anchor_lang::prelude::*;

use crate::state::{Market, UserBalance};

pub fn create_user_balance_account(ctx:Context<CreateUserBalanceAccount>) -> Result<()> {

    let accounts = ctx.accounts;

    let balance_account = &mut accounts.user_onchain_balance;

    balance_account.user = accounts.user.key();
    balance_account.market = accounts.market.key();
    balance_account.base_token = accounts.market.base_token;
    balance_account.quote_token = accounts.market.quote_token;
    balance_account.base_amount = 0;
    balance_account.quote_amount = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct CreateUserBalanceAccount<'info>{
    
    #[account(
        mut, 
        signer,
    )]
    pub user: Signer<'info>,

    pub market: Account<'info, Market>,
    
    #[account(
        init,
        space = 8 + UserBalance::INIT_SPACE,
        payer = user,
        seeds = [b"balance", user.key().as_ref()],
        bump
    )]

    pub user_onchain_balance: Account<'info, UserBalance>,

    pub system_program: Program<'info, System>,

}