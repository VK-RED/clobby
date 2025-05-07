use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::state::{Market, ResetSide, UserOnchainBalance};


pub fn settle_user_onchain_balance(ctx:Context<SettleUserOnchainBalance>) -> Result<()>{

    let accounts = ctx.accounts;

    let onchain_balance_account = &mut accounts.user_onchain_balance_account;

    let settle_base_token = if onchain_balance_account.base_onchain_amount > 0 {true} else {false};
    let settle_quote_token = if onchain_balance_account.quote_onchain_amount > 0 {true} else {false};

    let market_key = accounts.market.key();

    let signer_seeds:&[&[&[u8]]] = &[&[b"market", market_key.as_ref(), &[accounts.market.market_authority_bump]]];
        
    let cpi_program = accounts.token_progam.to_account_info();
    
    if settle_base_token {

        let base_cpi_accounts = TransferChecked {
            mint: accounts.base_token.to_account_info(),
            from: accounts.base_vault_account.to_account_info(),
            to: accounts.user_base_token_account.to_account_info(),
            authority: accounts.base_vault_account.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program.clone(), base_cpi_accounts).with_signer(signer_seeds);

        token_interface::transfer_checked(cpi_context, onchain_balance_account.base_onchain_amount, accounts.base_token.decimals)?;

        onchain_balance_account.reset_balance(ResetSide::Quote);
    }
    else{
        msg!("No base token to settle");
    }

    if settle_quote_token {
        let quote_cpi_accounts = TransferChecked {
            mint: accounts.quote_token.to_account_info(),
            from: accounts.quote_vault_account.to_account_info(),
            to: accounts.user_quote_token_account.to_account_info(),
            authority: accounts.quote_vault_account.to_account_info(),
        };

        let cpi_context = CpiContext::new(cpi_program, quote_cpi_accounts).with_signer(signer_seeds);

        token_interface::transfer_checked(cpi_context, onchain_balance_account.quote_onchain_amount, accounts.quote_token.decimals)?;

        onchain_balance_account.reset_balance(ResetSide::Quote);
    }
    else{
        msg!("No quote token to settle");
    }
    
    Ok(())
}

#[derive(Accounts)]
pub struct SettleUserOnchainBalance<'info>{

    #[account(
        mut,
        signer,
    )]
    pub user: Signer<'info>,

    #[account(
        has_one = base_token,
        has_one = quote_token,
    )]
    pub market: Account<'info, Market>,

    pub base_token: InterfaceAccount<'info, Mint>,
    pub quote_token: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        has_one = user,
        constraint = user_onchain_balance_account.market.key() == market.key()
    )]
    pub user_onchain_balance_account: Account<'info, UserOnchainBalance>,

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