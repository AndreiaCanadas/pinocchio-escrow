use solana_instruction::{AccountMeta, Instruction};
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_associated_token_account_interface::address::get_associated_token_address;
use solana_pubkey::{Pubkey, pubkey};
use solana_program::msg;
use litesvm_token::spl_token::ID as TOKEN_PROGRAM_ID;
use spl_associated_token_account_interface::program::ID as ASSOCIATED_TOKEN_PROGRAM_ID;

use crate::tests::test_helpers::setup_escrow_test;

const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");

#[test]
fn test_make() {
    let mut escrow_setup = setup_escrow_test();

    let seed: u8 = 1;
    let amount_a: u64 = 70_000_000;
    let amount_b: u64 = 50_000_000;

    // Derive the escrow PDA
    let maker_pubkey = escrow_setup.maker.pubkey();
    let escrow_seeds: &[&[u8]] = &[b"escrow", maker_pubkey.as_ref(), &[seed]];
    let (escrow_pda, escrow_bump) = Pubkey::find_program_address(escrow_seeds, &escrow_setup.program_id);
    msg!("Escrow PDA: {}", escrow_pda);

    // Derive the vault PDA
    let vault = get_associated_token_address(
        &escrow_pda,
        &escrow_setup.mint_a,
    );
    msg!("Escrow Vault: {}", vault);

    // Create the make instruction
    let make_data = [
        vec![0u8],  // discriminator
        amount_a.to_le_bytes().to_vec(),
        amount_b.to_le_bytes().to_vec(),
        vec![seed],
        vec![escrow_bump],
    ].concat();
    let make_accounts = vec![
        AccountMeta::new(escrow_setup.maker.pubkey(), true),
        AccountMeta::new(escrow_setup.mint_a, false),
        AccountMeta::new(escrow_setup.mint_b, false),
        AccountMeta::new(escrow_setup.maker_ata_a, false),
        AccountMeta::new(vault, false),
        AccountMeta::new(escrow_pda, false),
        AccountMeta::new(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new(TOKEN_PROGRAM_ID, false),
        AccountMeta::new(ASSOCIATED_TOKEN_PROGRAM_ID, false),
    ];
    let make_instruction = Instruction {
        program_id: escrow_setup.program_id,
        accounts: make_accounts,
        data: make_data,
    };

    // Create and send the transaction
    let message = Message::new(&[make_instruction], Some(&escrow_setup.maker.pubkey()));
    let recent_blockhash = escrow_setup.litesvm.latest_blockhash();
    let transaction = Transaction::new(
        &[&escrow_setup.maker],
        message,
        recent_blockhash
    );
    let tx = escrow_setup.litesvm.send_transaction(transaction).unwrap();

    // Log transaction details
    msg!("\n\nMake escrow transaction sucessfull");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);

}

