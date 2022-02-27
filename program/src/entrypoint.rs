//! Program entrypoint
//! The entrypoint forwards arguments to processor

use solana_program::{
    account_info::AccountInfo, 
    entrypoint, 
    entrypoint::ProgramResult, 
    pubkey::Pubkey,
};

use crate::processor::Processor;

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Processor::process(program_id, accounts, instruction_data)
}


// Register the entrypoint
entrypoint!(process_instruction);