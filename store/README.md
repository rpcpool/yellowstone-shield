# Yellowstone Shield Policy Store

The Policy Store is a library for managing and caching validator policies for Yellowstone Shield. It provides caching, snapshot updates, and validator permission lookups.

## Features

- **Thread-safe Cache:** Uses internal locking to manage validator policies.
- **Atomic Snapshots:** Utilizes `ArcSwap` for updating policy snapshots without locking reads.
- **Real-time Updates:** Synchronizes the cache and snapshot with policy updates from Solana RPC or gRPC.

## Usage

To integrate and use the Policy Store in Rust applications:

```bash
cargo add yellowstone-shield-store
```

```rust
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use yellowstone_shield_store::{BuiltPolicyStore, NullConfig, PolicyStoreBuilder, VixenConfig, PolicyStoreTrait};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
// Initialize the RPC client to communicate with Solana
let rpc = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());

    // Set up the default configuration for Vixen
    let vixen = VixenConfig::<NullConfig>::default();

    // Optionally seed and sync policy from on-chain
    let BuiltPolicyStore { subscription, policies } = PolicyStoreBuilder::new()
        .rpc(rpc)
        .vixen(vixen)
        .build()
        .await?;

    let local = tokio::task::LocalSet::new();

    if let Some(subscription) = subscription {
        local.spawn_local(subscription);
    }

    local
        .run_until(async {
            // Retrieve the latest snapshot of validator policies
            let snapshot = policies.snapshot();

            // Define validator and policy pubkeys to check permission
            // Note: These are dummy pubkeys for demonstration purposes
            let validator = Pubkey::new_unique();
            let policy = Pubkey::new_unique();

            // Check if the validator is allowed by the policy
            if snapshot.is_allowed(&[policy], &validator) {
                println!("Validator is allowed.");
            } else {
                println!("Validator is denied.");
            }

            Ok(())
        })
        .await
}

```

## Development

Ensure you have Rust installed, then use:

```bash
cargo build
cargo test
```

## License

Licensed under AGPL-3.0.
