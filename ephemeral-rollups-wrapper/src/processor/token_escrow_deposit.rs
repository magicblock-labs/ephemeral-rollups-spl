use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::invoke;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use spl_token::instruction::transfer;

use crate::state::token_escrow::TokenEscrow;
use crate::token_escrow_seeds_generator;
use crate::token_vault_seeds_generator;
use crate::util::ensure::ensure_is_owned_by_program;
use crate::util::ensure::ensure_is_pda;
use crate::util::ensure::ensure_is_program_id;

pub const DISCRIMINANT: [u8; 8] =
    [0xE0, 0x6C, 0xBE, 0x01, 0x34, 0xE4, 0x4B, 0xF2];

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Args {
    pub authority: Pubkey,
    pub validator: Pubkey,
    pub token_mint: Pubkey,
    pub slot: u64,
    pub amount: u64,
}

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Read instruction inputs
    let [source_authority, source_token_account, token_escrow_pda, token_vault_pda, token_program_id] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let args = Args::try_from_slice(data)?;

    // Verify the programs
    ensure_is_program_id(token_program_id, &spl_token::ID)?;

    // Verify that the program has proper control of the escrow PDA (and that
    // it's been initialized)
    ensure_is_owned_by_program(token_escrow_pda, program_id)?;

    // Verify that the vault has been initialized properly
    ensure_is_owned_by_program(token_vault_pda, token_program_id.key)?;

    // Verify the seeds of the escrow PDA
    let token_escrow_seeds = token_escrow_seeds_generator!(
        args.authority,
        args.validator,
        args.token_mint,
        args.slot
    );
    ensure_is_pda(token_escrow_pda, token_escrow_seeds, program_id)?;

    // Verify the seeds of the vault PDA
    let token_vault_seeds =
        token_vault_seeds_generator!(args.validator, args.token_mint);
    ensure_is_pda(token_vault_pda, token_vault_seeds, program_id)?;

    // Proceed to transfer the token amount from source_token_account to vault
    invoke(
        &transfer(
            token_program_id.key,
            source_token_account.key,
            token_vault_pda.key,
            source_authority.key,
            &[],
            args.amount,
        )?,
        &[
            source_token_account.clone(),
            token_vault_pda.clone(),
            source_authority.clone(),
        ],
    )?;

    // Update the escrow amount (if the transfer succeeded)
    let mut token_escrow_data =
        TokenEscrow::try_from_slice(&token_escrow_pda.data.borrow())?;
    if token_escrow_data.discriminant != TokenEscrow::discriminant() {
        return Err(ProgramError::InvalidAccountData);
    }
    token_escrow_data.amount =
        token_escrow_data.amount.checked_add(args.amount).unwrap();
    token_escrow_data.serialize(
        &mut &mut token_escrow_pda.try_borrow_mut_data()?.as_mut(),
    )?;

    // Log outcome
    msg!("Ephemeral Rollups Wrapper: Deposited to TokenEscrow");
    msg!(" - authority: {} (slot: {})", args.authority, args.slot);
    msg!(" - validator: {}", args.validator);
    msg!(" - token_mint: {}", args.token_mint);
    msg!(" - amount: {} (total: {})", args.amount, token_escrow_data.amount);

    // Done
    Ok(())
}
