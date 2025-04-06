mod generated;

use bytemuck::PodCastError;
pub use generated::programs::SHIELD_ID as ID;
pub use generated::*;
use solana_program::pubkey::Pubkey;
use solana_sdk::{rent::Rent, signer::keypair::Keypair, transaction::Transaction};
use std::str::FromStr;

#[cfg(feature = "token-extensions")]
use spl_associated_token_account::instruction::create_associated_token_account;

#[cfg(feature = "token-extensions")]
use spl_token_2022::{
    extension::metadata_pointer::instruction::initialize as initialize_metadata_pointer,
    instruction::{initialize_mint2, mint_to},
    ID as TOKEN_22_PROGRAM_ID,
};

#[cfg(feature = "token-extensions")]
use spl_token_metadata_interface::instruction::initialize as initialize_metadata;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid permission strategy")]
    InvalidStrategy,
    #[error("Invalid kind")]
    InvalidKind,
    #[error("Invalid PodCastError")]
    InvalidPodCastError,
}

impl FromStr for generated::types::PermissionStrategy {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "allow" => Ok(generated::types::PermissionStrategy::Allow),
            "deny" => Ok(generated::types::PermissionStrategy::Deny),
            _ => Err(ParseError::InvalidStrategy),
        }
    }
}

impl TryFrom<u8> for generated::types::PermissionStrategy {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(generated::types::PermissionStrategy::Deny),
            1 => Ok(generated::types::PermissionStrategy::Allow),
            _ => Err(ParseError::InvalidStrategy),
        }
    }
}

impl From<PodCastError> for ParseError {
    fn from(_: PodCastError) -> Self {
        Self::InvalidPodCastError
    }
}

impl TryFrom<u8> for generated::types::Kind {
    type Error = ParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(generated::types::Kind::Policy),
            _ => Err(ParseError::InvalidKind),
        }
    }
}

impl generated::accounts::Policy {
    pub fn identities_len(&self) -> u32 {
        u32::from_le_bytes(self.identities_len)
    }

    pub fn try_deserialize_identities(data: &[u8]) -> Result<Vec<Pubkey>, ParseError> {
        Ok(bytemuck::try_cast_slice(data)
            .map_err(ParseError::from)?
            .to_vec())
    }

    pub fn try_kind(&self) -> Result<generated::types::Kind, ParseError> {
        generated::types::Kind::try_from(self.kind)
    }
    pub fn try_strategy(&self) -> Result<generated::types::PermissionStrategy, ParseError> {
        generated::types::PermissionStrategy::try_from(self.strategy)
    }
}

/// Instruction builder for creating a solana account.
///
/// ### Accounts:
///
///   0. `[signer]` payer
///   1. `[writable]` mint
///   2. `[optional]` system_program (default to `11111111111111111111111111111111`)
#[derive(Clone, Debug, Default)]
pub struct CreateAccountBuilder<'a> {
    payer: Option<&'a Pubkey>,
    account: Option<&'a Pubkey>,
    space: Option<usize>,
    owner: Option<&'a Pubkey>,
    rent: Option<usize>,
}

impl<'a> CreateAccountBuilder<'a> {
    pub fn build() -> Self {
        Self::default()
    }

    /// The account paying for the storage fees
    #[inline(always)]
    pub fn payer(&mut self, payer: &'a Pubkey) -> &mut Self {
        self.payer = Some(payer);
        self
    }

    /// The account to be created
    #[inline(always)]
    pub fn account(&mut self, account: &'a Pubkey) -> &mut Self {
        self.account = Some(account);
        self
    }

    /// The space required for the account
    #[inline(always)]
    pub fn space(&mut self, space: usize) -> &mut Self {
        self.space = Some(space);
        self
    }

    /// The program to own the account
    #[inline(always)]
    pub fn owner(&mut self, owner: &'a Pubkey) -> &mut Self {
        self.owner = Some(owner);
        self
    }

