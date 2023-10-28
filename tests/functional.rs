#![cfg(feature = "test-sbf")]
use {
    solana_program::{
        borsh0_10::get_packed_len, instruction::InstructionError, pubkey::Pubkey, rent::Rent,
        system_instruction,
    },
    solana_program_test::*,
    solana_sdk::{
        signature::{Keypair, Signer},
        transaction::{Transaction, TransactionError},
    },
    vault::{error::VaultError, id, instruction, processor::Processor, state::VaultRecord},
};

fn program_test() -> ProgramTest {
    ProgramTest::new("vault", id(), processor!(Processor::process_instruction))
}

// Helper: create and initialize a vault account.
async fn initialize_account(
    context: &mut ProgramTestContext,
    pda: &Keypair,
    dart: &Keypair,
    authority: &Keypair,
) {
    // Rent
    let space = VaultRecord::LEN;
    let lamports = Rent::default().minimum_balance(space);
    println!("rent lamports = {lamports}");

    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &pda.pubkey(),
                lamports,
                space as u64,
                &id(),
            ),
            instruction::initialize(id(), &pda.pubkey(), &dart.pubkey(), &authority.pubkey()),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, pda, dart],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
}

#[tokio::test]
async fn initialize_success() {
    let mut context = program_test().start_with_context().await;

    let pda = Keypair::new();
    let dart = Keypair::new();
    let authority = Keypair::new();

    initialize_account(&mut context, &pda, &dart, &authority).await;
    let account_data = context
        .banks_client
        .get_account_data_with_borsh::<VaultRecord>(pda.pubkey())
        .await
        .unwrap();
    assert_eq!(account_data.dart, dart.pubkey());
    assert_eq!(account_data.authority, authority.pubkey());
    assert_eq!(account_data.version, VaultRecord::CURRENT_VERSION);
}

#[tokio::test]
async fn initialize_with_seed_success() {
    let mut context = program_test().start_with_context().await;

    let dart = Keypair::new();
    let seed = "U5f76katXToqua7SJzvP7"; // Could be DART account primary key
    let pda = Pubkey::create_with_seed(&dart.pubkey(), seed, &id()).unwrap();
    let authority = Keypair::new();

    // Rent
    let space = get_packed_len::<VaultRecord>();
    let lamports = Rent::default().minimum_balance(space);
    assert_eq!(space, VaultRecord::LEN);

    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account_with_seed(
                &context.payer.pubkey(),
                &pda,
                &dart.pubkey(),
                seed,
                lamports,
                space as u64,
                &id(),
            ),
            instruction::initialize(id(), &pda, &dart.pubkey(), &authority.pubkey()),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &dart],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
    let account_data = context
        .banks_client
        .get_account_data_with_borsh::<VaultRecord>(pda)
        .await
        .unwrap();
    assert_eq!(account_data.dart, dart.pubkey());
    assert_eq!(account_data.authority, authority.pubkey());
    assert_eq!(account_data.version, VaultRecord::CURRENT_VERSION);
}

#[tokio::test]
async fn initialize_twice_fail() {
    let mut context = program_test().start_with_context().await;

    let pda = Keypair::new();
    let dart = Keypair::new();
    let authority = Keypair::new();

    // First init (success)
    initialize_account(&mut context, &pda, &dart, &authority).await;

    // Second init (should fail)
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::initialize(
            id(),
            &pda.pubkey(),
            &dart.pubkey(),
            &authority.pubkey(),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &dart],
        context.last_blockhash,
    );
    assert_eq!(
        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );
}

#[tokio::test]
async fn transfer_authority_success() {
    let mut context = program_test().start_with_context().await;

    let pda = Keypair::new();
    let dart = Keypair::new();
    let authority = Keypair::new();

    initialize_account(&mut context, &pda, &dart, &authority).await;

    // The new owner
    let new_authority = Keypair::new();

    // Tx must be signed by DART -and- the existing owner.
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::transfer_authority(
            id(),
            &pda.pubkey(),
            &dart.pubkey(),
            &authority.pubkey(),
            &new_authority.pubkey(),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &dart, &authority],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let record = context
        .banks_client
        .get_account_data_with_borsh::<VaultRecord>(pda.pubkey())
        .await
        .unwrap();

    // Ensure the new owner was set in the record.
    assert_eq!(record.authority, new_authority.pubkey());
}

#[tokio::test]
async fn transfer_authority_fail_wrong_authority() {
    let mut context = program_test().start_with_context().await;

    let pda = Keypair::new();
    let dart = Keypair::new();
    let authority = Keypair::new();

    initialize_account(&mut context, &pda, &dart, &authority).await;

    // The new owner
    let new_authority = Keypair::new();

    // Try to use this as the authority
    let wrong_authority = Keypair::new();

    let transaction = Transaction::new_signed_with_payer(
        &[instruction::transfer_authority(
            id(),
            &pda.pubkey(),
            &dart.pubkey(),
            &wrong_authority.pubkey(),
            &new_authority.pubkey(),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &dart, &wrong_authority],
        context.last_blockhash,
    );

    assert_eq!(
        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(VaultError::IncorrectAuthority as u32)
        )
    );
}

#[tokio::test]
async fn close_account_success() {
    let mut context = program_test().start_with_context().await;

    let pda = Keypair::new();
    let dart = Keypair::new();
    let authority = Keypair::new();

    initialize_account(&mut context, &pda, &dart, &authority).await;

    let transaction = Transaction::new_signed_with_payer(
        &[instruction::close_account(
            id(),
            &pda.pubkey(),
            &dart.pubkey(),
            &authority.pubkey(),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &dart, &authority],
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let recipient = context
        .banks_client
        .get_account(authority.pubkey())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        recipient.lamports,
        Rent::default().minimum_balance(get_packed_len::<VaultRecord>())
    );
}

#[tokio::test]
async fn close_account_fail_wrong_authority() {
    let mut context = program_test().start_with_context().await;

    let pda = Keypair::new();
    let dart = Keypair::new();
    let authority = Keypair::new();

    initialize_account(&mut context, &pda, &dart, &authority).await;

    let wrong_authority = Keypair::new();
    let transaction = Transaction::new_signed_with_payer(
        &[instruction::close_account(
            id(),
            &pda.pubkey(),
            &dart.pubkey(),
            &wrong_authority.pubkey(),
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer, &dart, &wrong_authority],
        context.last_blockhash,
    );
    assert_eq!(
        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(VaultError::IncorrectAuthority as u32)
        )
    );
}
