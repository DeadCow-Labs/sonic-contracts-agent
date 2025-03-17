use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    clock::Epoch,
    pubkey::Pubkey,
    rent::Rent,
    system_program,
    hash::Hash,
    sysvar,
    system_instruction,
    msg,
};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
    instruction::{AccountMeta, Instruction},
};
use solana_banks_client::{BanksClient, BanksClientError};
use sonic_ai_infra::{
    AIAgent,
    AIInfraInstruction,
    ComputeRequirements,
    ComputeTask,
    TaskStatus,
    process_instruction,
};

// Helper function with corrected types
async fn create_test_agent(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    program_id: Pubkey,
    agent_keypair: &Keypair,
    owner_keypair: &Keypair,
) -> Result<(), BanksClientError> {
    let instruction_data = AIInfraInstruction::RegisterAgent.try_to_vec().unwrap();
    
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &instruction_data,
            vec![
                AccountMeta::new(agent_keypair.pubkey(), false),
                AccountMeta::new(owner_keypair.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
    );
    
    transaction.sign(&[payer, owner_keypair], recent_blockhash);
    banks_client.process_transaction(transaction).await
}

#[tokio::test]
async fn test_agent_registration() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "sonic_ai_infra",
        program_id,
        processor!(process_instruction),
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create a new account
    let agent_account = Keypair::new();
    
    // Calculate exact size needed
    let space = 32 + // Pubkey (owner)
                8 +  // u64 (compute_credits)
                4 +  // u32 (reputation_score)
                4 +  // u32 (tasks_completed)
                1;   // bool (is_active)

    let rent = banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(space);

    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &agent_account.pubkey(),
        lamports,
        space as u64,
        &program_id,
    );

    let register_ix = Instruction::new_with_borsh(
        program_id,
        &AIInfraInstruction::RegisterAgent,
        vec![
            AccountMeta::new(agent_account.pubkey(), false),
            AccountMeta::new_readonly(payer.pubkey(), true),
        ],
    );

    let transaction = Transaction::new_signed_with_payer(
        &[create_account_ix, register_ix],
        Some(&payer.pubkey()),
        &[&payer, &agent_account],
        recent_blockhash,
    );

    // Just verify the transaction succeeds
    banks_client.process_transaction(transaction).await.unwrap();

    // Only verify the account exists
    let account = banks_client.get_account(agent_account.pubkey()).await.unwrap().unwrap();
    assert_eq!(account.owner, program_id);
}

#[tokio::test]
async fn test_task_creation_and_execution() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "sonic_ai_infra",
        program_id,
        processor!(process_instruction),
    );
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    let agent_keypair = Keypair::new();
    let task_keypair = Keypair::new();
    let owner_keypair = Keypair::new();

    // Setup initial agent account with credits
    let initial_credits = 1000;
    let agent = AIAgent {
        owner: owner_keypair.pubkey(),
        compute_credits: initial_credits,
        reputation_score: 100,
        tasks_completed: 0,
        is_active: true,
    };

    let agent_account = Account {
        lamports: Rent::default().minimum_balance(std::mem::size_of::<AIAgent>()),
        data: agent.try_to_vec().unwrap(),
        owner: program_id,
        executable: false,
        rent_epoch: Epoch::default(),
    };

    program_test.add_account(agent_keypair.pubkey(), agent_account);

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Create task
    let requirements = ComputeRequirements {
        cpu_units: 100,
        memory_mb: 512,
        storage_mb: 1024,
        max_time_seconds: 3600,
    };

    let payment_amount = 500;
    let instruction_data = AIInfraInstruction::CreateTask {
        requirements,
        payment_amount,
    }
    .try_to_vec()
    .unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &instruction_data,
            vec![
                AccountMeta::new(task_keypair.pubkey(), false),
                AccountMeta::new(agent_keypair.pubkey(), false),
                AccountMeta::new(owner_keypair.pubkey(), true),
            ],
        )],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer, &owner_keypair], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify task creation
    let task_account = banks_client
        .get_account(task_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();

    let task = ComputeTask::try_from_slice(&task_account.data).unwrap();
    assert_eq!(task.agent, agent_keypair.pubkey());
    assert_eq!(task.payment_amount, payment_amount);
    assert_eq!(task.status, TaskStatus::Pending);

    // Verify agent credits deduction
    let agent_account = banks_client
        .get_account(agent_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();

    let updated_agent = AIAgent::try_from_slice(&agent_account.data).unwrap();
    assert_eq!(
        updated_agent.compute_credits,
        initial_credits - payment_amount
    );
}

#[tokio::test]
async fn test_credit_management() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "sonic_ai_infra",
        program_id,
        processor!(process_instruction),
    );
    
    let agent_keypair = Keypair::new();
    let owner_keypair = Keypair::new();
    
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Register agent first
    create_test_agent(
        &mut banks_client,
        &payer,
        recent_blockhash,
        program_id,
        &agent_keypair,
        &owner_keypair,
    )
    .await
    .unwrap();

    // Test deposit credits
    let deposit_amount = 1000;
    let instruction_data = AIInfraInstruction::DepositCredits {
        amount: deposit_amount,
    }
    .try_to_vec()
    .unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &instruction_data,
            vec![
                AccountMeta::new(agent_keypair.pubkey(), false),
                AccountMeta::new(owner_keypair.pubkey(), true),
            ],
        )],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer, &owner_keypair], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify credit deposit
    let agent_account = banks_client
        .get_account(agent_keypair.pubkey())
        .await
        .unwrap()
        .unwrap();

    let agent = AIAgent::try_from_slice(&agent_account.data).unwrap();
    assert_eq!(agent.compute_credits, deposit_amount);
}