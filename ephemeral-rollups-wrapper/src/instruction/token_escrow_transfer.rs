use borsh::BorshSerialize;
use solana_program::instruction::AccountMeta;
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;

use crate::processor::token_escrow_transfer;
use crate::state::token_escrow::TokenEscrow;

pub fn instruction(
    source_authority: &Pubkey,
    destination_authority: &Pubkey,
    validator: &Pubkey,
    token_mint: &Pubkey,
    source_slot: u64,
    destination_slot: u64,
    amount: u64,
) -> Instruction {
    let program_id = crate::ID;

    let source_token_escrow_pda = TokenEscrow::generate_pda(
        source_authority,
        validator,
        token_mint,
        source_slot,
        &program_id,
    );
    let destination_token_escrow_pda = TokenEscrow::generate_pda(
        destination_authority,
        validator,
        token_mint,
        destination_slot,
        &program_id,
    );

    let accounts = vec![
        AccountMeta::new_readonly(*source_authority, true),
        AccountMeta::new(source_token_escrow_pda, false),
        AccountMeta::new(destination_token_escrow_pda, false),
    ];

    let mut data = Vec::new();
    data.extend_from_slice(&token_escrow_transfer::DISCRIMINANT);
    token_escrow_transfer::Args {
        validator: *validator,
        token_mint: *token_mint,
        destination_authority: *destination_authority,
        source_slot,
        destination_slot,
        amount,
    }
    .serialize(&mut data)
    .unwrap();

    Instruction { program_id, accounts, data }
}
