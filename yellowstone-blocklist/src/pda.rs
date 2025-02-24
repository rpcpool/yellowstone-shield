use pinocchio::{
    msg,
    program_error::ProgramError,
    pubkey::{find_program_address, Pubkey},
};

use crate::error::ConfigErrors;

pub fn find_pda(program_id: &Pubkey, key: &Pubkey) -> (Pubkey, u8) {
    find_program_address(&[key.as_ref(), "noneknows".as_bytes()], program_id)
}

pub fn check_pda(
    program_id: &Pubkey,
    key: &Pubkey,
    pda_key_acc: &Pubkey,
) -> Result<(Pubkey, u8), ProgramError> {
    let (pda_key, bump_seed) = find_pda(program_id, key);
    if pda_key.ne(pda_key_acc) {
        msg!("Invalid PDA account");
        return Err(ConfigErrors::InvalidPDA.into());
    }
    Ok((pda_key, bump_seed))
}
