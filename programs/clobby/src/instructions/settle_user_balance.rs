use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::state::{Market, ResetSide, UserBalance};


pub fn settle_user_balance(ctx:Context<SettleUserBalance>) -> Result<()>{

    let accounts = ctx.accounts;

    let balance_account = &mut accounts.user_balance_account;

    let settle_base_token = if balance_account.base_amount > 0 {true} else {false};
    let settle_quote_token = if balance_account.quote_amount > 0 {true} else {false};

    let market_key = accounts.market.key();

    let signer_seeds:&[&[&[u8]]] = &[&[b"market", market_key.as_ref(), &[accounts.market.market_authority_bump]]];
        
    let cpi_program = accounts.token_progam.to_account_info();
    
    if settle_base_token {

        let base_cpi_accounts = TransferChecked {
            mint: accounts.base_token.to_account_info(),
            from: accounts.base_vault_account.to_account_info(),
            to: accounts.user_base_token_account.to_account_info(),
            authority: accounts.market_authority.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program.clone(), base_cpi_accounts).with_signer(signer_seeds);

        token_interface::transfer_checked(cpi_context, balance_account.base_amount, accounts.base_token.decimals)?;

        balance_account.reset_balance(ResetSide::Quote);
    }
    else{
        msg!("No base token to settle");
    }

    if settle_quote_token {
        let quote_cpi_accounts = TransferChecked {
            mint: accounts.quote_token.to_account_info(),
            from: accounts.quote_vault_account.to_account_info(),
            to: accounts.user_quote_token_account.to_account_info(),
            authority: accounts.market_authority.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, quote_cpi_accounts).with_signer(signer_seeds);

        token_interface::transfer_checked(cpi_context, balance_account.quote_amount, accounts.quote_token.decimals)?;

        balance_account.reset_balance(ResetSide::Quote);
    }
    else{
        msg!("No quote token to settle");
    }
    
    Ok(())
}

#[derive(Accounts)]
pub struct SettleUserBalance<'info>{

    #[account(
        mut,
        signer,
    )]
    pub user: Signer<'info>,

    #[account(
        has_one = base_token,
        has_one = quote_token,
        has_one = market_authority,
    )]
    pub market: Account<'info, Market>,

    /// CHECK: PDA of the market account, that can
    /// transfer tokens,
    #[account(
        mut,
        seeds=[b"market", market.key().as_ref()],
        bump=market.market_authority_bump,
    )]
    pub market_authority: UncheckedAccount<'info>,

    pub base_token: InterfaceAccount<'info, Mint>,
    pub quote_token: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        has_one = user,
        constraint = user_balance_account.market.key() == market.key()
    )]
    pub user_balance_account: Account<'info, UserBalance>,

    #[account(
        token::mint = base_token.key(),
        token::authority = user.key(),
    )]
    pub user_base_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        token::mint = quote_token.key(),
        token::authority = user.key(),
    )]
    pub user_quote_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        associated_token::mint = base_token.key(),
        associated_token::authority = market.market_authority.key(),
    )]
    pub base_vault_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        associated_token::mint = quote_token.key(),
        associated_token::authority = market.market_authority.key(),
    )]

    pub quote_vault_account: InterfaceAccount<'info, TokenAccount>,
    pub token_progam: Interface<'info, TokenInterface>,

}