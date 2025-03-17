use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum AIInfraInstruction {
    RegisterAgent,
    CreateTask {
        requirements: ComputeRequirements,
    },
    DepositCredits {
        amount: u64,
    },
}