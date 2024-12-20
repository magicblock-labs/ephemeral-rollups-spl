use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use ephemeral_rollups_wrapper::instruction::lamport_escrow_undelegate;

use crate::api::program_context::process_instruction::process_instruction_with_signer;
use crate::api::program_context::program_context_trait::ProgramContext;
use crate::api::program_context::program_error::ProgramError;

pub async fn process_lamport_escrow_undelegate(
    program_context: &mut Box<dyn ProgramContext>,
    payer: &Keypair,
    authority: &Keypair,
    validator: &Pubkey,
    slot: u64,
) -> Result<(), ProgramError> {
    let instruction = lamport_escrow_undelegate::instruction(
        &payer.pubkey(),
        &authority.pubkey(),
        validator,
        slot,
    );
    process_instruction_with_signer(program_context, instruction, payer, authority).await
}
