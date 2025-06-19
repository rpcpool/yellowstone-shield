use std::sync::Arc;

use anyhow::Result;
use arc_swap::ArcSwap;
use hashbrown::{HashMap, HashSet};
use parking_lot::RwLock;
use serde::Deserialize;
use tokio::{sync::mpsc::Sender, task::LocalSet};

use solana_account_decoder_client_types::UiAccountEncoding;
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

use yellowstone_shield_client::{
    accounts,
    types::{Kind, PermissionStrategy},
    PolicyTrait,
};
use yellowstone_shield_parser::accounts_parser::{AccountParser, Policy, ShieldProgramState};
use yellowstone_vixen::{
    config::{BufferConfig, OptConfig, VixenConfig, YellowstoneConfig},
    Pipeline, Runtime,
};

pub struct SlotCacheItem<T> {
    slot: u64,
    item: T,
}

/// A thread-safe cache for storing policies by their associated public keys.
pub struct PolicyCache {
    /// A read-write lock-protected hash map that stores policies keyed by public keys.
    /// Each entry contains a tuple of the slot number and the policy.
    policies: RwLock<HashMap<Pubkey, SlotCacheItem<Policy>>>,
}

impl Default for PolicyCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyCache {
    /// Creates a new, empty PolicyCache.
    ///
    /// # Returns
    ///
    /// A new instance of `PolicyCache` with an empty internal storage.
    pub fn new() -> Self {
        Self {
            policies: RwLock::new(HashMap::new()),
        }
    }

    /// Inserts a policy into the cache, associating it with the given public key.
    /// Only updates if the incoming slot is greater than the current slot.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - The public key to associate with the policy.
    /// * `slot` - The slot number of the policy update.
    /// * `policy` - The policy to be stored in the cache.
    pub fn insert(&self, pubkey: Pubkey, slot: u64, item: Policy) {
        let mut policies = self.policies.write();
        if let Some(current_item) = policies.get(&pubkey) {
            if slot > current_item.slot {
                policies.insert(pubkey, SlotCacheItem { slot, item });
            }
        } else {
            policies.insert(pubkey, SlotCacheItem { slot, item });
        }
    }

    /// Retrieves a policy from the cache associated with the given public key.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - The public key whose associated policy is to be retrieved.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the policy if found, or `None` if no policy is associated
    /// with the given public key.
    pub fn get(&self, pubkey: &Pubkey) -> Option<Policy> {
        self.policies
            .read()
            .get(pubkey)
            .map(|item| item.item.clone())
    }

    /// Removes a policy from the cache associated with the given public key.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - The public key whose associated policy is to be removed.
    ///
    /// # Returns
    ///
    /// `Some(())` if a policy was removed, or `None` if no policy was associated with the given public key.
    pub fn remove(&self, pubkey: &Pubkey) -> Option<()> {
        self.policies.write().remove(pubkey).map(|_| ())
    }

