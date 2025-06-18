use std::collections::{HashSet, VecDeque};

use super::{CommandComplete, RunCommand, RunResult, SolanaAccount};
use crate::{
    command::{send_batched_tx, CommandContext},
    policy::PolicyVersion,
};
use borsh::BorshDeserialize;

use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::{
    extension::{BaseStateWithExtensions, PodStateWithExtensions},
    pod::PodMint,
};
use spl_token_metadata_interface::{
    borsh::BorshDeserialize as TokenBorshDeserialize, state::TokenMetadata,
};

use yellowstone_shield_client::{
    accounts::{Policy, PolicyV2},
    instructions::ReplaceIdentityBuilder,
    types::Kind,
};
use yellowstone_shield_client::{
    instructions::{AddIdentityBuilder, RemoveIdentityBuilder},
    PolicyTrait,
};

const CHUNK_SIZE: usize = 20;

/// Builder for adding a identities to a policy
#[derive(Debug, Clone)]
pub struct AddBatchCommandBuilder<'a> {
    mint: Option<&'a Pubkey>,
    identities: Option<Vec<Pubkey>>,
}

impl Default for AddBatchCommandBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> AddBatchCommandBuilder<'a> {
    /// Create a new AddCommandBuilder
    pub fn new() -> Self {
        Self {
            mint: None,
            identities: None,
        }
    }

    /// Set the mint address
    pub fn mint(mut self, mint: &'a Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the identities to add
    pub fn identities(mut self, identities: Vec<Pubkey>) -> Self {
        self.identities = Some(identities);
        self
    }
}

#[async_trait::async_trait]
impl RunCommand for AddBatchCommandBuilder<'_> {
    /// Execute the addition of a identity to the policy
    async fn run(&mut self, context: CommandContext) -> RunResult {
        let CommandContext { keypair, client } = context;

        let mint = self.mint.expect("mint must be set");

        // PDA seeds are same for both Policy and PolicyV2
        let (address, _) = Policy::find_pda(mint);
        let identities = self.identities.take().expect("identities must be set");
        let token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            mint,
            &spl_token_2022::ID,
        );

        let account_data = client.get_account(&address).await?;
        let account_data: &[u8] = &account_data.data;

        let policy_version = Kind::try_from_slice(&[account_data[0]])?;

        let current = match policy_version {
            Kind::Policy => Policy::try_deserialize_identities(account_data),
            Kind::PolicyV2 => PolicyV2::try_deserialize_identities(account_data),
        }?;

        let add: Vec<Pubkey> = identities
            .into_iter()
            .filter(|identity| !current.contains(identity))
            .collect();

        send_batched_tx(&client, &keypair, &add, CHUNK_SIZE, |identity| {
            AddIdentityBuilder::new()
                .policy(address)
                .mint(*mint)
                .token_account(token_account)
                .payer(keypair.pubkey())
                .owner(keypair.pubkey())
                .identity(*identity)
                .instruction()
        })
        .await?;

        let account_data = client.get_account(&address).await?;
        let account_data: &[u8] = &account_data.data;

        let policy_version = Kind::try_from_slice(&[account_data[0]])?;

        let policy = match policy_version {
            Kind::Policy => PolicyVersion::V1(Policy::from_bytes(&account_data[..Policy::LEN])?),
            Kind::PolicyV2 => {
                PolicyVersion::V2(PolicyV2::from_bytes(&account_data[..PolicyV2::LEN])?)
            }
        };

        let mint_data = client.get_account(mint).await?;
        let account_data: &[u8] = &mint_data.data;

        let mint_pod = PodStateWithExtensions::<PodMint>::unpack(account_data).unwrap();
        let mint_bytes = mint_pod.get_extension_bytes::<TokenMetadata>().unwrap();
        let token_metadata = TokenMetadata::try_from_slice(mint_bytes).unwrap();

        Ok(CommandComplete(
            SolanaAccount(*mint, Some(token_metadata)),
            SolanaAccount(address, Some(policy)),
        ))
    }
}

/// Builder for updating/replacing identities in a policy
pub struct UpdateBatchCommandBuilder<'a> {
    mint: Option<&'a Pubkey>,
    identities: Option<Vec<Pubkey>>,
}

