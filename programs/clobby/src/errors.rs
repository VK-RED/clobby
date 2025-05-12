use anchor_lang::error_code;

#[error_code]
pub enum ClobbyProgramError {
    #[msg("Minimum Order account cannot be 0")]
    InvalidMinOrderAmount,

    #[msg("Insufficient funds to place the order")]
    InSufficientBalance,

    #[msg("Order has been filled Partially")]
    OrderFilledPartially,

    #[msg("Market Events has reached the limit")]
    EventsMaxLimit,

    #[msg("Bookside can be either 0 or 1")]
    InvalidBookSide,

}