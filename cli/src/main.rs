use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use log::info;
use solana_cli_config::{Config, CONFIG_FILE};
use yellowstone_shield_cli::{run, Args, CliError, CommandComplete, SolanaAccount};
use yellowstone_shield_client::types::PermissionStrategy;

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let args = Args::parse();

    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let config_file = CONFIG_FILE.as_ref().ok_or(CliError::ConfigFilePathError)?;

    let mut config = Config::load(config_file).map_err::<CliError, _>(Into::into)?;

    if let Some(custom_json_rpc_url) = args.rpc {
        config.json_rpc_url = custom_json_rpc_url;
    }

    if let Some(custom_keypair_path) = args.keypair {
        config.keypair_path = custom_keypair_path;
    }

    config
        .save(config_file)
        .map_err::<CliError, _>(Into::into)?;

    let config = Arc::new(config);

    let CommandComplete(SolanaAccount(mint, token_metadata), SolanaAccount(address, policy)) =
        run(config, args.command).await?;

    info!("📜 Policy");
    info!("--------------------------------");
    info!("🏠 Addresses");
    info!("  📜 Policy: {}", address);
    info!("  🔑 Mint: {}", mint);
    info!("--------------------------------");
    info!("🔍 Details");
    match policy.strategy {
        PermissionStrategy::Allow => info!("  ✅ Strategy: Allow"),
        PermissionStrategy::Deny => info!("  ❌ Strategy: Deny"),
    }
    info!("  🛡️ Identities: {:?}", policy.identities);
    info!("  🏷️ Name: {}", token_metadata.name);
    info!("  🔖 Symbol: {}", token_metadata.symbol);
    info!("  🌐 URI: {}", token_metadata.uri);
    info!("--------------------------------");

    Ok(())
}
