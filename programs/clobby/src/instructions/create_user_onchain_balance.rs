use anchor_lang::prelude::*;

use crate::state::{Market, UserOnchainBalance};

pub fn create_user_onchain_balance(ctx:Context<CreateUserOnchainBalance>) -> Result<()> {

    let accounts = ctx.accounts;

    let onchain_balance_account = &mut accounts.user_onchain_balance;

    onchain_balance_account.user = accounts.signer.key();
    onchain_balance_account.market = accounts.market.key();
    onchain_balance_account.base_token = accounts.market.base_token;
    onchain_balance_account.quote_token = accounts.market.quote_token;
    onchain_balance_account.base_onchain_amount = 0;
    onchain_balance_account.quote_onchain_amount = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct CreateUserOnchainBalance<'info>{
    
    #[account(
        mut, 
        signer,
    )]
    pub signer: Signer<'info>,

    pub market: Account<'info, Market>,
    
    #[account(
        init,
        space = 8 + UserOnchainBalance::INIT_SPACE,
        payer = signer,
    )]
    pub user_onchain_balance: Account<'info, UserOnchainBalance>,

    pub system_program: Program<'info, System>,

}