impl Default for UpdateBatchCommandBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> UpdateBatchCommandBuilder<'a> {
    /// Create a new UpdateCommandBuilder
    pub fn new() -> Self {
        Self {
            mint: None,
            identities: None,
        }
    }

    /// Set the mint address
    pub fn mint(mut self, mint: &'a Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the identities to replace/update
    pub fn identities(mut self, identities: Vec<Pubkey>) -> Self {
        self.identities = Some(identities);
        self
    }
}

#[async_trait::async_trait]
impl RunCommand for UpdateBatchCommandBuilder<'_> {
    /// Execute replace/update of identities
    async fn run(&mut self, context: CommandContext) -> RunResult {
        let CommandContext { keypair, client } = context;

        let mint = self.mint.expect("mint must be set");

        // PDA seeds are same for both Policy and PolicyV2
        let (address, _) = Policy::find_pda(mint);

        let identities = self
            .identities
            .take()
            .expect("identity_with_index must be set");

        let token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            mint,
            &spl_token_2022::ID,
        );

        let account_data = client.get_account(&address).await?;
        let account_data: &[u8] = &account_data.data;

        let policy_version = Kind::try_from_slice(&[account_data[0]])?;

        let current = match policy_version {
            Kind::Policy => Policy::try_deserialize_identities(&account_data[Policy::LEN..]),
            Kind::PolicyV2 => PolicyV2::try_deserialize_identities(&account_data[PolicyV2::LEN..]),
        }?;

        let current_set: HashSet<_> = current.iter().collect();

        let mut iden_to_replace_or_add = VecDeque::new();
        let identities_set: HashSet<_> = identities.iter().collect();

        for i in &identities {
            if !current_set.contains(&i) {
                iden_to_replace_or_add.push_back(*i);
            }
        }

        let mut iden_to_be_replaced_or_deleted_indices = current
            .iter()
            .enumerate()
            .filter_map(|(idx, p)| {
                if p == &Pubkey::default() || !identities_set.contains(p) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<VecDeque<usize>>();

        let len_current = current.len();
        let len_identities = identities.len();

        // REMOVE if current > identities
        let len_diff = len_current.saturating_sub(len_identities);

        let mut remove = Vec::new();
        for _ in 0..len_diff {
            if let Some(idx) = iden_to_be_replaced_or_deleted_indices.pop_back() {
                remove.push(idx);
            }
        }

        let mut replace = Vec::new();
        while let (Some(idx), Some(identity)) = (
            iden_to_be_replaced_or_deleted_indices.pop_front(),
            iden_to_replace_or_add.pop_front(),
        ) {
            replace.push((idx, identity));
        }

        let add: Vec<_> = iden_to_replace_or_add.into_iter().collect();

        // REMOVE
        send_batched_tx(&client, &keypair, &remove, CHUNK_SIZE, |idx| {
            RemoveIdentityBuilder::new()
                .policy(address)
                .mint(*mint)
                .token_account(token_account)
                .owner(keypair.pubkey())
                .index(*idx as u64)
                .instruction()
        })
        .await?;

        // REPLACE
        send_batched_tx(
            &client,
            &keypair,
            &replace,
            CHUNK_SIZE,
            |(idx, identity)| {
                ReplaceIdentityBuilder::new()
                    .policy(address)
                    .mint(*mint)
                    .token_account(token_account)
                    .owner(keypair.pubkey())
                    .identity(*identity)
                    .index(*idx as u64)
                    .instruction()
            },
        )
        .await?;

        // ADD
        send_batched_tx(&client, &keypair, &add, CHUNK_SIZE, |identity| {
            AddIdentityBuilder::new()
                .policy(address)
                .mint(*mint)
                .token_account(token_account)
                .payer(keypair.pubkey())
                .owner(keypair.pubkey())
                .identity(*identity)
                .instruction()
        })
        .await?;

        let account_data = client.get_account(&address).await?;
        let account_data: &[u8] = &account_data.data;

        let policy_version = Kind::try_from_slice(&[account_data[0]])?;

        let policy = match policy_version {
            Kind::Policy => PolicyVersion::V1(Policy::from_bytes(&account_data[..Policy::LEN])?),
            Kind::PolicyV2 => {
                PolicyVersion::V2(PolicyV2::from_bytes(&account_data[..PolicyV2::LEN])?)
            }
        };

        let mint_data = client.get_account(mint).await?;
        let account_data: &[u8] = &mint_data.data;

        let mint_pod = PodStateWithExtensions::<PodMint>::unpack(account_data).unwrap();
        let mint_bytes = mint_pod.get_extension_bytes::<TokenMetadata>().unwrap();
        let token_metadata = TokenMetadata::try_from_slice(mint_bytes).unwrap();

        Ok(CommandComplete(
            SolanaAccount(*mint, Some(token_metadata)),
            SolanaAccount(address, Some(policy)),
        ))
    }
}

