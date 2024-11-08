use solana_sdk::instruction::Instruction;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;

use crate::api::program_context::program_context_trait::ProgramContext;
use crate::api::program_context::program_error::ProgramError;

async fn process_instruction_result(
    instruction: Instruction,
    result: Result<(), ProgramError>,
) -> Result<(), ProgramError> {
    println!(" -------- PROCESSING INSTRUCTION --------");
    println!(
        " - instruction.program_id: {:?}",
        instruction.program_id.to_string()
    );
    println!(" - instruction.data: {:?}", instruction.data);
    let backtrace_data = std::backtrace::Backtrace::force_capture();
    let backtrace_formatted = std::format!("{}", backtrace_data);
    let backtrace_lines = backtrace_formatted.lines();
    for backtrace_line in backtrace_lines {
        if backtrace_line.contains("at ./tests/") && !backtrace_line.contains("process_instruction")
        {
            println!(" - instruction.from: {}", backtrace_line.trim());
        }
    }
    let mut idx = 0;
    for account in instruction.accounts {
        idx += 1;
        println!(" - instruction.account: #{:?} {:?}", idx, account.pubkey);
    }
    if result.is_ok() {
        println!(" - instruction.result: {:?}", "OK");
    } else {
        println!(" - instruction.result: {:?}", "ERROR");
    }
    result
}

pub async fn process_instruction(
    program_context: &mut Box<dyn ProgramContext>,
    instruction: Instruction,
    payer: &Keypair,
) -> Result<(), ProgramError> {
    let latest_blockhash = program_context.get_latest_blockhash().await?;
    let mut transaction: Transaction =
        Transaction::new_with_payer(&[instruction.clone()], Some(&payer.pubkey()));
    transaction.partial_sign(&[payer], latest_blockhash);
    let result = program_context.process_transaction(transaction).await;
    process_instruction_result(instruction.clone(), result).await
}

pub async fn process_instruction_with_signer(
    program_context: &mut Box<dyn ProgramContext>,
    instruction: Instruction,
    payer: &Keypair,
    signer: &Keypair,
) -> Result<(), ProgramError> {
    let latest_blockhash = program_context.get_latest_blockhash().await?;
    let mut transaction: Transaction =
        Transaction::new_with_payer(&[instruction.clone()], Some(&payer.pubkey()));
    transaction.partial_sign(&[payer, signer], latest_blockhash);
    let result = program_context.process_transaction(transaction).await;
    process_instruction_result(instruction.clone(), result).await
}

pub async fn process_instruction_with_signers(
    program_context: &mut Box<dyn ProgramContext>,
    instruction: Instruction,
    payer: &Keypair,
    signers: &[&Keypair],
) -> Result<(), ProgramError> {
    let latest_blockhash = program_context.get_latest_blockhash().await?;
    let mut transaction: Transaction =
        Transaction::new_with_payer(&[instruction.clone()], Some(&payer.pubkey()));
    let mut keypairs = signers.to_owned();
    keypairs.push(payer);
    transaction.partial_sign(&keypairs, latest_blockhash);
    let result = program_context.process_transaction(transaction).await;
    process_instruction_result(instruction.clone(), result).await
}