    /// Retrieves all policies currently stored in the cache.
    ///
    /// # Returns
    ///
    /// A vector of tuples where each tuple contains a reference to a public key and its
    /// associated policy.
    pub fn all(&self) -> Vec<(Pubkey, Policy)> {
        self.policies
            .read()
            .iter()
            .map(|(k, item)| (*k, item.item.clone()))
            .collect()
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CheckError {
    #[error("Policy not found")]
    PolicyNotFound,
}

/// permission strategies for specific identities.
///
/// The `Snapshot` struct is designed to facilitate quick lookups of permission strategies
/// associated with a combination of policy and identity public keys. It is particularly
/// useful for determining whether a specific identity is allowed or denied by a set of
/// policies.
#[derive(Default)]
pub struct Snapshot {
    /// A hash set that stores tuples of policy and identity public keys for quick lookup.
    lookup: HashSet<(Pubkey, Pubkey)>,
    strategies: HashMap<Pubkey, PermissionStrategy>,
}

impl Snapshot {
    /// Creates a new `Snapshot` from a `PolicyCache`.
    ///
    /// # Arguments
    ///
    /// * `cache` - A reference to a `PolicyCache` from which to create the snapshot.
    ///
    /// # Returns
    ///
    /// A new instance of `Snapshot` with a populated lookup table for quick access to
    /// permission strategies.
    pub fn new(cache: &PolicyCache) -> Self {
        let mut lookup = HashSet::new();
        let mut strategies = HashMap::new();

        for (address, policy) in cache.all().iter() {
            strategies.insert(*address, policy.strategy);
            for identity in &policy.identities {
                lookup.insert((*address, *identity));
            }
        }

        Self { lookup, strategies }
    }

    /// Determines if a identity is allowed by any of the specified policy pubkey.
    ///
    /// This function iterates over a list of policy public keys and checks if a given validator
    /// is allowed according to the permission strategies associated with those policies.
    ///
    /// The function maintains a boolean flag `not_found` initialized to `true`. This flag is used
    /// to track whether any policy with an `Allow` strategy has been encountered that does not
    /// explicitly deny the identity.
    ///
    /// For each policy public key in the provided slice:
    /// - It retrieves the associated permission strategy from the `strategies` map.
    /// - It checks if the combination of the policy public key and the identity public key exists
    ///   in the `lookup` set.
    ///   - If the combination exists, it evaluates the permission strategy:
    ///     - If the strategy is `Deny`, the function immediately returns `false`, indicating the
    ///       identity is not allowed.
    ///     - If the strategy is `Allow`, the function immediately returns `true`, indicating the
    ///       identity is allowed.
    ///   - If the combination does not exist and the strategy is `Allow`, it sets `not_found` to `false`,
    ///     indicating that there is at least one policy that could potentially allow the validator.
    ///
    /// After iterating through all policies, if no explicit `Deny` or `Allow` decision was made,
    /// the function returns the value of `not_found`. If `not_found` is `true`, it means no
    /// applicable `Allow` strategy was found, and the function returns `true`. Otherwise, it returns `false`.
    ///
    /// # Arguments
    ///
    /// * `policies` - A slice of policy public keys to check against.
    /// * `identity` - The identity public key.
    ///
    /// # Returns
    ///
    /// `true` if the identity is allowed by any of the specified policies, `false` otherwise.
    pub fn is_allowed(&self, policies: &[Pubkey], identity: &Pubkey) -> Result<bool, CheckError> {
        let mut not_found = true;

        for address in policies.iter() {
            if let Some(strategy) = self.strategies.get(address) {
                if self.lookup.contains(&(*address, *identity)) {
                    match strategy {
                        PermissionStrategy::Deny => {
                            return Ok(false);
                        }
                        PermissionStrategy::Allow => {
                            return Ok(true);
                        }
                    }
                } else if let PermissionStrategy::Allow = strategy {
                    not_found = false;
                }
            } else {
                return Err(CheckError::PolicyNotFound);
            }
        }

        Ok(not_found)
    }
}

#[derive(Debug, Default)]
pub struct SlotRpcResponse<T> {
    slot: u64,
    result: T,
}

pub type PoliciesSlotRpcResponse = SlotRpcResponse<Vec<(Pubkey, Policy)>>;
pub struct PolicyRpcClient(RpcClient);

impl PolicyRpcClient {
    pub fn new(client: RpcClient) -> Self {
        Self(client)
    }

    pub async fn list(&self, program_id: &Pubkey) -> Result<PoliciesSlotRpcResponse> {
        let slot = self.0.get_slot().await?;

        let result = self
            .0
            .get_program_accounts_with_config(
                program_id,
                RpcProgramAccountsConfig {
                    account_config: RpcAccountInfoConfig {
                        encoding: Some(UiAccountEncoding::Base64),
                        commitment: Some(CommitmentConfig::confirmed()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await?
            .into_iter()
            .filter_map(|(address, account)| {
                let data: &[u8] = &account.data;

                let (strategy, identities) = match data[0] {
                    0 => {
                        let strategy = accounts::Policy::from_bytes(data)
                            .ok()?
                            .try_strategy()
                            .ok()?;
                        let identities = accounts::Policy::try_deserialize_identities(data).ok()?;
                        Some((strategy, identities))
                    }
                    1 => {
                        let strategy = accounts::PolicyV2::from_bytes(data)
                            .ok()?
                            .try_strategy()
                            .ok()?;
                        let identities =
                            accounts::PolicyV2::try_deserialize_identities(data).ok()?;
                        Some((strategy, identities))
                    }
                    _ => None,
                }?;

                let policy = Policy::new(strategy, identities);

                Some((address, policy))
            })
            .collect::<Vec<_>>();

        Ok(SlotRpcResponse { slot, result })
    }
}

impl From<PoliciesSlotRpcResponse> for PolicyCache {
    fn from(response: PoliciesSlotRpcResponse) -> Self {
        let cache = Self::new();

        for (address, policy) in response.result.into_iter() {
            cache.insert(address, response.slot, policy);
        }

        cache
    }
}

#[derive(Debug)]
pub struct PolicyHandler {
    sender: Sender<yellowstone_shield_parser::accounts_parser::ShieldProgramState>,
}

impl PolicyHandler {
    pub fn new(
        sender: Sender<yellowstone_shield_parser::accounts_parser::ShieldProgramState>,
    ) -> Self {
        Self { sender }
    }
}

impl yellowstone_vixen::Handler<ShieldProgramState> for PolicyHandler {
    async fn handle(&self, value: &ShieldProgramState) -> yellowstone_vixen::HandlerResult<()> {
        if let Err(e) = self.sender.send(value.clone()).await {
            log::error!("Failed to send value: {:?}", e);
        }

        Ok(())
    }
}

pub trait PolicyStoreTrait {
    fn snapshot(&self) -> Arc<Snapshot>;
}

/// A structure that manages the caching and synchronization of identity policies.
pub struct PolicyStore {
    /// An atomic reference-counted snapshot of the current state of policies.
    snapshot: Arc<ArcSwap<Snapshot>>,
}

impl PolicyStore {
    /// Creates a new `PolicyStore` from a given set of policies.
    ///
    /// # Arguments
    ///
    /// * `policies` - A response containing policies and their associated slots.
    ///
    /// # Returns
    ///
    /// A new instance of `PolicyStore`.
    pub fn new(snapshot: Arc<ArcSwap<Snapshot>>) -> Self {
        Self { snapshot }
    }
}

impl PolicyStoreTrait for PolicyStore {
    fn snapshot(&self) -> Arc<Snapshot> {
        self.snapshot.load_full()
    }
}

/// A mock implementation of PolicyStore for testing purposes.
pub struct MockPolicyStore {
    snapshot: Arc<Snapshot>,
}

impl MockPolicyStore {
    /// Creates a new `MockPolicyStore` with a given snapshot.
    ///
    /// # Arguments
    ///
    /// * `snapshot` - An atomic reference-counted snapshot of the current state of policies.
    ///
    /// # Returns
    ///
    /// A new instance of `MockPolicyStore`.
    pub fn new(snapshot: Arc<Snapshot>) -> Self {
        Self { snapshot }
    }
}

impl PolicyStoreTrait for MockPolicyStore {
    fn snapshot(&self) -> Arc<Snapshot> {
        Arc::clone(&self.snapshot)
    }
}

pub type SubscriptionTask = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'static>>;

#[derive(Deserialize)]
pub struct PolicyStoreRpcConfig {
    pub endpoint: String,
}

#[derive(Deserialize)]
pub struct PolicyStoreConfig {
    pub rpc: PolicyStoreRpcConfig,
    pub grpc: YellowstoneConfig,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BuilderError {
    #[error("No config")]
    NoConfig,
    #[error("Unable to deserialize policy")]
    DeserializePolicy,
}

#[derive(Default)]
pub struct PolicyStoreBuilder {
    config: Option<PolicyStoreConfig>,
}

impl PolicyStoreBuilder {
    pub fn config(&mut self, config: PolicyStoreConfig) -> &mut Self {
        self.config = Some(config);

        self
    }

    pub async fn run(&mut self, local: &LocalSet) -> Result<PolicyStore> {
        let config = self.config.take().ok_or(BuilderError::NoConfig)?;
        let rpc = RpcClient::new(config.rpc.endpoint);

        let policies = PolicyRpcClient::new(rpc)
            .list(&yellowstone_shield_client::ID)
            .await?;

        let cache = Arc::new(policies.into());
        let snapshot = Arc::new(ArcSwap::from_pointee(Snapshot::new(&cache)));

        let (sender, mut receiver) = tokio::sync::mpsc::channel::<ShieldProgramState>(10_000);

        let vixen = VixenConfig {
            yellowstone: config.grpc,
            buffer: BufferConfig::default(),
            metrics: OptConfig::default(),
        };

        let pipeline = Pipeline::new(AccountParser, [PolicyHandler::new(sender)]);
        let runtime = Runtime::builder()
            .account(pipeline)
            .commitment_level(yellowstone_vixen::CommitmentLevel::Confirmed)
            .build(vixen);

        let cache = Arc::clone(&cache);

        let subscription_snapshot = Arc::clone(&snapshot);
        local.spawn_local(Box::pin(async move {
            tokio::task::spawn_local(async move {
                if let Err(e) = runtime.try_run_async().await {
                    log::error!("Vixen runtime error: {:?}", e);
                }
            });

            while let Some(value) = receiver.recv().await {
                let ShieldProgramState::Policy(slot, pubkey, policy) = value;
                cache.insert(pubkey, slot, policy);
                subscription_snapshot.store(Arc::new(Snapshot::new(&cache)));
            }
        }) as SubscriptionTask);

        Ok(PolicyStore::new(snapshot))
    }
}

impl PolicyStore {
    pub fn build() -> PolicyStoreBuilder {
        PolicyStoreBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;
    use yellowstone_shield_parser::accounts_parser::Policy;

    #[test]
    fn test_policy_cache_insert_and_get() {
        let cache = PolicyCache::new();
        let address = Pubkey::new_unique();
        let validator = Pubkey::new_unique();
        let policy = Policy::new(
            yellowstone_shield_client::types::PermissionStrategy::Deny,
            vec![validator],
        );

        cache.insert(address, 1, policy.clone());
        let retrieved_policy = cache.get(&address).unwrap();

        assert_eq!(retrieved_policy.strategy, policy.strategy);
        assert_eq!(retrieved_policy.identities, policy.identities);
    }

    #[test]
    fn test_policy_cache_all() {
        let cache = PolicyCache::new();
        let validator = Pubkey::new_unique();

        let policies = [
            (
                Pubkey::new_unique(),
                Policy::new(
                    yellowstone_shield_client::types::PermissionStrategy::Deny,
                    vec![validator],
                ),
            ),
            (
                Pubkey::new_unique(),
                Policy::new(
                    yellowstone_shield_client::types::PermissionStrategy::Allow,
                    vec![validator],
                ),
            ),
        ];

        for (pubkey, policy) in policies.iter() {
            cache.insert(*pubkey, 1, policy.clone());
        }

        let policies = cache.all();
        assert_eq!(policies.len(), 2);
    }

    #[test]
    fn test_policy_cache_remove() {
        let cache = PolicyCache::new();
        let address = Pubkey::new_unique();
        let validator = Pubkey::new_unique();
        let policy = Policy::new(
            yellowstone_shield_client::types::PermissionStrategy::Deny,
            vec![validator],
        );

        cache.insert(address, 1, policy.clone());
        cache.remove(&address).unwrap();

        assert!(cache.get(&address).is_none());
    }

    #[test]
    fn test_snapshot_is_allowed() {
        let cache = PolicyCache::new();

        let deny = Pubkey::new_unique();
        let allow = Pubkey::new_unique();
        let missing = Pubkey::new_unique();

        let good = Pubkey::new_unique();
        let other = Pubkey::new_unique();
        let sanctioned = Pubkey::new_unique();
        let sandwich = Pubkey::new_unique();

        let policies = [
            (
                allow,
                Policy::new(
                    yellowstone_shield_client::types::PermissionStrategy::Allow,
                    vec![good],
                ),
            ),
            (
                deny,
                Policy::new(
                    yellowstone_shield_client::types::PermissionStrategy::Deny,
                    vec![sanctioned, sandwich],
                ),
            ),
        ];

        for (address, policy) in policies.into_iter() {
            cache.insert(address, 1, policy.clone());
        }
        let snapshot = Snapshot::new(&cache);

        assert_eq!(
            snapshot.is_allowed(&[missing], &good),
            Err(CheckError::PolicyNotFound)
        );
        assert_eq!(
            snapshot.is_allowed(&[missing, allow], &good),
            Err(CheckError::PolicyNotFound)
        );
        assert_eq!(snapshot.is_allowed(&[allow, missing], &good), Ok(true));
        assert_eq!(
            snapshot.is_allowed(&[deny, missing], &good),
            Err(CheckError::PolicyNotFound)
        );

        assert_eq!(snapshot.is_allowed(&[deny], &sanctioned), Ok(false));
        assert_eq!(snapshot.is_allowed(&[deny], &sandwich), Ok(false));
        assert_eq!(snapshot.is_allowed(&[deny], &good), Ok(true));
        assert_eq!(snapshot.is_allowed(&[deny], &other), Ok(true));

        assert_eq!(snapshot.is_allowed(&[allow], &good), Ok(true));
        assert_eq!(snapshot.is_allowed(&[allow], &sanctioned), Ok(false));
        assert_eq!(snapshot.is_allowed(&[allow], &sandwich), Ok(false));
        assert_eq!(snapshot.is_allowed(&[allow], &other), Ok(false));

        assert_eq!(snapshot.is_allowed(&[allow, deny], &other), Ok(false));
        assert_eq!(snapshot.is_allowed(&[allow, deny], &good), Ok(true));
        assert_eq!(snapshot.is_allowed(&[allow, deny], &sandwich), Ok(false));

        assert_eq!(snapshot.is_allowed(&[deny, allow], &other), Ok(false));
        assert_eq!(snapshot.is_allowed(&[deny, allow], &good), Ok(true));
        assert_eq!(snapshot.is_allowed(&[deny, allow], &sandwich), Ok(false));
    }

    #[test]
    fn test_mock_policy_store() {
        let snapshot = Arc::new(Snapshot::default());

        let store = MockPolicyStore::new(Arc::clone(&snapshot));

        let fetched = store.snapshot();

        assert!(std::sync::Arc::ptr_eq(&fetched, &snapshot));
    }
}