    /// The rent to be paid for the account
    #[inline(always)]
    pub fn rent(&mut self, rent: usize) -> &mut Self {
        self.rent = Some(rent);
        self
    }

    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let space = self.space.expect("space is not set");
        let lamports = Rent::default().minimum_balance(self.rent.expect("rent is not set"));

        solana_sdk::system_instruction::create_account(
            self.payer.expect("payer is not set"),
            self.account.expect("mint is not set"),
            lamports,
            u64::try_from(space).expect("space is too large"),
            self.owner.expect("owner is not set"),
        )
    }
}

#[cfg(feature = "token-extensions")]
pub struct InitializeMint2Builder<'a> {
    mint: Option<&'a Pubkey>,
    mint_authority: Option<&'a Pubkey>,
    freeze_authority: Option<&'a Pubkey>,
    token_program: Option<&'a Pubkey>,
}

#[cfg(feature = "token-extensions")]
impl<'a> InitializeMint2Builder<'a> {
    pub fn build() -> Self {
        Self {
            mint: None,
            mint_authority: None,
            freeze_authority: None,
            token_program: None,
        }
    }

    /// The mint account to be initialized
    #[inline(always)]
    pub fn mint(&mut self, mint: &'a Pubkey) -> &mut Self {
        self.mint = Some(mint);
        self
    }

    /// The authority that can mint new tokens
    #[inline(always)]
    pub fn mint_authority(&mut self, mint_authority: &'a Pubkey) -> &mut Self {
        self.mint_authority = Some(mint_authority);
        self
    }

    /// The authority that can freeze token accounts
    #[inline(always)]
    pub fn freeze_authority(&mut self, freeze_authority: &'a Pubkey) -> &mut Self {
        self.freeze_authority = Some(freeze_authority);
        self
    }

    /// The token program
    #[inline(always)]
    pub fn token_program(&mut self, token_program: &'a Pubkey) -> &mut Self {
        self.token_program = Some(token_program);
        self
    }

    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        initialize_mint2(
            self.token_program.unwrap_or(&TOKEN_22_PROGRAM_ID),
            self.mint.expect("mint is not set"),
            self.mint_authority.expect("mint_authority is not set"),
            self.freeze_authority,
            0,
        )
        .expect("Failed to create initialize_mint2 instruction")
    }
}

#[cfg(feature = "token-extensions")]
pub struct MetadataPointerInitializeBuilder<'a> {
    token_program: Option<&'a Pubkey>,
    mint: Option<&'a Pubkey>,
    metadata: Option<Pubkey>,
    authority: Option<Pubkey>,
}

#[cfg(feature = "token-extensions")]
impl<'a> MetadataPointerInitializeBuilder<'a> {
    pub fn build() -> Self {
        Self {
            token_program: None,
            mint: None,
            metadata: None,
            authority: None,
        }
    }

    /// The token program
    #[inline(always)]
    pub fn token_program(&mut self, token_program: &'a Pubkey) -> &mut Self {
        self.token_program = Some(token_program);
        self
    }

    /// The mint account
    #[inline(always)]
    pub fn mint(&mut self, mint: &'a Pubkey) -> &mut Self {
        self.mint = Some(mint);
        self
    }

    /// The metadata account
    #[inline(always)]
    pub fn metadata(&mut self, metadata: Pubkey) -> &mut Self {
        self.metadata = Some(metadata);
        self
    }

    /// The metadata account
    #[inline(always)]
    pub fn authority(&mut self, authority: Pubkey) -> &mut Self {
        self.authority = Some(authority);
        self
    }

    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        initialize_metadata_pointer(
            self.token_program.unwrap_or(&TOKEN_22_PROGRAM_ID),
            self.mint.expect("mint is not set"),
            self.authority,
            self.metadata,
        )
        .expect("Failed to create metadata pointer initialize instruction")
    }
}

