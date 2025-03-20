mod generated;

pub use generated::programs::BLOCKLIST_ID as ID;
pub use generated::*;
use solana_program::pubkey::Pubkey;
use solana_sdk::{rent::Rent, signer::keypair::Keypair, transaction::Transaction};
use spl_token_2022::{extension::metadata_pointer, instruction::mint_to};

pub trait Size {
    const BASE_SIZE: usize;
    fn size(&self) -> usize;
}

impl Size for generated::accounts::Policy {
    // The base size of the Policy struct is 5 bytes.
    const BASE_SIZE: usize = 6;

    fn size(&self) -> usize {
        Self::BASE_SIZE + (self.validator_identities.len() * std::mem::size_of::<Pubkey>())
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

    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let space = self.space.expect("space is not set");
        let lamports = Rent::default().minimum_balance(space);

        solana_program::system_instruction::create_account(
            self.payer.expect("payer is not set"),
            self.account.expect("mint is not set"),
            lamports,
            u64::try_from(space).expect("space is too large"),
            self.owner.expect("owner is not set"),
        )
    }
}

pub struct InitializeMint2Builder<'a> {
    mint: Option<&'a Pubkey>,
    mint_authority: Option<&'a Pubkey>,
    freeze_authority: Option<&'a Pubkey>,
    token_program: Option<&'a Pubkey>,
}

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
        spl_token_2022::instruction::initialize_mint2(
            self.token_program.unwrap_or(&spl_token_2022::ID),
            self.mint.expect("mint is not set"),
            self.mint_authority.expect("mint_authority is not set"),
            self.freeze_authority,
            0,
        )
        .expect("Failed to create initialize_mint2 instruction")
    }
}

pub struct MetadataPointerInitializeBuilder<'a> {
    token_program: Option<&'a Pubkey>,
    mint: Option<&'a Pubkey>,
    payer: Option<&'a Pubkey>,
    metadata: Option<Pubkey>,
    authority: Option<Pubkey>,
}

impl<'a> MetadataPointerInitializeBuilder<'a> {
    pub fn build() -> Self {
        Self {
            token_program: None,
            mint: None,
            payer: None,
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

    /// The payer account
    #[inline(always)]
    pub fn payer(&mut self, payer: &'a Pubkey) -> &mut Self {
        self.payer = Some(payer);
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
        metadata_pointer::instruction::initialize(
            self.token_program.unwrap_or(&spl_token_2022::ID),
            self.mint.expect("mint is not set"),
            self.authority,
            self.metadata,
        )
        .expect("Failed to create metadata pointer initialize instruction")
    }
}

pub struct InitializeTokenExtensionsAccountBuilder<'a> {
    token_program: Option<&'a Pubkey>,
    account: Option<&'a Pubkey>,
    mint: Option<&'a Pubkey>,
    owner: Option<&'a Pubkey>,
}

impl<'a> InitializeTokenExtensionsAccountBuilder<'a> {
    pub fn build() -> Self {
        Self {
            token_program: None,
            account: None,
            mint: None,
            owner: None,
        }
    }

    /// The token program
    #[inline(always)]
    pub fn token_program(&mut self, token_program: &'a Pubkey) -> &mut Self {
        self.token_program = Some(token_program);
        self
    }

    /// The account to initialize
    #[inline(always)]
    pub fn account(&mut self, account: &'a Pubkey) -> &mut Self {
        self.account = Some(account);
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
        spl_token_2022::instruction::initialize_account(
            self.token_program.unwrap_or(&spl_token_2022::id()),
            self.account.expect("account is not set"),
            self.mint.expect("mint is not set"),
            self.owner.expect("owner is not set"),
        )
        .expect("Failed to create initialize account instruction")
    }
}

pub struct TokenExtensionsMintToBuilder<'a> {
    token_program: Option<&'a Pubkey>,
    mint: Option<&'a Pubkey>,
    account: Option<&'a Pubkey>,
    owner: Option<&'a Pubkey>,
    signers: Vec<&'a Pubkey>,
    amount: u64,
}

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
            self.token_program.unwrap_or(&spl_token_2022::id()),
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
