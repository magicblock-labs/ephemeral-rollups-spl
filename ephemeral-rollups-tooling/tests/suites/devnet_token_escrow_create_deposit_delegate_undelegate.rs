use ephemeral_rollups_wrapper::state::token_escrow::TokenEscrow;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use spl_token::state::Account;

use crate::api::program_context::program_context_trait::ProgramContext;
use crate::api::program_context::program_error::ProgramError;
use crate::api::program_context::read_account::{read_account_borsh, read_account_packed};
use crate::api::program_delegation::process_delegate_on_curve::process_delegate_on_curve;
use crate::api::program_delegation::wait_until_undelegation::wait_until_undelegation;
use crate::api::program_spl::process_associated_token_account_get_or_init::process_associated_token_account_get_or_init;
use crate::api::program_spl::process_token_mint_init::process_token_mint_init;
use crate::api::program_spl::process_token_mint_to::process_token_mint_to;
use crate::api::program_wrapper::process_token_escrow_create::process_token_escrow_create;
use crate::api::program_wrapper::process_token_escrow_delegate::process_token_escrow_delegate;
use crate::api::program_wrapper::process_token_escrow_deposit::process_token_escrow_deposit;
use crate::api::program_wrapper::process_token_escrow_transfer::process_token_escrow_transfer;
use crate::api::program_wrapper::process_token_escrow_undelegate::process_token_escrow_undelegate;
use crate::api::program_wrapper::process_token_escrow_withdraw::process_token_escrow_withdraw;
use crate::api::program_wrapper::process_token_vault_init::process_token_vault_init;

