use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    program::invoke,
    sysvar::{rent::Rent, Sysvar},
};
use borsh::{BorshDeserialize, BorshSerialize};

use crate::state::{AIAgent, ComputeTask, AIInfraInstruction, TaskStatus};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = AIInfraInstruction::try_from_slice(instruction_data)?;
        
        match instruction {
            AIInfraInstruction::RegisterAgent => {
                Self::process_register_agent(program_id, accounts)
            }
            AIInfraInstruction::CreateTask { requirements, payment_amount } => {
                Self::process_create_task(program_id, accounts, requirements, payment_amount)
            }
            // ... Add other instruction handlers
        }
    }

    fn process_register_agent(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        // Your existing register_agent implementation
        Ok(())
    }

    fn process_create_task(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        requirements: ComputeRequirements,
        payment_amount: u64,
    ) -> ProgramResult {
        // Your existing create_task implementation
        Ok(())
    }
    
    // ... Add other process functions
}