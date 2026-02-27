pub mod make;
pub mod take;
pub mod refund;
pub use make::*;
pub use take::*;
pub use refund::*;

use shank::ShankInstruction;

use pinocchio::error::ProgramError;

// Create an enum for the instructions
#[derive(ShankInstruction)]
#[rustfmt::skip]
pub enum EscrowInstructions {
    #[account(0, writable, signer, name="maker", desc="The user that creates the escrow")]
    #[account(1, name="mint_a", desc="The mint that the maker gives in exchange")]
    #[account(2, name="mint_b", desc="The mint that the maker wants to receive")]
    #[account(3, writable, name="maker_ata", desc="The maker ATA of the `mint_a`")]
    #[account(4, writable, name="vault", desc="The ATA owned by the escrow program to hold the maker `mint_a` until the exchange completes")]
    #[account(5, writable, name="escrow", desc="The escrow state account that will be created (PDA derived from seeds and maker pubkey)")]
    #[account(6, name="system_program", desc="The system program for account creation")]
    #[account(7, name="token_program", desc="The token program for token managing")]
    #[account(8, name="associated_token_program", desc="The associated token program for ATA creation")]
    MAKE = 0,
    
    #[account(0, writable, signer, name="taker", desc="The taker that takes the escrow")]
    #[account(1, name="maker", desc="The maker that created the escrow")]
    #[account(2, name="mint_a", desc="The mint that the taker will get from the maker")]
    #[account(3, name="mint_b", desc="The mint that the taker will give to the maker")]
    #[account(4, writable, name="taker_ata_a", desc="The taker ATA of the mint_a")]
    #[account(5, writable, name="taker_ata_b", desc="The taker ATA of the mint_b")]
    #[account(6, writable, name="vault", desc="The ATA owned by the escrow program that is holding the `mint_a`")]
    #[account(7, writable, name="maker_ata_b", desc="The maker ATA of the `mint_b` to receive from the taker")]
    #[account(8, writable, name="escrow", desc="The escrow state account")]
    #[account(9, name="system_program", desc="The system program for account creation")]
    #[account(10, name="token_program", desc="The token program for token managing")]
    TAKE = 1,
    
    #[account(0, writable, signer, name="maker", desc="The maker that created the escrow")]
    #[account(1, name="mint_a", desc="The mint that the taker will get from the maker")]
    #[account(2, name="mint_b", desc="The mint that the taker will give to the maker")]
    #[account(3, writable, name="maker_ata", desc="The maker ATA of the `mint_a`")]
    #[account(4, writable, name="vault", desc="The ATA owned by the escrow program that is holding the `mint_a`")]
    #[account(5, writable, name="escrow", desc="The escrow state account")]
    #[account(6, name="system_program", desc="The system program for account creation")]
    #[account(7, name="token_program", desc="The token program for token managing")]
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