use crate::{error::ConfigErrors, instruction::ConfigInstructions, state::ConfigListState};
use borsh::BorshSerialize;
// use solana_config_program::{ConfigKeys, ConfigState};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh1::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = ConfigInstructions::unpack(instruction_data)?;

    match instruction {
        ConfigInstructions::AddOrEditBlocklist { pubkey, blocklist } => {
            add_or_edit_blocklist(program_id, accounts, pubkey, blocklist)?
        }
        ConfigInstructions::DeleteBlocklist { pubkey } => {
            delete_blocklist(program_id, accounts, pubkey)?
        }
        ConfigInstructions::InitializeAccount => initialize_account(program_id, accounts)?,
    }

    Ok(())
}

fn add_or_edit_blocklist(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    pubkey: String,
    blocklist: Vec<String>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let initializer = next_account_info(accounts_iter)?;
    let pda_config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (pda_key, _bump_seed) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), "noneknows".as_bytes()],
        program_id,
    );

    if &pda_key != pda_config_account.key {
        return Err(ConfigErrors::InvalidPDA.into());
    }

    let mut account_data =
        try_from_slice_unchecked::<ConfigListState>(&pda_config_account.data.borrow())?;
    account_data.blocklists.insert(pubkey, blocklist);

    account_data.serialize(&mut &mut pda_config_account.data.borrow_mut()[..])?;

    Ok(())
}

fn delete_blocklist(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    pubkey: String,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let initializer = next_account_info(accounts_iter)?;
    let pda_config_account: &AccountInfo<'_> = next_account_info(accounts_iter)?;

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (pda_key, _bump_seed) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), "noneknows".as_bytes()],
        program_id,
    );

    if &pda_key != pda_config_account.key {
        return Err(ConfigErrors::InvalidPDA.into());
    }

    let mut account_data =
        try_from_slice_unchecked::<ConfigListState>(&pda_config_account.data.borrow())?;
    account_data.blocklists.remove(&pubkey);

    account_data.serialize(&mut &mut pda_config_account.data.borrow_mut()[..])?;

    Ok(())
}

fn initialize_account(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let initializer = next_account_info(accounts_iter)?;
    let pda_config_account = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (pda_key, bump_seed) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), "noneknows".as_bytes()],
        program_id,
    );

    if &pda_key != pda_config_account.key {
        return Err(ConfigErrors::InvalidPDA.into());
    }
    let space = ConfigListState::max_space();
    let rent = get_rent(&(space as usize))?;

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_config_account.key,
            rent,
            space,
            program_id,
        ),
        &[
            initializer.clone(),
            pda_config_account.clone(),
            system_account.clone(),
        ],
        &[&[
            initializer.key.as_ref(),
            "noneknows".as_bytes(),
            &[bump_seed],
        ]],
    )?;

    msg!("Config account created");

    Ok(())
}

fn get_rent(account_len: &usize) -> Result<u64, ProgramError> {
    let rent = Rent::get()?;
    Ok(rent.minimum_balance(*account_len))
}