#[cfg(feature = "token-extensions")]
pub struct CreateAsscoiatedTokenAccountBuilder<'a> {
    token_program: Option<&'a Pubkey>,
    mint: Option<&'a Pubkey>,
    owner: Option<&'a Pubkey>,
    payer: Option<&'a Pubkey>,
}

#[cfg(feature = "token-extensions")]
impl<'a> CreateAsscoiatedTokenAccountBuilder<'a> {
    pub fn build() -> Self {
        Self {
            token_program: None,
            mint: None,
            owner: None,
            payer: None,
        }
    }

    #[inline(always)]
    pub fn payer(&mut self, payer: &'a Pubkey) -> &mut Self {
        self.payer = Some(payer);
        self
    }

    /// The token program
    #[inline(always)]
    pub fn token_program(&mut self, token_program: &'a Pubkey) -> &mut Self {
        self.token_program = Some(token_program);
        self
    }

    /// The mint account
    #[inline(always)]
    pub fn mint(&mut self, mint: &'a Pubkey) -> &mut Self {
        self.mint = Some(mint);
        self
    }

    /// The owner of the account
    #[inline(always)]
    pub fn owner(&mut self, owner: &'a Pubkey) -> &mut Self {
        self.owner = Some(owner);
        self
    }

    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let owner = self.owner.expect("owner is not set");
        let mint = self.mint.expect("mint is not set");
        let payer = self.payer.expect("payer is not set");
        let token_program = self.token_program.unwrap_or(&TOKEN_22_PROGRAM_ID);

        create_associated_token_account(payer, owner, mint, token_program)
    }
}

#[cfg(feature = "token-extensions")]
pub struct TokenExtensionsMintToBuilder<'a> {
    token_program: Option<&'a Pubkey>,
    mint: Option<&'a Pubkey>,
    account: Option<&'a Pubkey>,
    owner: Option<&'a Pubkey>,
    signers: Vec<&'a Pubkey>,
    amount: u64,
}

#[cfg(feature = "token-extensions")]
impl<'a> TokenExtensionsMintToBuilder<'a> {
    pub fn build() -> Self {
        Self {
            token_program: None,
            mint: None,
            account: None,
            owner: None,
            signers: Vec::new(),
            amount: 1,
        }
    }

    /// The token program
    #[inline(always)]
    pub fn token_program(&mut self, token_program: &'a Pubkey) -> &mut Self {
        self.token_program = Some(token_program);
        self
    }

    /// The mint account
    #[inline(always)]
    pub fn mint(&mut self, mint: &'a Pubkey) -> &mut Self {
        self.mint = Some(mint);
        self
    }

    /// The account to mint to
    #[inline(always)]
    pub fn account(&mut self, account: &'a Pubkey) -> &mut Self {
        self.account = Some(account);
        self
    }

    /// The owner of the account
    #[inline(always)]
    pub fn owner(&mut self, owner: &'a Pubkey) -> &mut Self {
        self.owner = Some(owner);
        self
    }

    /// Add a signer
    #[inline(always)]
    pub fn add_signer(&mut self, signer: &'a Pubkey) -> &mut Self {
        self.signers.push(signer);
        self
    }

    /// The amount to mint
    #[inline(always)]
    pub fn amount(&mut self, amount: u64) -> &mut Self {
        self.amount = amount;
        self
    }

    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        mint_to(
            self.token_program.unwrap_or(&TOKEN_22_PROGRAM_ID),
            self.mint.expect("mint is not set"),
            self.account.expect("account is not set"),
            self.owner.expect("owner is not set"),
            &self.signers,
            self.amount,
        )
        .expect("Failed to create mint to instruction")
    }
}

pub struct TransactionBuilder<'a> {
    instructions: Vec<solana_program::instruction::Instruction>,
    signers: Vec<&'a Keypair>,
    payer: Option<&'a Pubkey>,
    recent_blockhash: Option<solana_sdk::hash::Hash>,
}

