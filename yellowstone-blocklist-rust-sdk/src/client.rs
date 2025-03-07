use std::str::FromStr;

use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::{
    error::BlocklistResult,
    instruction::{
        AclPayload, AddListPayload, ConfigInstructions, DeleteListPayload, IndexPubkey,
        InitializeListPayload,
    },
    state::AclType,
    BLOCKLIST_PROGRAM_ID,
};

/// Client for interacting with the Yellowstone Blocklist program
pub struct BlocklistClient {
    program_id: Pubkey,
}

impl BlocklistClient {
    /// Creates a new BlocklistClient instance
    ///
    /// # Arguments
    ///
    /// * `program_id` - Optional custom program ID. If None, uses the default program ID
    pub fn new(program_id: Option<Pubkey>) -> Self {
        let program_id = program_id.unwrap_or_else(|| {
            Pubkey::from_str(BLOCKLIST_PROGRAM_ID).unwrap_or_else(|_| panic!("Invalid program ID"))
        });
        Self { program_id }
    }

    /// Get program ID
    pub fn program_id(&self) -> &Pubkey {
        &self.program_id
    }

    /// Get PDA for a given authority
    pub fn get_blocklist_pda(&self, authority: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[authority.as_ref(), b"noneknows"], &self.program_id)
    }

    /// Creates instruction for initializing a new blocklist
    ///
    /// # Arguments
    ///
    /// * `payer` - The account that will pay for the initialization
    /// * `authority` - Optional authority that will control the list. Defaults to payer if None
    /// * `acl_type` - Type of access control list (Allow or Deny)
    pub fn create_initialize_instruction(
        &self,
        payer: &Pubkey,
        authority: Option<Pubkey>,
        acl_type: AclType,
    ) -> BlocklistResult<(Instruction, Pubkey)> {
        let authority = authority.unwrap_or(*payer);
        let (pda, _) =
            Pubkey::find_program_address(&[authority.as_ref(), b"noneknows"], &self.program_id);

        let accounts = vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new(authority, true),
        ];

        let instruction = Instruction::new_with_borsh(
            self.program_id,
            &ConfigInstructions::InitializeList(InitializeListPayload { acl_type }),
            accounts,
        );

        Ok((instruction, pda))
    }

    /// Creates instruction to add pubkeys to the blocklist
    ///
    /// # Arguments
    ///
    /// * `payer` - Account paying for transaction fees
    /// * `authority` - The authority of the blocklist
    /// * `list_pda` - PDA of the blocklist account
    /// * `pubkey_list` - List of pubkeys and their indices to add
    pub fn create_add_instruction(
        &self,
        payer: &Pubkey,
        authority: &Pubkey,
        list_pda: &Pubkey,
        pubkey_list: Vec<IndexPubkey>,
    ) -> BlocklistResult<Instruction> {
        let accounts = vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*list_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new(*authority, true),
        ];

        let instruction = Instruction::new_with_borsh(
            self.program_id,
            &ConfigInstructions::Add(AddListPayload { list: pubkey_list }),
            accounts,
        );

        Ok(instruction)
    }

    /// Creates instruction to remove pubkeys from the blocklist
    ///
    /// # Arguments
    ///
    /// * `payer` - Account paying for transaction fees
    /// * `authority` - The authority of the blocklist
    /// * `list_pda` - PDA of the blocklist account
    /// * `indices` - List of indices to remove
    pub fn create_remove_instruction(
        &self,
        payer: &Pubkey,
        authority: &Pubkey,
        list_pda: &Pubkey,
        indices: Vec<usize>,
    ) -> BlocklistResult<Instruction> {
        let accounts = vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*list_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new(*authority, true),
        ];

        let instruction = Instruction::new_with_borsh(
            self.program_id,
            &ConfigInstructions::RemoveItemList(DeleteListPayload { vec_index: indices }),
            accounts,
        );

        Ok(instruction)
    }

    /// Creates instruction to update the ACL type of the blocklist
    ///
    /// # Arguments
    ///
    /// * `payer` - Account paying for transaction fees
    /// * `authority` - The authority of the blocklist
    /// * `list_pda` - PDA of the blocklist account
    /// * `acl_type` - New ACL type (Allow or Deny)
    pub fn create_update_acl_type_instruction(
        &self,
        payer: &Pubkey,
        authority: &Pubkey,
        list_pda: &Pubkey,
        acl_type: AclType,
    ) -> BlocklistResult<Instruction> {
        let accounts = vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*list_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new(*authority, true),
        ];

        let instruction = Instruction::new_with_borsh(
            self.program_id,
            &ConfigInstructions::UpdateAclType(AclPayload { acl_type }),
            accounts,
        );

        Ok(instruction)
    }

    /// Creates instruction to freeze the blocklist account, making it immutable
    ///
    /// # Arguments
    ///
    /// * `payer` - Account paying for transaction fees
    /// * `authority` - The authority of the blocklist
    /// * `list_pda` - PDA of the blocklist account
    pub fn create_freeze_instruction(
        &self,
        payer: &Pubkey,
        authority: &Pubkey,
        list_pda: &Pubkey,
    ) -> BlocklistResult<Instruction> {
        let accounts = vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*list_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new(*authority, true),
        ];

        let instruction = Instruction::new_with_borsh(
            self.program_id,
            &ConfigInstructions::FreezeAccount,
            accounts,
        );

        Ok(instruction)
    }

    /// Creates instruction to close the blocklist account and recover rent
    ///
    /// # Arguments
    ///
    /// * `payer` - Account paying for transaction fees
    /// * `authority` - The authority of the blocklist
    /// * `list_pda` - PDA of the blocklist account
    /// * `destination` - Account to receive the recovered lamports
    pub fn create_close_instruction(
        &self,
        payer: &Pubkey,
        authority: &Pubkey,
        list_pda: &Pubkey,
        destination: &Pubkey,
    ) -> BlocklistResult<Instruction> {
        let accounts = vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*list_pda, false),
            AccountMeta::new(*destination, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new(*authority, true),
        ];

        let instruction = Instruction::new_with_borsh(
            self.program_id,
            &ConfigInstructions::CloseAccount,
            accounts,
        );

        Ok(instruction)
    }
}

impl Default for BlocklistClient {
    fn default() -> Self {
        Self::new(None)
    }
}
