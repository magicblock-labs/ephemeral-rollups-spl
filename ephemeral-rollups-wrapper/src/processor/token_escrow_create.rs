use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::system_program;

use crate::state::token_escrow::TokenEscrow;
use crate::token_escrow_seeds_generator;
use crate::util::create::create_pda;
use crate::util::ensure::ensure_is_owned_by_program;
use crate::util::ensure::ensure_is_pda;
use crate::util::ensure::ensure_is_program_id;
use crate::util::ensure::ensure_is_signer;

pub const DISCRIMINANT: [u8; 8] =
    [0xFE, 0x25, 0x5A, 0x94, 0x2E, 0x8E, 0x50, 0xAC];

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Args {
    pub authority: Pubkey,
    pub validator: Pubkey,
    pub token_mint: Pubkey,
    pub slot: u64,
}

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Read instruction inputs
    let [payer, token_escrow_pda, system_program_id] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let args = Args::try_from_slice(data)?;

    // Verify the programs
    ensure_is_program_id(system_program_id, &system_program::ID)?;

    // Verify that the payer is allowed to pay for the rent fees
    ensure_is_signer(payer)?;

    // Verify that the escrow PDA is currently un-initialized
    ensure_is_owned_by_program(token_escrow_pda, &system_program::ID)?;

    // Verify the seeds of the escrow PDA
    let token_escrow_seeds = token_escrow_seeds_generator!(
        args.authority,
        args.validator,
        args.token_mint,
        args.slot
    );
    let token_escrow_bump =
        ensure_is_pda(token_escrow_pda, token_escrow_seeds, program_id)?;

    // Initialize the escrow PDA
    create_pda(
        payer,
        token_escrow_pda,
        token_escrow_seeds,
        token_escrow_bump,
        TokenEscrow::space(),
        program_id,
        system_program_id,
    )?;

    // Initialize the escrow data
    let token_escrow_data =
        TokenEscrow { discriminant: TokenEscrow::discriminant(), amount: 0 };
    token_escrow_data.serialize(
        &mut &mut token_escrow_pda.try_borrow_mut_data()?.as_mut(),
    )?;

    // Log outcome
    msg!("Ephemeral Rollups Wrapper: Created a new TokenEscrow");
    msg!(" - authority: {} (slot: {})", args.authority, args.slot);
    msg!(" - validator: {}", args.validator);
    msg!(" - token_mint: {}", args.token_mint);

    // Done
    Ok(())
}
