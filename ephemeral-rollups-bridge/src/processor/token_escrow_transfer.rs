use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::state::token_escrow::TokenEscrow;
use crate::token_escrow_seeds_generator;
use crate::util::ensure::{ensure_is_owned_by_program, ensure_is_pda, ensure_is_signer};

pub const DISCRIMINANT: [u8; 8] = [0x01, 0x1d, 0xe7, 0xcb, 0x37, 0x6e, 0x04, 0x70];

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Args {
    pub source_index: u64,
    pub destination_index: u64,
    pub amount: u64,
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Read instruction inputs
    let [source_authority, destination_authority, validator, token_mint, source_token_escrow_pda, destination_token_escrow_pda] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let args = Args::try_from_slice(data)?;

    // Verify that the authority user is indeed the one initiating this IX
    ensure_is_signer(source_authority)?;

    // Verify that the program has proper control of the escrow PDA (and that it's been initialized)
    ensure_is_owned_by_program(source_token_escrow_pda, program_id)?;

    // Verify that the program has proper control of the escrow PDA (and that it's been initialized)
    ensure_is_owned_by_program(destination_token_escrow_pda, program_id)?;

    // Verify the seeds of the escrow PDA
    let source_token_escrow_seeds = token_escrow_seeds_generator!(
        source_authority.key,
        validator.key,
        token_mint.key,
        args.source_index
    );
    ensure_is_pda(
        source_token_escrow_pda,
        source_token_escrow_seeds,
        program_id,
    )?;

    // Verify the seeds of the escrow PDA
    let destination_token_escrow_seeds = token_escrow_seeds_generator!(
        destination_authority.key,
        validator.key,
        token_mint.key,
        args.destination_index
    );
    ensure_is_pda(
        destination_token_escrow_pda,
        destination_token_escrow_seeds,
        program_id,
    )?;

    // Update the source escrow amount (panic if not enough amount available)
    let mut source_token_escrow_data =
        TokenEscrow::try_from_slice(&source_token_escrow_pda.data.borrow())?;
    if source_token_escrow_data.discriminant != TokenEscrow::discriminant() {
        return Err(ProgramError::InvalidAccountData);
    }
    source_token_escrow_data.amount = source_token_escrow_data
        .amount
        .checked_sub(args.amount)
        .unwrap();
    source_token_escrow_data
        .serialize(&mut &mut source_token_escrow_pda.try_borrow_mut_data()?.as_mut())?;

    // Update the destination escrow amount (if everything else suceeded)
    let mut destination_token_escrow_data =
        TokenEscrow::try_from_slice(&destination_token_escrow_pda.data.borrow())?;
    if destination_token_escrow_data.discriminant != TokenEscrow::discriminant() {
        return Err(ProgramError::InvalidAccountData);
    }
    destination_token_escrow_data.amount = destination_token_escrow_data
        .amount
        .checked_add(args.amount)
        .unwrap();
    destination_token_escrow_data
        .serialize(&mut &mut destination_token_escrow_pda.try_borrow_mut_data()?.as_mut())?;

    // Log outcome
    msg!("Ephemeral Rollups Bridge: Transfered between TokenEscrow");
    msg!(" - source_authority: {}", source_authority.key);
    msg!(" - destination_authority: {}", destination_authority.key);
    msg!(" - validator: {}", validator.key);
    msg!(" - token_mint: {}", token_mint.key);
    msg!(" - source_index: {}", args.source_index);
    msg!(" - destination_index: {}", args.destination_index);
    msg!(" - amount: {}", args.amount);

    // Done
    Ok(())
}