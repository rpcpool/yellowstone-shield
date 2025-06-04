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
use solana_sdk::pubkey::Pubkey;
use yellowstone_shield_store::{PolicyStore, PolicyStoreConfig, PolicyStoreTrait};

#[derive(Parser)]
struct Opts {
    #[clap(short, long)]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Opts { config } = Opts::parse();
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let config: PolicyStoreConfig = toml::from_str(&config).expect("Error parsing config");

    let local = tokio::task::LocalSet::new();

    let policy_store = PolicyStore::build()
        .config(config)
        .run(&local)
        .await?;

    local
        .run_until(async {
            // Retrieve the latest snapshot of validator policies
            let snapshot = policy_store.snapshot();

            // Define validator and policy pubkeys to check permission
            // Note: These are dummy pubkeys for demonstration purposes
            let validator = Pubkey::new_unique();
            let policy = Pubkey::new_unique();

            // Check if the validator is allowed by the policy
            match snapshot.is_allowed(&[policy], &validator) {
                Ok(true) => println!("Validator is allowed."),
                Ok(false) => println!("Validator is denied."),
                Err(e) => println!("Error checking policy: {:?}", e),
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
