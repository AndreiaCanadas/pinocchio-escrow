// COMMENTED OUT FOR TESTING WITH LITESVM
// #![no_std]
// use pinocchio::nostd_panic_handler;
// nostd_panic_handler!();

use pinocchio::address::declare_id;
use pinocchio::error::ProgramError;
use pinocchio::{
  AccountView,
  Address,
  entrypoint,
  ProgramResult,
};
use solana_program_log::log;

mod state;
mod instructions;
use instructions::{make, take, refund};

use crate::instructions::EscrowInstructions;

declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");

entrypoint!(process_instruction);

pub fn process_instruction(
  program_id: &Address,
  accounts: &[AccountView],
  instruction_data: &[u8],
) -> ProgramResult {

  log!("Hello from my escrow pinocchio program!");

  let (discriminator, data) = instruction_data.split_first().ok_or(ProgramError::InvalidInstructionData)?;
  
  match EscrowInstructions::try_from(discriminator)? {
    EscrowInstructions::MAKE => make(accounts, data)?,
    EscrowInstructions::TAKE => take(accounts, data)?,
    EscrowInstructions::REFUND => refund(accounts, data)?,
  }

  Ok(())
}

#[cfg(test)]
mod tests;