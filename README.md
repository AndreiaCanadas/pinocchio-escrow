# Pinocchio Escrow
---
This is a Solana smart-contract implementing a trustless token swap (escrow), built with Pinocchio library.

## What is it?
The Pinocchio Escrow is a Solana program that allows two parties to exchange SPL tokens trustlessly. A `maker` initiates the deal by depositing a given amount of `token_a` into a vault and specifying how much `token_b` they want in return. Any `taker` can then fulfil the deal atomically. If no taker steps in, the maker can cancel at any time and reclaim their tokens.

- **Make:** The maker creates an escrow state account (PDA) and deposits `amount_a` of `mint_a` into a vault ATA owned by the escrow PDA. The deal terms (`mint_b`, `amount_b`) are saved in the escrow account.
- **Take:** A taker fulfils the deal by transferring `amount_b` of `mint_b` to the maker and receiving `amount_a` of `mint_a` from the vault. All accounts are closed at the end.
- **Refund:** The maker cancels the open escrow, reclaims `amount_a` from the vault, and closes all accounts.

## How it works?
- The escrow state account is a PDA derived from the static seed `b"escrow"`, the maker's public key, a user-supplied `seed` byte (allowing multiple concurrent escrows per maker), and the PDA bump.
- The vault is an Associated Token Account (ATA) of `mint_a` whose authority is the escrow PDA.
- The escrow state account stores `mint_b`, `amount_b`, `seed`, and `bump` — the minimum data needed to verify and execute the swap.
- When the trade is completed (Take) or cancelled (Refund), the vault ATA is closed via CPI and the escrow account is closed manually, returning rent to the maker.
- All token transfers use `TransferChecked` from the token program for safe, decimal-aware transfers.

## Architecture

### Escrow State Account

```rust
pub struct Escrow {
    pub mint_b:   [u8; 32],  // The mint the maker wants to receive
    pub amount_b: [u8; 8],   // Amount of mint_b expected (u64 LE)
    pub seed:     [u8; 1],   // Seed used to derive this escrow PDA
    pub bump:     [u8; 1],   // Canonical bump of this escrow PDA
}
```

- Size: 42 bytes
- PDA seeds: `["escrow", maker_pubkey, seed, bump]`
- Owned by this program

### Vault Account
- An ATA of `mint_a` whose authority is the escrow PDA
- Holds the maker's `mint_a` tokens until the trade completes or is cancelled

---

## Instructions

### Make

Allows the maker to create an escrow and deposit `mint_a` tokens.

**Accounts:**

| # | Name | Writable | Signer | Description |
|---|------|----------|--------|-------------|
| 0 | `maker` | ✓ | ✓ | The user creating the escrow |
| 1 | `mint_a` | | | The mint the maker is depositing |
| 2 | `mint_b` | | | The mint the maker wants to receive |
| 3 | `maker_ata` | ✓ | | The maker's ATA of `mint_a` |
| 4 | `vault` | ✓ | | ATA owned by the escrow PDA to hold `mint_a` |
| 5 | `escrow` | ✓ | | Escrow state account (PDA) to be created |
| 6 | `system_program` | | | For account creation |
| 7 | `token_program` | | | For token operations |
| 8 | `associated_token_program` | | | For ATA creation |

**Instruction Data:**

| Field | Type | Description |
|-------|------|-------------|
| `amount_a` | `u64` (LE) | Amount of `mint_a` to deposit |
| `amount_b` | `u64` (LE) | Amount of `mint_b` expected in return |
| `seed` | `u8` | Seed to derive the escrow PDA |
| `escrow_bump` | `u8` | Bump of the escrow PDA |

**Validation:**
- `maker` must be a signer
- `mint_a` and `mint_b` must be owned by the token program
- `maker_ata` must be owned by the token program
- `vault` and `escrow` must not be initialized (owned by the system program)
- `amount_a` and `amount_b` must be greater than 0
- Escrow PDA must match the address derived from the provided seeds

---

### Take

Allows a taker to fulfil the escrow deal atomically.

**Accounts:**

| # | Name | Writable | Signer | Description |
|---|------|----------|--------|-------------|
| 0 | `taker` | ✓ | ✓ | The user accepting the deal |
| 1 | `maker` | | | The user that created the escrow |
| 2 | `mint_a` | | | The mint the taker will receive |
| 3 | `mint_b` | | | The mint the taker will send |
| 4 | `taker_ata_a` | ✓ | | The taker's ATA of `mint_a` (receives tokens) |
| 5 | `taker_ata_b` | ✓ | | The taker's ATA of `mint_b` (sends tokens) |
| 6 | `vault` | ✓ | | ATA holding the maker's `mint_a` |
| 7 | `maker_ata_b` | ✓ | | The maker's ATA of `mint_b` (receives tokens) |
| 8 | `escrow` | ✓ | | The escrow state account |
| 9 | `system_program` | | | System program |
| 10 | `token_program` | | | For token operations |

**Validation:**
- `taker` must be a signer
- `mint_a` and `mint_b` must be owned by the token program
- All ATAs must be owned by the token program
- `taker_ata_a` must have correct owner (taker) and mint (`mint_a`)
- `taker_ata_b` must have correct owner (taker) and mint (`mint_b`)
- `vault` must be owned by the escrow PDA and hold `mint_a`
- `maker_ata_b` must be owned by the maker and hold `mint_b`
- Escrow PDA must match the address derived from the seeds stored in the escrow account
- `mint_b` must match the one stored in the escrow account

**Flow:**
1. Transfer `amount_b` of `mint_b` from the taker to the maker
2. Transfer all `mint_a` from the vault to the taker (signed by the escrow PDA)
3. Close the vault ATA (rent returned to maker)
4. Close the escrow account (rent returned to maker)

---

### Refund

Allows the maker to cancel the open escrow and reclaim their tokens.

**Accounts:**

| # | Name | Writable | Signer | Description |
|---|------|----------|--------|-------------|
| 0 | `maker` | ✓ | ✓ | The user cancelling the escrow |
| 1 | `mint_a` | | | The mint the maker originally deposited |
| 2 | `mint_b` | | | The mint the maker was expecting |
| 3 | `maker_ata` | ✓ | | The maker's ATA of `mint_a` (receives tokens back) |
| 4 | `vault` | ✓ | | ATA holding the maker's `mint_a` |
| 5 | `escrow` | ✓ | | The escrow state account |
| 6 | `system_program` | | | System program |
| 7 | `token_program` | | | For token operations |

**Validation:**
- `maker` must be a signer
- `mint_a` and `mint_b` must be owned by the token program
- `maker_ata` and `vault` must be owned by the token program
- `maker_ata` must have correct owner (maker) and mint (`mint_a`)
- `vault` must be owned by the escrow PDA and hold `mint_a`
- Escrow PDA must match the address derived from the seeds stored in the escrow account
- `mint_b` must match the one stored in the escrow account

**Flow:**
1. Transfer all `mint_a` from the vault back to the maker (signed by the escrow PDA)
2. Close the vault ATA (rent returned to maker)
3. Close the escrow account (rent returned to maker)
