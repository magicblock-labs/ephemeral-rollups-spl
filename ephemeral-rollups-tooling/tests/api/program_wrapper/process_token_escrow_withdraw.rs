use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;

use ephemeral_rollups_wrapper::instruction::token_escrow_withdraw;

use crate::api::program_context::process_instruction::process_instruction_with_signer;
use crate::api::program_context::program_context_trait::ProgramContext;
use crate::api::program_context::program_error::ProgramError;

pub async fn process_token_escrow_withdraw(
    program_context: &mut Box<dyn ProgramContext>,
    payer: &Keypair,
    authority: &Keypair,
    destination_token_account: &Pubkey,
    validator: &Pubkey,
    token_mint: &Pubkey,
    slot: u64,
    amount: u64,
) -> Result<(), ProgramError> {
    let instruction = token_escrow_withdraw::instruction(
        &authority.pubkey(),
        destination_token_account,
        validator,
        token_mint,
        slot,
        amount,
    );
    process_instruction_with_signer(program_context, instruction, payer, authority).await
}
