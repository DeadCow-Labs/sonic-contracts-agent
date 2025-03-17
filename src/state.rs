use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct AIAgent {
    pub owner: Pubkey,
    pub credits: u64,
    pub compute_credits: u64,
    pub reputation_score: u32,
    pub tasks_completed: u32,
    pub is_active: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct ComputeTask {
    pub owner: Pubkey,
    pub requirements: ComputeRequirements,
    pub status: TaskStatus,
    pub result_hash: [u8; 32],
    pub payment_amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct ComputeRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub storage_mb: u32,
    pub max_time_seconds: u32,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum AIInfraInstruction {
    RegisterAgent,
    UpdateAgentStatus { is_active: bool },
    CreateTask {
        requirements: ComputeRequirements,
        payment_amount: u64,
    },
    StartTask { task_id: Pubkey },
    CompleteTask {
        task_id: Pubkey,
        result_hash: [u8; 32],
    },
    DepositCredits { amount: u64 },
    WithdrawCredits { amount: u64 },
}