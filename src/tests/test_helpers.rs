use litesvm::LiteSVM;
use litesvm_token::{
    CreateAssociatedTokenAccount, CreateMint, MintTo
};
use solana_keypair::Keypair;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_pubkey::{Pubkey};
use solana_signer::Signer;
use solana_program::msg;

use std::path::PathBuf;

pub fn get_program_id() -> Pubkey {
    Pubkey::from(crate::ID)
}

/// # Escrow Test Setup
/// 
/// This struct is used to create a test setup for the escrow program.
/// It contains the necessary accounts and data for the test.
pub struct EscrowTestSetup {
    /// The LitesVM instance for simulating Solana transactions
    pub litesvm: LiteSVM,
    /// The program ID for the escrow program
    pub program_id: Pubkey,
    /// The mint authority for creating tokens
    pub _mint_authority: Keypair,
    /// The mint for the first token
    pub mint_a: Pubkey,
    /// The mint for the second token
    pub mint_b: Pubkey,
    /// The maker Keypair
    pub maker: Keypair,
    /// The taker Keypair
    pub taker: Keypair,
    /// The maker ATA for the mint_a
    pub maker_ata_a: Pubkey,
    /// The maker ATA for the mint_b
    pub maker_ata_b: Pubkey,
    /// The taker ATA for the mint_a
    pub taker_ata_a: Pubkey,
    /// The taker ATA for the mint_b
    pub taker_ata_b: Pubkey,
}

pub fn setup_escrow_test() -> EscrowTestSetup {

    // Create a new LitesVM instance
    let mut litesvm = LiteSVM::new();
    let program_id = get_program_id();

    // Load the program .so
    let so_path = PathBuf::from("/Users/andreiacanadas/Documents/Solana/Github/pinocchio-escrow/target/sbpf-solana-solana/release/pinocchio_escrow.so");
    let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
    litesvm.add_program(program_id, &program_data).expect("Failed to add program");

    // Create and fund the mint authority
    let mint_authority = Keypair::new();
    litesvm.airdrop(&mint_authority.pubkey(), 10 * LAMPORTS_PER_SOL).expect("Failed to airdrop");

    // Create the mints
    let mint_a = CreateMint::new(&mut litesvm, &mint_authority)
        .authority(&mint_authority.pubkey())
        .decimals(9)
        .send()
        .unwrap();
    let mint_b = CreateMint::new(&mut litesvm, &mint_authority)
        .authority(&mint_authority.pubkey())
        .decimals(9)
        .send()
        .unwrap();
    msg!("Mint A created: {}", mint_a);
    msg!("Mint B created: {}", mint_b);

    // Create and fund the maker and taker accounts
    let maker = Keypair::new();
    let taker = Keypair::new();
    litesvm.airdrop(&maker.pubkey(), 10 * LAMPORTS_PER_SOL).expect("Failed to airdrop");
    litesvm.airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL).expect("Failed to airdrop");

    // Create the ATA accounts
    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut litesvm, &maker, &mint_a)
        .owner(&maker.pubkey())
        .send()
        .unwrap();
    let maker_ata_b = CreateAssociatedTokenAccount::new(&mut litesvm, &maker, &mint_b)
        .owner(&maker.pubkey())
        .send()
        .unwrap();
    let taker_ata_a = CreateAssociatedTokenAccount::new(&mut litesvm, &taker, &mint_a)
        .owner(&taker.pubkey())
        .send()
        .unwrap();
    let taker_ata_b = CreateAssociatedTokenAccount::new(&mut litesvm, &taker, &mint_b)
        .owner(&taker.pubkey())
        .send()
        .unwrap();
    msg!("Maker ATA Mint A created: {}", maker_ata_a);
    msg!("Maker ATA Mint B created: {}", maker_ata_b);
    msg!("Taker ATA Mint A created: {}", taker_ata_a);
    msg!("Taker ATA Mint B created: {}", taker_ata_b);

    // Mint tokens to the maker and taker
    MintTo::new(&mut litesvm, &mint_authority, &mint_a, &maker_ata_a, 100_000_000)
        .send()
        .unwrap();
    MintTo::new(&mut litesvm, &mint_authority, &mint_b, &taker_ata_b, 100_000_000)
        .send()
        .unwrap();
    msg!("Minted 100 tokens of Mint A to Maker");
    msg!("Minted 100 tokens of Mint B to Taker");

    EscrowTestSetup {
        litesvm,
        program_id,
        _mint_authority: mint_authority,
        mint_a,
        mint_b,
        maker,
        taker,
        maker_ata_a,
        maker_ata_b,
        taker_ata_a,
        taker_ata_b,
    }
}

