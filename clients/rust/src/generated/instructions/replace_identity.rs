//! This code was AUTOGENERATED using the codama library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun codama to update it.
//!
//! <https://github.com/codama-idl/codama>
//!

use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use solana_program::pubkey::Pubkey;

/// Accounts.
#[derive(Debug)]
pub struct ReplaceIdentity {
    /// The token extensions mint account linked to the policy
    pub mint: solana_program::pubkey::Pubkey,
    /// The authority over the policy based on token ownership of the mint
    pub token_account: solana_program::pubkey::Pubkey,
    /// The shield policy account
    pub policy: solana_program::pubkey::Pubkey,
    /// The account paying for the storage fees
    pub payer: solana_program::pubkey::Pubkey,
    /// The owner of the token account
    pub owner: solana_program::pubkey::Pubkey,
    /// The system program
    pub system_program: solana_program::pubkey::Pubkey,
}

impl ReplaceIdentity {
    pub fn instruction(
        &self,
        args: ReplaceIdentityInstructionArgs,
    ) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(args, &[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        args: ReplaceIdentityInstructionArgs,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(6 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.mint, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.token_account,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.policy,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.payer, true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.owner, true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.system_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let mut data = borsh::to_vec(&ReplaceIdentityInstructionData::new()).unwrap();
        let mut args = borsh::to_vec(&args).unwrap();
        data.append(&mut args);

        solana_program::instruction::Instruction {
            program_id: crate::SHIELD_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReplaceIdentityInstructionData {
    discriminator: u8,
}

impl ReplaceIdentityInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 3 }
    }
}

impl Default for ReplaceIdentityInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReplaceIdentityInstructionArgs {
    pub index: u64,
    pub identity: Pubkey,
}

/// Instruction builder for `ReplaceIdentity`.
///
/// ### Accounts:
///
///   0. `[]` mint
///   1. `[]` token_account
///   2. `[writable]` policy
///   3. `[writable, signer]` payer
///   4. `[writable, signer]` owner
///   5. `[optional]` system_program (default to `11111111111111111111111111111111`)
#[derive(Clone, Debug, Default)]
pub struct ReplaceIdentityBuilder {
    mint: Option<solana_program::pubkey::Pubkey>,
    token_account: Option<solana_program::pubkey::Pubkey>,
    policy: Option<solana_program::pubkey::Pubkey>,
    payer: Option<solana_program::pubkey::Pubkey>,
    owner: Option<solana_program::pubkey::Pubkey>,
    system_program: Option<solana_program::pubkey::Pubkey>,
    index: Option<u64>,
    identity: Option<Pubkey>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl ReplaceIdentityBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// The token extensions mint account linked to the policy
    #[inline(always)]
    pub fn mint(&mut self, mint: solana_program::pubkey::Pubkey) -> &mut Self {
        self.mint = Some(mint);
        self
    }
    /// The authority over the policy based on token ownership of the mint
    #[inline(always)]
    pub fn token_account(&mut self, token_account: solana_program::pubkey::Pubkey) -> &mut Self {
        self.token_account = Some(token_account);
        self
    }
    /// The shield policy account
    #[inline(always)]
    pub fn policy(&mut self, policy: solana_program::pubkey::Pubkey) -> &mut Self {
        self.policy = Some(policy);
        self
    }
    /// The account paying for the storage fees
    #[inline(always)]
    pub fn payer(&mut self, payer: solana_program::pubkey::Pubkey) -> &mut Self {
        self.payer = Some(payer);
        self
    }
    /// The owner of the token account
    #[inline(always)]
    pub fn owner(&mut self, owner: solana_program::pubkey::Pubkey) -> &mut Self {
        self.owner = Some(owner);
        self
    }
    /// `[optional account, default to '11111111111111111111111111111111']`
    /// The system program
    #[inline(always)]
    pub fn system_program(&mut self, system_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn index(&mut self, index: u64) -> &mut Self {
        self.index = Some(index);
        self
    }
    #[inline(always)]
    pub fn identity(&mut self, identity: Pubkey) -> &mut Self {
        self.identity = Some(identity);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: solana_program::instruction::AccountMeta,
    ) -> &mut Self {
        self.__remaining_accounts.push(account);
        self
    }
    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[solana_program::instruction::AccountMeta],
    ) -> &mut Self {
        self.__remaining_accounts.extend_from_slice(accounts);
        self
    }
    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = ReplaceIdentity {
            mint: self.mint.expect("mint is not set"),
            token_account: self.token_account.expect("token_account is not set"),
            policy: self.policy.expect("policy is not set"),
            payer: self.payer.expect("payer is not set"),
            owner: self.owner.expect("owner is not set"),
            system_program: self
                .system_program
                .unwrap_or(solana_program::pubkey!("11111111111111111111111111111111")),
        };
        let args = ReplaceIdentityInstructionArgs {
            index: self.index.clone().expect("index is not set"),
            identity: self.identity.clone().expect("identity is not set"),
        };

        accounts.instruction_with_remaining_accounts(args, &self.__remaining_accounts)
    }
}

/// `replace_identity` CPI accounts.
pub struct ReplaceIdentityCpiAccounts<'a, 'b> {
    /// The token extensions mint account linked to the policy
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// The authority over the policy based on token ownership of the mint
    pub token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// The shield policy account
    pub policy: &'b solana_program::account_info::AccountInfo<'a>,
    /// The account paying for the storage fees
    pub payer: &'b solana_program::account_info::AccountInfo<'a>,
    /// The owner of the token account
    pub owner: &'b solana_program::account_info::AccountInfo<'a>,
    /// The system program
    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `replace_identity` CPI instruction.
pub struct ReplaceIdentityCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The token extensions mint account linked to the policy
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// The authority over the policy based on token ownership of the mint
    pub token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// The shield policy account
    pub policy: &'b solana_program::account_info::AccountInfo<'a>,
    /// The account paying for the storage fees
    pub payer: &'b solana_program::account_info::AccountInfo<'a>,
    /// The owner of the token account
    pub owner: &'b solana_program::account_info::AccountInfo<'a>,
    /// The system program
    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
    /// The arguments for the instruction.
    pub __args: ReplaceIdentityInstructionArgs,
}

impl<'a, 'b> ReplaceIdentityCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: ReplaceIdentityCpiAccounts<'a, 'b>,
        args: ReplaceIdentityInstructionArgs,
    ) -> Self {
        Self {
            __program: program,
            mint: accounts.mint,
            token_account: accounts.token_account,
            policy: accounts.policy,
            payer: accounts.payer,
            owner: accounts.owner,
            system_program: accounts.system_program,
            __args: args,
        }
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::arithmetic_side_effects)]
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        let mut accounts = Vec::with_capacity(6 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.mint.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.token_account.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.policy.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.payer.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.owner.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.system_program.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let mut data = borsh::to_vec(&ReplaceIdentityInstructionData::new()).unwrap();
        let mut args = borsh::to_vec(&self.__args).unwrap();
        data.append(&mut args);

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::SHIELD_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(7 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.mint.clone());
        account_infos.push(self.token_account.clone());
        account_infos.push(self.policy.clone());
        account_infos.push(self.payer.clone());
        account_infos.push(self.owner.clone());
        account_infos.push(self.system_program.clone());
        remaining_accounts
            .iter()
            .for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

        if signers_seeds.is_empty() {
            solana_program::program::invoke(&instruction, &account_infos)
        } else {
            solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

/// Instruction builder for `ReplaceIdentity` via CPI.
///
/// ### Accounts:
///
///   0. `[]` mint
///   1. `[]` token_account
///   2. `[writable]` policy
///   3. `[writable, signer]` payer
///   4. `[writable, signer]` owner
///   5. `[]` system_program
#[derive(Clone, Debug)]
pub struct ReplaceIdentityCpiBuilder<'a, 'b> {
    instruction: Box<ReplaceIdentityCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> ReplaceIdentityCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(ReplaceIdentityCpiBuilderInstruction {
            __program: program,
            mint: None,
            token_account: None,
            policy: None,
            payer: None,
            owner: None,
            system_program: None,
            index: None,
            identity: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    /// The token extensions mint account linked to the policy
    #[inline(always)]
    pub fn mint(&mut self, mint: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.mint = Some(mint);
        self
    }
    /// The authority over the policy based on token ownership of the mint
    #[inline(always)]
    pub fn token_account(
        &mut self,
        token_account: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.token_account = Some(token_account);
        self
    }
    /// The shield policy account
    #[inline(always)]
    pub fn policy(
        &mut self,
        policy: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.policy = Some(policy);
        self
    }
    /// The account paying for the storage fees
    #[inline(always)]
    pub fn payer(&mut self, payer: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.payer = Some(payer);
        self
    }
    /// The owner of the token account
    #[inline(always)]
    pub fn owner(&mut self, owner: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.owner = Some(owner);
        self
    }
    /// The system program
    #[inline(always)]
    pub fn system_program(
        &mut self,
        system_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.system_program = Some(system_program);
        self
    }
    #[inline(always)]
    pub fn index(&mut self, index: u64) -> &mut Self {
        self.instruction.index = Some(index);
        self
    }
    #[inline(always)]
    pub fn identity(&mut self, identity: Pubkey) -> &mut Self {
        self.instruction.identity = Some(identity);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: &'b solana_program::account_info::AccountInfo<'a>,
        is_writable: bool,
        is_signer: bool,
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .push((account, is_writable, is_signer));
        self
    }
    /// Add additional accounts to the instruction.
    ///
    /// Each account is represented by a tuple of the `AccountInfo`, a `bool` indicating whether the account is writable or not,
    /// and a `bool` indicating whether the account is a signer or not.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed(&[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        let args = ReplaceIdentityInstructionArgs {
            index: self.instruction.index.clone().expect("index is not set"),
            identity: self
                .instruction
                .identity
                .clone()
                .expect("identity is not set"),
        };
        let instruction = ReplaceIdentityCpi {
            __program: self.instruction.__program,

            mint: self.instruction.mint.expect("mint is not set"),

            token_account: self
                .instruction
                .token_account
                .expect("token_account is not set"),

            policy: self.instruction.policy.expect("policy is not set"),

            payer: self.instruction.payer.expect("payer is not set"),

            owner: self.instruction.owner.expect("owner is not set"),

            system_program: self
                .instruction
                .system_program
                .expect("system_program is not set"),
            __args: args,
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct ReplaceIdentityCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    mint: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    token_account: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    policy: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    payer: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    owner: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    index: Option<u64>,
    identity: Option<Pubkey>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