/// Builder for removing identities from a policy
pub struct RemoveBatchCommandBuilder<'a> {
    mint: Option<&'a Pubkey>,
    identities: Option<Vec<Pubkey>>,
}

impl Default for RemoveBatchCommandBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> RemoveBatchCommandBuilder<'a> {
    /// Create a new RemoveCommandBuilder
    pub fn new() -> Self {
        Self {
            mint: None,
            identities: None,
        }
    }

    /// Set the mint address
    pub fn mint(mut self, mint: &'a Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the identities to remove
    pub fn identities(mut self, identities: Vec<Pubkey>) -> Self {
        self.identities = Some(identities);
        self
    }
}

#[async_trait::async_trait]
impl RunCommand for RemoveBatchCommandBuilder<'_> {
    /// Execute the removal of an identity from the policy
    async fn run(&mut self, context: CommandContext) -> RunResult {
        let CommandContext { keypair, client } = context;

        let mint = self.mint.expect("mint must be set");
        // PDA seeds are same for both Policy and PolicyV2
        let (address, _) = Policy::find_pda(mint);
        let identities = self.identities.take().expect("identity must be set");

        let token_account = get_associated_token_address_with_program_id(
            &keypair.pubkey(),
            mint,
            &spl_token_2022::ID,
        );

        let account_data = client.get_account(&address).await?;
        let account_data: &[u8] = &account_data.data;

        let policy_version = Kind::try_from_slice(&[account_data[0]])?;

        let current = match policy_version {
            Kind::Policy => Policy::try_deserialize_identities(account_data),
            Kind::PolicyV2 => PolicyV2::try_deserialize_identities(account_data),
        }?;

        let remove: Vec<usize> = identities
            .into_iter()
            .filter_map(|identity| {
                current
                    .iter()
                    .position(|&current_identity| current_identity == identity)
            })
            .collect();

        send_batched_tx(&client, &keypair, &remove, CHUNK_SIZE, |idx| {
            RemoveIdentityBuilder::new()
                .policy(address)
                .mint(*mint)
                .token_account(token_account)
                .owner(keypair.pubkey())
                .index(*idx as u64)
                .instruction()
        })
        .await?;

        let account_data = client.get_account(&address).await?;
        let account_data: &[u8] = &account_data.data;

        let policy_version = Kind::try_from_slice(&[account_data[0]])?;

        let policy = match policy_version {
            Kind::Policy => PolicyVersion::V1(Policy::from_bytes(&account_data[..Policy::LEN])?),
            Kind::PolicyV2 => {
                PolicyVersion::V2(PolicyV2::from_bytes(&account_data[..PolicyV2::LEN])?)
            }
        };

        let mint_data = client.get_account(mint).await?;
        let account_data: &[u8] = &mint_data.data;

        let mint_pod = PodStateWithExtensions::<PodMint>::unpack(account_data).unwrap();
        let mint_bytes = mint_pod.get_extension_bytes::<TokenMetadata>().unwrap();
        let token_metadata = TokenMetadata::try_from_slice(mint_bytes).unwrap();

        Ok(CommandComplete(
            SolanaAccount(*mint, Some(token_metadata)),
            SolanaAccount(address, Some(policy)),
        ))
    }
}
