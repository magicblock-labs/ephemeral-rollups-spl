use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use ephemeral_rollups_wrapper::instruction::bubblegum_escrow_transfer;

use crate::api::program_context::process_instruction::process_instruction_with_signer;
use crate::api::program_context::program_context_trait::ProgramContext;
use crate::api::program_context::program_error::ProgramError;

pub async fn process_bubblegum_escrow_transfer(
    program_context: &mut Box<dyn ProgramContext>,
    payer: &Keypair,
    source_authority: &Keypair,
    destination_authority: &Pubkey,
    validator: &Pubkey,
    tree: &Pubkey,
    nonce: u64,
) -> Result<(), ProgramError> {
    let instruction = bubblegum_escrow_transfer::instruction(
        &source_authority.pubkey(),
        destination_authority,
        validator,
        tree,
        nonce,
    );
    process_instruction_with_signer(program_context, instruction, payer, source_authority).await
}
