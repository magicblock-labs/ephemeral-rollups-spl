use solana_program::program::invoke;
use solana_program::program_error::ProgramError;
use solana_program::system_program;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};
use spl_token::instruction::initialize_account3;

use crate::state::token_escrow::TokenEscrow;
use crate::token_vault_seeds_generator;
use crate::util::create::create_pda;
use crate::util::ensure::{ensure_is_owned_by_program, ensure_is_pda};

pub const DISCRIMINANT: [u8; 8] = [0x70, 0xfe, 0x66, 0x40, 0x47, 0x49, 0x16, 0x0e];

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    // Read instruction inputs
    let [payer, validator_id, token_mint, token_vault_pda, token_program, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Verify that the escrow PDA is currently un-initialized
    ensure_is_owned_by_program(token_vault_pda, &system_program::ID)?;

    // Verify the seeds of the escrow PDA
    let token_vault_seeds = token_vault_seeds_generator!(validator_id.key, token_mint.key);
    ensure_is_pda(token_vault_pda, token_vault_seeds, program_id)?;

    // Initialize the escrow PDA
    create_pda(
        payer,
        token_vault_pda,
        token_vault_seeds,
        TokenEscrow::space(),
        program_id,
        system_program,
    )?;

    // Write the spl token vault's content
    invoke(
        &initialize_account3(
            token_program.key,
            token_vault_pda.key,
            token_mint.key,
            token_vault_pda.key,
        )?,
        &[token_vault_pda.clone(), token_vault_pda.clone()],
    )?;

    // Done
    Ok(())
}
