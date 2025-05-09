use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Market{
    pub market_authority: Pubkey, // PDA THAT HAS THE AUTHORITY TO SEND THE TOKENS TO THE USER
    pub market_authority_bump: u8,
    pub market_event: Pubkey, // PDA THAT STORES ALL THE FILL AND OUT EVENTS, 
    pub market_event_bump: u8,
    pub base_token: Pubkey, 
    pub quote_token: Pubkey,
    pub min_base_amount: u64, // Minimum base token needed to place a trade in LAMPORTS
    pub min_quote_amount: u64, // Minimum quote assest amount needed to place a trade in LAMOPRTS
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub base_token_vault: Pubkey,
    pub quote_token_vault: Pubkey,
    #[max_len(15)]
    pub name: String,  // always better to use at last
}