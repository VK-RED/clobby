use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Market{
    #[max_len(15)]
    pub name: String,
    pub market_authority: Pubkey, // PDA THAT HAS THE AUTHORITY TO SEND THE TOKENS TO THE USER
    pub base_token: Pubkey, 
    pub quote_token: Pubkey,
    pub min_base_amount: u128, // Minimum base token needed to place a trade in LAMPORTS
    pub min_quote_amount: u128, // Minimum quote assest amount needed to place a trade in LAMOPRTS
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub base_token_vault: Pubkey,
    pub quote_token_vault: Pubkey,
}