impl<'a> TransactionBuilder<'a> {
    pub fn build() -> Self {
        Self {
            instructions: Vec::new(),
            signers: Vec::new(),
            payer: None,
            recent_blockhash: None,
        }
    }

    /// Set the entire list of instructions for the transaction
    #[inline(always)]
    pub fn instructions(
        &mut self,
        instructions: Vec<solana_program::instruction::Instruction>,
    ) -> &mut Self {
        self.instructions = instructions;
        self
    }

    /// Add an instruction to the transaction
    #[inline(always)]
    pub fn instruction(
        &mut self,
        instruction: solana_program::instruction::Instruction,
    ) -> &mut Self {
        self.instructions.push(instruction);
        self
    }

    /// Add a signer to the transaction
    #[inline(always)]
    pub fn signer(&mut self, signer: &'a Keypair) -> &mut Self {
        self.signers.push(signer);
        self
    }

    /// Set the payer for the transaction
    #[inline(always)]
    pub fn payer(&mut self, payer: &'a Pubkey) -> &mut Self {
        self.payer = Some(payer);
        self
    }

    /// Set the recent blockhash for the transaction
    #[inline(always)]
    pub fn recent_blockhash(&mut self, recent_blockhash: solana_sdk::hash::Hash) -> &mut Self {
        self.recent_blockhash = Some(recent_blockhash);
        self
    }

    /// Build the transaction
    pub fn transaction(&self) -> Transaction {
        Transaction::new_signed_with_payer(
            &self.instructions,
            self.payer,
            &self.signers,
            self.recent_blockhash.expect("recent blockhash is not set"),
        )
    }
}

#[cfg(feature = "token-extensions")]
pub struct InitializeMetadataBuilder<'a> {
    token_program: Option<&'a Pubkey>,
    mint: Option<&'a Pubkey>,
    owner: Option<&'a Pubkey>,
    update_authority: Option<&'a Pubkey>,
    mint_authority: Option<&'a Pubkey>,
    name: Option<String>,
    symbol: Option<String>,
    uri: Option<String>,
}

#[cfg(feature = "token-extensions")]
impl Default for InitializeMetadataBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "token-extensions")]
impl<'a> InitializeMetadataBuilder<'a> {
    pub fn new() -> Self {
        Self {
            token_program: None,
            mint: None,
            owner: None,
            update_authority: None,
            mint_authority: None,
            name: None,
            symbol: None,
            uri: None,
        }
    }

    pub fn token_program(&mut self, token_program: &'a Pubkey) -> &mut Self {
        self.token_program = Some(token_program);
        self
    }

    pub fn mint(&mut self, mint: &'a Pubkey) -> &mut Self {
        self.mint = Some(mint);
        self
    }

    pub fn owner(&mut self, owner: &'a Pubkey) -> &mut Self {
        self.owner = Some(owner);
        self
    }

    pub fn update_authority(&mut self, update_authority: &'a Pubkey) -> &mut Self {
        self.update_authority = Some(update_authority);
        self
    }

    pub fn mint_authority(&mut self, mint_authority: &'a Pubkey) -> &mut Self {
        self.mint_authority = Some(mint_authority);
        self
    }

    pub fn name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn symbol(&mut self, symbol: String) -> &mut Self {
        self.symbol = Some(symbol);
        self
    }

    pub fn uri(&mut self, uri: String) -> &mut Self {
        self.uri = Some(uri);
        self
    }

    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        initialize_metadata(
            self.token_program.unwrap_or(&TOKEN_22_PROGRAM_ID),
            self.mint.expect("mint_pubkey is not set"),
            self.update_authority
                .expect("update_authority_pubkey is not set"),
            self.mint.expect("mint_pubkey is not set"),
            self.mint_authority.expect("mint_authority is not set"),
            self.name.as_ref().expect("name is not set").clone(),
            self.symbol.as_ref().expect("symbol is not set").clone(),
            self.uri.as_ref().expect("uri is not set").clone(),
        )
    }
}
