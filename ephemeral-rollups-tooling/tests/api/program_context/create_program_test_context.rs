use ephemeral_rollups_sdk::consts::DELEGATION_PROGRAM_ID;
use solana_program_test::ProgramTest;
use solana_program_test::ProgramTestContext;

pub async fn create_program_test_context() -> ProgramTestContext {
    let mut program_test = ProgramTest::default();
    program_test.prefer_bpf(true);
    program_test.add_program("./binaries/dlp", DELEGATION_PROGRAM_ID, None);
    program_test.add_program("./binaries/bubblegum", mpl_bubblegum::ID, None);
    program_test.add_program("./binaries/noop", spl_noop::ID, None);
    program_test.add_program("./binaries/compression", spl_account_compression::ID, None);
    program_test.add_program(
        "../target/deploy/ephemeral_rollups_wrapper",
        ephemeral_rollups_wrapper::ID,
        None,
    );
    program_test.start_with_context().await
}
