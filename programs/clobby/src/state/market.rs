use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Market{
    pub market_authority: Pubkey, // PDA THAT HAS THE AUTHORITY TO SEND THE TOKENS TO THE USER
    pub market_authority_bump: u8,
    pub market_events: Pubkey, // PDA THAT STORES ALL THE FILL AND OUT EVENTS, 
    pub market_event_bump: u8,
    pub base_token: Pubkey, 
    pub quote_token: Pubkey,
    /// Should be in the power of 10's, 
    /// if 1 base_lot_size = 1000, then buying 10 base lots, will be equal to
    /// 1_000_000 native base tokens in LAMPORTS    
    pub base_lot_size: u64, 
    pub total_orders: u64,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub base_token_vault: Pubkey,
    pub quote_token_vault: Pubkey,
    #[max_len(15)]
    pub name: String,  // always better to use at last
}