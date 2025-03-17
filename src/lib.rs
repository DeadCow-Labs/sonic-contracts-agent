use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    program::invoke,
    sysvar::rent::Rent,
    sysvar::Sysvar,
};

// AI Agent Account Structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AIAgent {
    pub owner: Pubkey,
    pub compute_credits: u64,
    pub reputation_score: u32,
    pub tasks_completed: u32,
    pub is_active: bool,
}

// Compute Task Structure
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ComputeTask {
    pub agent: Pubkey,
    pub requirements: ComputeRequirements,
    pub status: TaskStatus,
    pub result_hash: [u8; 32],
    pub payment_amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ComputeRequirements {
    pub cpu_units: u32,
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

// Program Instructions
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum AIInfraInstruction {
    // Agent Management
    RegisterAgent,
    UpdateAgentStatus { is_active: bool },
    
    // Task Management
    CreateTask {
        requirements: ComputeRequirements,
        payment_amount: u64,
    },
    StartTask { task_id: Pubkey },
    CompleteTask {
        task_id: Pubkey,
        result_hash: [u8; 32],
    },
    
    // Payment Management
    DepositCredits { amount: u64 },
    WithdrawCredits { amount: u64 },
}

// Program entrypoint
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = AIInfraInstruction::try_from_slice(instruction_data)?;
    
    match instruction {
        AIInfraInstruction::RegisterAgent => {
            process_register_agent(program_id, accounts)
        }
        AIInfraInstruction::CreateTask { requirements, payment_amount } => {
            process_create_task(program_id, accounts, requirements, payment_amount)
        }
        AIInfraInstruction::StartTask { task_id } => {
            process_start_task(program_id, accounts, task_id)
        }
        AIInfraInstruction::CompleteTask { task_id, result_hash } => {
            process_complete_task(program_id, accounts, task_id, result_hash)
        }
        AIInfraInstruction::DepositCredits { amount } => {
            process_deposit_credits(program_id, accounts, amount)
        }
        AIInfraInstruction::WithdrawCredits { amount } => {
            process_withdraw_credits(program_id, accounts, amount)
        }
        AIInfraInstruction::UpdateAgentStatus { is_active } => {
            process_update_status(program_id, accounts, is_active)
        }
    }
}

// Implementation of register_agent
fn process_register_agent(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let agent_account = next_account_info(accounts_iter)?;
    let owner_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;
    let rent_sysvar = next_account_info(accounts_iter)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let agent = AIAgent {
        owner: *owner_account.key,
        compute_credits: 0,
        reputation_score: 100, // Initial reputation
        tasks_completed: 0,
        is_active: true,
    };

    let rent = &Rent::from_account_info(rent_sysvar)?;
    let rent_lamports = rent.minimum_balance(std::mem::size_of::<AIAgent>());

    // Create account
    invoke(
        &system_instruction::create_account(
            owner_account.key,
            agent_account.key,
            rent_lamports,
            std::mem::size_of::<AIAgent>() as u64,
            program_id,
        ),
        &[owner_account.clone(), agent_account.clone(), system_program.clone()],
    )?;

    agent.serialize(&mut *agent_account.data.borrow_mut())?;
    msg!("AI Agent registered successfully");
    Ok(())
}

// Implementation of create_task
pub fn process_create_task(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    requirements: ComputeRequirements,
    payment_amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let task_account = next_account_info(accounts_iter)?;
    let agent_account = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;

    if !payer_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut agent = AIAgent::try_from_slice(&agent_account.data.borrow())?;
    if agent.compute_credits < payment_amount {
        return Err(ProgramError::InsufficientFunds);
    }

    let task = ComputeTask {
        agent: *agent_account.key,
        requirements,
        status: TaskStatus::Pending,
        result_hash: [0; 32],
        payment_amount,
    };

    task.serialize(&mut *task_account.data.borrow_mut())?;
    
    // Deduct credits
    agent.compute_credits -= payment_amount;
    agent.serialize(&mut *agent_account.data.borrow_mut())?;

    msg!("Compute task created successfully");
    Ok(())
}

pub fn process_start_task(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    task_id: Pubkey,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let task_account = next_account_info(accounts_iter)?;
    let agent_account = next_account_info(accounts_iter)?;

    if task_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut task = ComputeTask::try_from_slice(&task_account.data.borrow())?;
    if task.status != TaskStatus::Pending {
        return Err(ProgramError::InvalidAccountData);
    }

    task.status = TaskStatus::InProgress;
    task.serialize(&mut *task_account.data.borrow_mut())?;

    msg!("Task started successfully");
    Ok(())
}

pub fn process_complete_task(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    task_id: Pubkey,
    result_hash: [u8; 32],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let task_account = next_account_info(accounts_iter)?;
    let agent_account = next_account_info(accounts_iter)?;

    if task_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut task = ComputeTask::try_from_slice(&task_account.data.borrow())?;
    if task.status != TaskStatus::InProgress {
        return Err(ProgramError::InvalidAccountData);
    }

    task.status = TaskStatus::Completed;
    task.result_hash = result_hash;
    task.serialize(&mut *task_account.data.borrow_mut())?;

    // Update agent stats
    let mut agent = AIAgent::try_from_slice(&agent_account.data.borrow())?;
    agent.tasks_completed += 1;
    agent.serialize(&mut *agent_account.data.borrow_mut())?;

    msg!("Task completed successfully");
    Ok(())
}

pub fn process_deposit_credits(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let agent_account = next_account_info(accounts_iter)?;
    let owner_account = next_account_info(accounts_iter)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut agent = AIAgent::try_from_slice(&agent_account.data.borrow())?;
    if agent.owner != *owner_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    agent.compute_credits = agent.compute_credits.checked_add(amount)
        .ok_or(ProgramError::InvalidInstructionData)?;
    
    agent.serialize(&mut *agent_account.data.borrow_mut())?;

    msg!("Credits deposited successfully");
    Ok(())
}

pub fn process_withdraw_credits(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let agent_account = next_account_info(accounts_iter)?;
    let owner_account = next_account_info(accounts_iter)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut agent = AIAgent::try_from_slice(&agent_account.data.borrow())?;
    if agent.owner != *owner_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    if agent.compute_credits < amount {
        return Err(ProgramError::InsufficientFunds);
    }

    agent.compute_credits -= amount;
    agent.serialize(&mut *agent_account.data.borrow_mut())?;

    msg!("Credits withdrawn successfully");
    Ok(())
}

pub fn process_update_status(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    is_active: bool,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let agent_account = next_account_info(accounts_iter)?;
    let owner_account = next_account_info(accounts_iter)?;

    if !owner_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut agent = AIAgent::try_from_slice(&agent_account.data.borrow())?;
    if agent.owner != *owner_account.key {
        return Err(ProgramError::InvalidAccountData);
    }

    agent.is_active = is_active;
    agent.serialize(&mut *agent_account.data.borrow_mut())?;

    msg!("Agent status updated successfully");
    Ok(())
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
