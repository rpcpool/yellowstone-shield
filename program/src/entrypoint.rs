#![allow(unexpected_cfgs)]

use crate::processor;
use pinocchio::{account_info::AccountInfo, entrypoint, msg, pubkey::Pubkey, ProgramResult};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(e) = processor::process_instruction(program_id, accounts, instruction_data) {
        msg!("Error processing instruction: {:?}", e);
        return Err(e);
    }

    Ok(())
}
