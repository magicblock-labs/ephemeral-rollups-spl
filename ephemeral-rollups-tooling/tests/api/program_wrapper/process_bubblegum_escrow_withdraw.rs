use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_toolbox_endpoint::{Endpoint, EndpointError};

use ephemeral_rollups_wrapper::instruction::bubblegum_escrow_withdraw;

pub async fn process_bubblegum_escrow_withdraw(
    endpoint: &mut Endpoint,
    payer: &Keypair,
    authority: &Keypair,
    new_leaf_owner: &Pubkey,
    validator: &Pubkey,
    spill: &Pubkey,
    tree: &Pubkey,
    root_hash: &[u8; 32],
    data_hash: &[u8; 32],
    creator_hash: &[u8; 32],
    nonce: u64,
    index: u32,
) -> Result<(), EndpointError> {
    let instruction = bubblegum_escrow_withdraw::instruction(
        &authority.pubkey(),
        new_leaf_owner,
        validator,
        spill,
        tree,
        root_hash,
        data_hash,
        creator_hash,
        nonce,
        index,
    );
    endpoint
        .process_instruction_with_signers(instruction, payer, &[authority])
        .await?;
    Ok(())
}