#[tokio::test]
async fn devnet_token_escrow_create_deposit_delegate_undelegate() -> Result<(), ProgramError> {
    let mut program_context_chain: Box<dyn ProgramContext> =
        Box::new(RpcClient::new_with_commitment(
            "https://api.devnet.solana.com".to_string(),
            CommitmentConfig::confirmed(),
        ));
    let mut program_context_ephem: Box<dyn ProgramContext> =
        Box::new(RpcClient::new_with_commitment(
            "https://devnet.magicblock.app".to_string(),
            CommitmentConfig::confirmed(),
        ));

    // Devnet dummy payer: Payi9ovX2Tbe69XuUdgav5qS3sVnNAn2dN8BZoAQwyq
    let payer_chain = Keypair::from_bytes(&[
        243, 85, 166, 238, 237, 2, 46, 208, 68, 40, 98, 2, 148, 117, 134, 238, 144, 223, 165, 108,
        203, 120, 96, 89, 172, 223, 98, 26, 162, 92, 234, 167, 5, 201, 50, 82, 10, 153, 196, 60,
        132, 31, 123, 66, 63, 113, 122, 83, 145, 102, 200, 15, 46, 50, 207, 1, 6, 109, 0, 216, 225,
        247, 70, 96,
    ])
    .map_err(|e| ProgramError::Signature(e.to_string()))?;

    // Important keys used in the test
    let validator = Pubkey::new_unique();

    let authority1 = Keypair::new();
    let authority2 = Keypair::new();

    let chain_input = Keypair::new();
    let chain_output = Pubkey::new_unique();

    // Create token mint
    let token_mint = Keypair::new();
    process_token_mint_init(
        &mut program_context_chain,
        &payer_chain,
        &token_mint,
        6,
        &token_mint.pubkey(),
    )
    .await?;

    // Airdrop token to our chain_input wallet
    let chain_input_token = process_associated_token_account_get_or_init(
        &mut program_context_chain,
        &payer_chain,
        &token_mint.pubkey(),
        &chain_input.pubkey(),
    )
    .await?;
    process_token_mint_to(
        &mut program_context_chain,
        &payer_chain,
        &token_mint.pubkey(),
        &token_mint,
        &chain_input_token,
        10_000_000,
    )
    .await?;

    // Escrow accounts we will be creating
    let authority1_token_escrow_slot = 99;
    let authority1_token_escrow_pda = TokenEscrow::generate_pda(
        &authority1.pubkey(),
        &validator,
        &token_mint.pubkey(),
        authority1_token_escrow_slot,
        &ephemeral_rollups_wrapper::ID,
    );
    let authority2_token_escrow_slot = 11;
    let authority2_token_escrow_pda = TokenEscrow::generate_pda(
        &authority2.pubkey(),
        &validator,
        &token_mint.pubkey(),
        authority2_token_escrow_slot,
        &ephemeral_rollups_wrapper::ID,
    );

    // Prepare being able to escrow this token mint for this validator
    process_token_vault_init(
        &mut program_context_chain,
        &payer_chain,
        &validator,
        &token_mint.pubkey(),
    )
    .await?;

    // Create all escrows
    process_token_escrow_create(
        &mut program_context_chain,
        &payer_chain,
        &authority1.pubkey(),
        &validator,
        &token_mint.pubkey(),
        authority1_token_escrow_slot,
    )
    .await?;
    process_token_escrow_create(
        &mut program_context_chain,
        &payer_chain,
        &authority2.pubkey(),
        &validator,
        &token_mint.pubkey(),
        authority2_token_escrow_slot,
    )
    .await?;

    // Fund the first escrow
    process_token_escrow_deposit(
        &mut program_context_chain,
        &payer_chain,
        &chain_input,
        &chain_input_token,
        &authority1.pubkey(),
        &validator,
        &token_mint.pubkey(),
        authority1_token_escrow_slot,
        10_000_000,
    )
    .await?;

    // Delegate all escrows
    process_token_escrow_delegate(
        &mut program_context_chain,
        &payer_chain,
        &authority1,
        &validator,
        &token_mint.pubkey(),
        authority1_token_escrow_slot,
    )
    .await?;
    process_token_escrow_delegate(
        &mut program_context_chain,
        &payer_chain,
        &authority2,
        &validator,
        &token_mint.pubkey(),
        authority2_token_escrow_slot,
    )
    .await?;

    // Ephemeral dummy payer, delegate it to be used in the ER
    let payer_ephem = Keypair::new();
    process_delegate_on_curve(
        &mut program_context_chain,
        &payer_chain,
        &payer_ephem,
        1_000_000,
    )
    .await?;

    // Do a transfer between the two escrow inside of the ER
    process_token_escrow_transfer(
        &mut program_context_ephem,
        &payer_ephem,
        &authority1,
        &authority2.pubkey(),
        &validator,
        &token_mint.pubkey(),
        authority1_token_escrow_slot,
        authority2_token_escrow_slot,
        1_000_000,
    )
    .await?;

    // Transfer success should be reflected in the balances inside the ER
    assert_eq!(
        9_000_000,
        read_account_borsh::<TokenEscrow>(&mut program_context_ephem, &authority1_token_escrow_pda)
            .await?
            .amount
    );
    assert_eq!(
        1_000_000,
        read_account_borsh::<TokenEscrow>(&mut program_context_ephem, &authority2_token_escrow_pda)
            .await?
            .amount
    );

    // Undelegate all escrows
    process_token_escrow_undelegate(
        &mut program_context_ephem,
        &payer_ephem,
        &authority1,
        &validator,
        &token_mint.pubkey(),
        authority1_token_escrow_slot,
    )
    .await?;
    process_token_escrow_undelegate(
        &mut program_context_ephem,
        &payer_ephem,
        &authority2,
        &validator,
        &token_mint.pubkey(),
        authority2_token_escrow_slot,
    )
    .await?;

    // Wait for undelegations to succeed
    wait_until_undelegation(&mut program_context_chain, &authority1_token_escrow_pda).await?;
    wait_until_undelegation(&mut program_context_chain, &authority2_token_escrow_pda).await?;

    // Transfer success should be reflected in the balances on the chain
    assert_eq!(
        9_000_000,
        read_account_borsh::<TokenEscrow>(&mut program_context_chain, &authority1_token_escrow_pda)
            .await?
            .amount
    );
    assert_eq!(
        1_000_000,
        read_account_borsh::<TokenEscrow>(&mut program_context_chain, &authority2_token_escrow_pda)
            .await?
            .amount
    );

    // Just for fun, we should now be able to withdraw funds on-chain
    let chain_output_token = process_associated_token_account_get_or_init(
        &mut program_context_chain,
        &payer_chain,
        &token_mint.pubkey(),
        &chain_output,
    )
    .await?;

    process_token_escrow_withdraw(
        &mut program_context_chain,
        &payer_chain,
        &authority1,
        &chain_output_token,
        &validator,
        &token_mint.pubkey(),
        authority1_token_escrow_slot,
        9_000_000,
    )
    .await?;
    process_token_escrow_withdraw(
        &mut program_context_chain,
        &payer_chain,
        &authority2,
        &chain_output_token,
        &validator,
        &token_mint.pubkey(),
        authority2_token_escrow_slot,
        1_000_000,
    )
    .await?;

    // Verify that the on-chain chain_output token account now has our tokens
    assert_eq!(
        10_000_000,
        read_account_packed::<Account>(&mut program_context_chain, &chain_output_token)
            .await?
            .amount
    );

    // Done
    Ok(())
}
