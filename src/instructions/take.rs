use pinocchio::{
    AccountView, ProgramResult, cpi::{Seed, Signer}, error::ProgramError,

};
use pinocchio_token::{instructions::{CloseAccount, TransferChecked}, state::{Mint, TokenAccount}};
use solana_program_log::log;

use crate::state::Escrow;

/// # Take Instruction
/// 
/// This function allows a user (taker) to accept the escrow deal created by a maker
/// 
/// ## Business Logic:
/// 1. Validate all accounts and verify the escrow PDA from the seeds stored in the escrow account
/// 2. Verify mint_b matches the one stored in the escrow account
/// 3. Transfer amount_b of mint_b from the taker to the maker
/// 4. Transfer all mint_a from the vault to the taker (signed by the escrow PDA)
/// 5. Close the vault ATA and return rent to the maker
/// 6. Close the escrow account and return rent to the maker
/// 
/// ## Accounts Expected:
/// 0. [signer] taker - The taker that takes the escrow
/// 1. [] maker - The maker that created the escrow
/// 2. [] mint_a - The mint that the taker will get from the maker
/// 3. [] mint_b - The mint that the taker will give to the maker
/// 4. [writable] taker_ata_a - The taker ATA of the mint_a
/// 5. [writable] taker_ata_b - The taker ATA of the mint_b
/// 6. [writable] vault - The ATA owned by the escrow program that is holding the `mint_a`
/// 7. [writable] maker_ata_b - The maker ATA of the `mint_b` to receive from the taker
/// 8. [writable] escrow - The escrow state account
/// 9. [] system_program - The system program for account creation
/// 10. [] token_program - The token program for token managing
/// 
pub fn take (accounts: &[AccountView], _instruction_data: &[u8]) -> ProgramResult {

    // Unpack accounts - Validate expected accounts
    let [taker, maker, mint_a, mint_b, taker_ata_a, taker_ata_b, vault, maker_ata_b, escrow, _system_program, _token_program, _remaining @..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Check if taker is signer
    if !taker.is_signer() {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Check if mint accounts are owned by the token program
    if !mint_a.owned_by(&pinocchio_token::ID) || !mint_b.owned_by(&pinocchio_token::ID) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Validate the ATAs are owned by the token program
    if !taker_ata_a.owned_by(&pinocchio_token::ID) || 
        !taker_ata_b.owned_by(&pinocchio_token::ID) ||
        !vault.owned_by(&pinocchio_token::ID) ||
        !maker_ata_b.owned_by(&pinocchio_token::ID)
    {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Validate the taker ATA mint and authority
    if TokenAccount::from_account_view(taker_ata_a)?.owner() != taker.address() {
        return Err(ProgramError::InvalidAccountData);
    }
    if TokenAccount::from_account_view(taker_ata_a)?.mint() != mint_a.address() {
        return Err(ProgramError::InvalidAccountData);
    }
    if TokenAccount::from_account_view(taker_ata_b)?.owner() != taker.address() {
        return Err(ProgramError::InvalidAccountData);
    }
    if TokenAccount::from_account_view(taker_ata_b)?.mint() != mint_b.address() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Validate the vault mint and authority
    if TokenAccount::from_account_view(vault)?.owner() != escrow.address() {
        return Err(ProgramError::InvalidAccountData);
    }
    if TokenAccount::from_account_view(vault)?.mint() != mint_a.address() {
        return Err(ProgramError::InvalidAccountData);
    }

    // Validate the maker ATA mint and authority
    if TokenAccount::from_account_view(maker_ata_b)?.owner() != maker.address() {
        return Err(ProgramError::InvalidAccountData);
    }
    if TokenAccount::from_account_view(maker_ata_b)?.mint() != mint_b.address() {
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

    // Transfer amount_b from taker to maker
    let amount_b = u64::from_le_bytes(escrow_account.amount_b);
    TransferChecked {
        from: taker_ata_b,
        mint: mint_b,
        to: maker_ata_b,
        authority: taker,
        amount: amount_b,
        decimals: Mint::from_account_view(mint_b)?.decimals(),
    }.invoke()?;

    // Transfer amount_a from vault to taker
    let signer_seeds = [Seed::from(b"escrow"), Seed::from(maker.address().as_ref()), Seed::from(escrow_account.seed.as_ref()), Seed::from(escrow_account.bump.as_ref())];
    let signers = Signer::from(&signer_seeds);
    let amount_a = TokenAccount::from_account_view(vault)?.amount();
    TransferChecked {
        from: vault,
        mint: mint_a,
        to: taker_ata_a,
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