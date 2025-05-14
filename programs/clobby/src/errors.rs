use anchor_lang::error_code;

#[error_code]
pub enum ClobbyProgramError {
    #[msg("Insufficient funds to place the order")]
    InSufficientBalance,

    #[msg("Order has been filled Partially")]
    OrderFilledPartially,

    #[msg("Market Events has reached the limit")]
    EventsMaxLimit,

    #[msg("Bookside can be either 0 or 1")]
    InvalidBookSide,

    #[msg("Order Id Not found")]
    InvalidOrderId,

    #[msg("Invalid Event type, it can be either 0 or 1")]
    InvalidEventType

}