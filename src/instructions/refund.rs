use pinocchio::{
    AccountView, ProgramResult, cpi::{Seed, Signer}, error::ProgramError,

};
use pinocchio_token::{instructions::{CloseAccount, TransferChecked}, state::{Mint, TokenAccount}};
use solana_program_log::log;

use crate::state::Escrow;

/// # Refund Instruction
/// 
/// This function allows the maker to cancel the escrow deal he created
/// 
/// ## Business Logic:
/// 1.
/// 
/// ## Accounts Expected:
/// 0. [signer] maker - The maker that created the escrow
/// 1. [] mint_a - The mint that the taker will get from the maker
/// 2. [] mint_b - The mint that the taker will give to the maker
/// 3. [writable] maker_ata - The maker ATA of the `mint_a`
/// 4. [writable] vault - The ATA owned by the escrow program that is holding the `mint_a`
/// 5. [writable] escrow - The escrow state account
/// 6. [] system_program - The system program for account creation
/// 7. [] token_program - The token program for token managing
/// 
pub fn refund (accounts: &[AccountView], _instruction_data: &[u8]) -> ProgramResult {

    // Unpack accounts - Validate expected accounts
    let [maker, mint_a, mint_b, maker_ata, vault, escrow, _system_program, _token_program, _remaining @..] = accounts else {
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

    // Validate the ATAs are owned by the token program
    if !maker_ata.owned_by(&pinocchio_token::ID) || !vault.owned_by(&pinocchio_token::ID) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Validate the maker ATA mint and authority
    if TokenAccount::from_account_view(maker_ata)?.owner() != maker.address() {
        return Err(ProgramError::InvalidAccountData);
    }
    if TokenAccount::from_account_view(maker_ata)?.mint() != mint_a.address() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Validate the vault mint and authority
    if TokenAccount::from_account_view(vault)?.owner() != escrow.address() {
        return Err(ProgramError::InvalidAccountData);
    }
    if TokenAccount::from_account_view(vault)?.mint() != mint_a.address() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Validate escrow PDA (derive expected PDA and verify it matches provided address)
    let escrow_account = Escrow::from_account_info_mut(escrow)?;
    let escrow_seeds = [(b"escrow"), maker.address().as_ref(), escrow_account.seed.as_slice(), escrow_account.bump.as_slice()];
    let escrow_pda = pinocchio_pubkey::derive_address_const(&escrow_seeds, None, &crate::ID.as_array());
    if escrow_pda != escrow.address().to_bytes() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Validate the mint_b is the same as the one in the escrow
    if mint_b.address().to_bytes() != escrow_account.mint_b {
        return Err(ProgramError::InvalidAccountData);
    }

    // Transfer amount_a from vault back to maker
    let signer_seeds = [Seed::from(b"escrow"), Seed::from(maker.address().as_ref()), Seed::from(escrow_account.seed.as_ref()), Seed::from(escrow_account.bump.as_ref())];
    let signers = Signer::from(&signer_seeds);
    let amount_a = TokenAccount::from_account_view(vault)?.amount();
    TransferChecked {
        from: vault,
        mint: mint_a,
        to: maker_ata,
        authority: escrow,
        amount: amount_a,
        decimals: Mint::from_account_view(mint_a)?.decimals(),
    }.invoke_signed(&[signers.clone()])?;

    log!("debug");

    // Close Vault Account
    CloseAccount {
        account: vault,
        destination: maker,
        authority: escrow,
    }.invoke_signed(&[signers])?;

    // Manually close the escrow account and return rent to the maker
    // This completes the trade by cleaning up all accounts
    maker.set_lamports(maker.lamports() + escrow.lamports());
    escrow.set_lamports(0);
        
    Ok(())
} 