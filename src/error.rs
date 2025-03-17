use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum AIInfraError {
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Not Rent Exempt")]
    NotRentExempt,
    #[error("Insufficient Credits")]
    InsufficientCredits,
}

impl From<AIInfraError> for ProgramError {
    fn from(e: AIInfraError) -> Self {
        ProgramError::Custom(e as u32)
    }
}