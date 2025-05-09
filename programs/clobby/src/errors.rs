use anchor_lang::error_code;

#[error_code]
pub enum ClobbyProgramError {
    #[msg("Minimum Order account cannot be 0")]
    InvalidMinOrderAmount,

    #[msg("Insufficient funds to place the order")]
    InSufficientBalance,
}