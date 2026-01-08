use pinocchio::{
    AccountView, ProgramResult, cpi::{Seed, Signer}, error::ProgramError, sysvars::{Sysvar, rent::Rent}

};
use pinocchio_associated_token_account::instructions::Create;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{instructions::TransferChecked, state::Mint};

use crate::state::Escrow;

/// # Make Instruction
/// 
/// This function allows an user to create an escrow
/// 
/// ## Business Logic:
/// 1.
/// 
/// ## Accounts Expected:
/// 1. [signer] maker - The user that creates the escrow
/// 2. [] mint_a - The mint that the maker gives in exchange
/// 3. [] mint_b - The mint that the maker wants to receive
/// 4. [writable] maker_ata - The maker ATA of the `mint_a`
/// 5. [writable] vault - The ATA owned by the escrow program to hold the maker `mint_a` until the exchange completes
/// 6. [writable] escrow - The escrow state account that will be created (PDA derived from seeds and maker pubkey)
/// 7. [] system_program - The system program for account creation
/// 8. [] token_program - The token program for token managing
/// 9. [] associated_token_program - The associated token program for ATA creation
/// 
/// ## Data Parameters:
/// 1. [u8; 8] amount_a - The amount of mint_a that the maker gives for the exchange (u64)
/// 2. [u8; 8] amount_b - The amount of mint_b that the maker wants to receive in the exchange (u64)
/// 3. [u8; 1] seed - The seed to derive the escrow PDA (u8)
/// 4. [u8; 1] escrow_bump - The bump of the escrow account
pub fn make(accounts: &[AccountView], instruction_data: &[u8]) -> ProgramResult {
    
    // Unpack accounts - Validate expected accounts
    let [maker, mint_a, mint_b, maker_ata, vault, escrow, system_program, token_program, _associated_token_program, _remaining @..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Check if maker is signer
    if !maker.is_signer() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Check if mint accounts are owned by the token program
    if !mint_a.owned_by(&pinocchio_token::ID) || !mint_b.owned_by(&pinocchio_token::ID) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Validate the maker ATA
    // TBD: is this correct and what else is needed?
    if !maker_ata.owned_by(&pinocchio_token::ID) {
        return Err(ProgramError::InvalidAccountOwner)
    }

    // Check if the vault and escrow are not initialized (if are owned by the system program)
    if !escrow.owned_by(&pinocchio_system::ID) || !vault.owned_by(&pinocchio_system::ID) {
        return Err(ProgramError::InvalidAccountOwner);
    }
    
    // Validate data parameters
    if instruction_data.len() != 18 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Unpack data
    let amount_a = u64::from_le_bytes(instruction_data[0..8].try_into().unwrap());
    let amount_b = u64::from_le_bytes(instruction_data[8..16].try_into().unwrap());
    let seed = unsafe { *(instruction_data.as_ptr().add(16) as *const u8)}.to_le_bytes();
    let escrow_bump = unsafe { *(instruction_data.as_ptr().add(17) as *const u8) }.to_le_bytes();

    // Validate if amount values are greater than 0
    if amount_a == 0 || amount_b == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // Validate escrow PDA (derive expected PDA and verify it matches provided address)
    let escrow_seeds = [(b"escrow"), maker.address().as_ref(), seed.as_slice(), escrow_bump.as_slice()];
    let escrow_pda = pinocchio_pubkey::derive_address_const(&escrow_seeds, None, &crate::ID.as_array());
    if escrow_pda != escrow.address().to_bytes() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Create Escrow account
    let signer_seeds = [Seed::from(b"escrow"), Seed::from(maker.address().as_ref()), Seed::from(seed.as_ref()), Seed::from(escrow_bump.as_ref())];
    let signers = Signer::from(&signer_seeds);
    CreateAccount {
        from: maker,
        to: escrow,
        lamports: Rent::get()?.minimum_balance_unchecked(Escrow::LEN),
        space: Escrow::LEN as u64,
        owner: &crate::ID,
    }.invoke_signed(&[signers])?;
    let escrow_account = Escrow::from_account_info_mut(escrow)?;
    escrow_account.set_inner(mint_b.address().to_bytes(), amount_b.to_le_bytes(), seed, escrow_bump);

    // Create Vault account
    Create {
        funding_account: maker,
        account: vault,
        wallet: escrow,
        mint: mint_a,
        system_program: system_program,
        token_program: token_program,
    }.invoke()?;

    // Transfer amount_a to vault
    let decimals = Mint::from_account_view(mint_a)?.decimals();
    TransferChecked {
        from: maker_ata,
        mint: mint_a,
        to: vault,
        authority: maker,
        amount: amount_a,
        decimals,
    }.invoke()?;
    
    Ok(())
}