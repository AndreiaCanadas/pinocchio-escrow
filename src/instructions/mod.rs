pub mod make;
pub mod take;
pub mod refund;
pub use make::*;
pub use take::*;
pub use refund::*;

use pinocchio::error::ProgramError;

// Create an enum for the instructions
pub enum EscrowInstructions {
    MAKE = 0,
    TAKE = 1,
    REFUND = 2,
}

// Implement the TryFrom trait for the enum
impl TryFrom<&u8> for EscrowInstructions {
    type Error = ProgramError;

    fn try_from(discriminator: &u8) -> Result<Self, Self::Error> {
        match discriminator {
            0 => Ok(EscrowInstructions::MAKE),
            1 => Ok(EscrowInstructions::TAKE),
            2 => Ok(EscrowInstructions::REFUND),
            _ => Err(ProgramError::InvalidInstructionData)
        }
    }
}