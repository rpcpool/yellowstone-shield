use std::sync::Arc;

use clap::Parser;
use solana_cli_config::{Config, CONFIG_FILE};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use yellowstone_shield_cli::{run, Command, CommandComplete, PolicyAction, SolanaAccount};
use yellowstone_shield_store::{
    BuiltPolicyStore, NullConfig, PolicyStoreBuilder, PolicyStoreTrait, VixenConfig,
};

#[derive(Parser)]
struct Opts {
    #[clap(short, long)]
    config: String,
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let Opts { config } = Opts::parse();
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let config: VixenConfig<NullConfig> = toml::from_str(&config).expect("Error parsing config");

    let cli = CONFIG_FILE.as_ref().unwrap();

    let mut cli = Config::load(cli).unwrap();
    cli.json_rpc_url = "http://localhost:8899".to_string();

    let cli = Arc::new(cli);

    let rpc = RpcClient::new(cli.json_rpc_url.clone());

    let BuiltPolicyStore {
        policies,
        subscription,
    } = PolicyStoreBuilder::new()
        .rpc(rpc)
        .vixen(config)
        .build()
        .await
        .unwrap();

    let local = tokio::task::LocalSet::new();

    if let Some(sub) = subscription {
        local.spawn_local(sub);
    }

    local
        .run_until(async {
            let good = Pubkey::new_unique();
            let bad = Pubkey::new_unique();
            let other = Pubkey::new_unique();

            let allow = run(
                Arc::clone(&cli),
                Command::Policy {
                    action: PolicyAction::Create {
                        strategy: yellowstone_shield_client::types::PermissionStrategy::Allow,
                        identities: vec![good],
                        name: "Good".to_string(),
                        symbol: "G".to_string(),
                        uri: "https://test.com/good.json".to_string(),
                    },
                },
            )
            .await
            .unwrap();
            let deny = run(
                Arc::clone(&cli),
                Command::Policy {
                    action: PolicyAction::Create {
                        strategy: yellowstone_shield_client::types::PermissionStrategy::Deny,
                        identities: vec![bad],
                        name: "Bad".to_string(),
                        symbol: "B".to_string(),
                        uri: "https://test.com/bad.json".to_string(),
                    },
                },
            )
            .await
            .unwrap();

            let snapshot = policies.snapshot();

            let CommandComplete(_, SolanaAccount(address, _)) = deny;

            assert_eq!(snapshot.is_allowed(&[address], &good), Ok(true));
            assert_eq!(snapshot.is_allowed(&[address], &other), Ok(true));
            assert_eq!(snapshot.is_allowed(&[address], &bad), Ok(false));

            let CommandComplete(_, SolanaAccount(address, _)) = allow;

            assert_eq!(snapshot.is_allowed(&[address], &good), Ok(true));
            assert_eq!(snapshot.is_allowed(&[address], &other), Ok(false));
            assert_eq!(snapshot.is_allowed(&[address], &bad), Ok(false));
        })
        .await;
}
