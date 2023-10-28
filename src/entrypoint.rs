#![cfg(all(target_os = "solana", not(feature = "no-entrypoint")))]
use crate::processor::Processor;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

entrypoint!(process_instruction);

/// Program entrypoint for processing instructions.
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: &[u8],
) -> ProgramResult {
    Processor::process_instruction(program_id, accounts, instruction)
}