#[test]
fn test_take() {
    let mut escrow_setup = setup_escrow_test();

    let seed: u8 = 123;
    let amount_a: u64 = 30_000_000;
    let amount_b: u64 = 70_000_000;

    // Derive the escrow PDA
    let maker_pubkey = escrow_setup.maker.pubkey();
    let escrow_seeds: &[&[u8]] = &[b"escrow", maker_pubkey.as_ref(), &[seed]];
    let (escrow_pda, escrow_bump) = Pubkey::find_program_address(escrow_seeds, &escrow_setup.program_id);
    msg!("Escrow PDA: {}", escrow_pda);

    // Derive the vault PDA
    let vault = get_associated_token_address(
        &escrow_pda,
        &escrow_setup.mint_a,
    );
    msg!("Escrow Vault: {}", vault);

    // Create the make instruction
    let make_data = [
        vec![0u8],  // discriminator
        amount_a.to_le_bytes().to_vec(),
        amount_b.to_le_bytes().to_vec(),
        vec![seed],
        vec![escrow_bump],
    ].concat();
    let make_accounts = vec![
        AccountMeta::new(escrow_setup.maker.pubkey(), true),
        AccountMeta::new(escrow_setup.mint_a, false),
        AccountMeta::new(escrow_setup.mint_b, false),
        AccountMeta::new(escrow_setup.maker_ata_a, false),
        AccountMeta::new(vault, false),
        AccountMeta::new(escrow_pda, false),
        AccountMeta::new(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new(TOKEN_PROGRAM_ID, false),
        AccountMeta::new(ASSOCIATED_TOKEN_PROGRAM_ID, false),
    ];
    let make_instruction = Instruction {
        program_id: escrow_setup.program_id,
        accounts: make_accounts,
        data: make_data,
    };

    // Create and send the transaction
    let message = Message::new(&[make_instruction], Some(&escrow_setup.maker.pubkey()));
    let recent_blockhash = escrow_setup.litesvm.latest_blockhash();
    let transaction = Transaction::new(
        &[&escrow_setup.maker],
        message,
        recent_blockhash
    );
    let _tx = escrow_setup.litesvm.send_transaction(transaction).unwrap();

    // Create the take instruction
    let take_data = [
        vec![1u8],  // discriminator
    ].concat();
    let take_accounts = vec![
        AccountMeta::new(escrow_setup.taker.pubkey(), true),
        AccountMeta::new(escrow_setup.maker.pubkey(), false),
        AccountMeta::new(escrow_setup.mint_a, false),
        AccountMeta::new(escrow_setup.mint_b, false),
        AccountMeta::new(escrow_setup.taker_ata_a, false),
        AccountMeta::new(escrow_setup.taker_ata_b, false),
        AccountMeta::new(vault, false),
        AccountMeta::new(escrow_setup.maker_ata_b, false),
        AccountMeta::new(escrow_pda, false),
        AccountMeta::new(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new(TOKEN_PROGRAM_ID, false),
    ];
    let take_instruction = Instruction {
        program_id: escrow_setup.program_id,
        accounts: take_accounts,
        data: take_data,
    };

    // Create and send the transaction
    let message = Message::new(&[take_instruction], Some(&escrow_setup.taker.pubkey()));
    let recent_blockhash = escrow_setup.litesvm.latest_blockhash();
    let transaction = Transaction::new(
        &[&escrow_setup.taker],
        message,
        recent_blockhash
    );
    let tx = escrow_setup.litesvm.send_transaction(transaction).unwrap();

    // Log transaction details
    msg!("\n\nTake escrow transaction sucessfull");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);

}

#[test]
fn test_refund() {
    let mut escrow_setup = setup_escrow_test();

    let seed: u8 = 255;
    let amount_a: u64 = 70_000_000;
    let amount_b: u64 = 30_000_000;

    // Derive the escrow PDA
    let maker_pubkey = escrow_setup.maker.pubkey();
    let escrow_seeds: &[&[u8]] = &[b"escrow", maker_pubkey.as_ref(), &[seed]];
    let (escrow_pda, escrow_bump) = Pubkey::find_program_address(escrow_seeds, &escrow_setup.program_id);
    msg!("Escrow PDA: {}", escrow_pda);

    // Derive the vault PDA
    let vault = get_associated_token_address(
        &escrow_pda,
        &escrow_setup.mint_a,
    );
    msg!("Escrow Vault: {}", vault);

    // Create the make instruction
    let make_data = [
        vec![0u8],  // discriminator
        amount_a.to_le_bytes().to_vec(),
        amount_b.to_le_bytes().to_vec(),
        vec![seed],
        vec![escrow_bump],
    ].concat();
    let make_accounts = vec![
        AccountMeta::new(escrow_setup.maker.pubkey(), true),
        AccountMeta::new(escrow_setup.mint_a, false),
        AccountMeta::new(escrow_setup.mint_b, false),
        AccountMeta::new(escrow_setup.maker_ata_a, false),
        AccountMeta::new(vault, false),
        AccountMeta::new(escrow_pda, false),
        AccountMeta::new(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new(TOKEN_PROGRAM_ID, false),
        AccountMeta::new(ASSOCIATED_TOKEN_PROGRAM_ID, false),
    ];
    let make_instruction = Instruction {
        program_id: escrow_setup.program_id,
        accounts: make_accounts,
        data: make_data,
    };

    // Create and send the transaction
    let message = Message::new(&[make_instruction], Some(&escrow_setup.maker.pubkey()));
    let recent_blockhash = escrow_setup.litesvm.latest_blockhash();
    let transaction = Transaction::new(
        &[&escrow_setup.maker],
        message,
        recent_blockhash
    );
    let _tx = escrow_setup.litesvm.send_transaction(transaction).unwrap();

    // Create the refund instruction
    let refund_data = [
        vec![2u8],  // discriminator
    ].concat();
    let refund_accounts = vec![
        AccountMeta::new(escrow_setup.maker.pubkey(), true),
        AccountMeta::new(escrow_setup.mint_a, false),
        AccountMeta::new(escrow_setup.mint_b, false),
        AccountMeta::new(escrow_setup.maker_ata_a, false),
        AccountMeta::new(vault, false),
        AccountMeta::new(escrow_pda, false),
        AccountMeta::new(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new(TOKEN_PROGRAM_ID, false),
    ];
    let refund_instruction = Instruction {
        program_id: escrow_setup.program_id,
        accounts: refund_accounts,
        data: refund_data,
    };

    // Create and send the transaction
    let message = Message::new(&[refund_instruction], Some(&escrow_setup.maker.pubkey()));
    let recent_blockhash = escrow_setup.litesvm.latest_blockhash();
    let transaction = Transaction::new(
        &[&escrow_setup.maker],
        message,
        recent_blockhash
    );
    let tx = escrow_setup.litesvm.send_transaction(transaction).unwrap();

    // Log transaction details
    msg!("\n\nRefund escrow transaction sucessfull");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);

}