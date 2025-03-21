mod command;

use crate::command::RunCommand;
use anyhow::{anyhow, Context, Result};
use bs58::decode;
use clap::Parser;
use clap_derive::{Parser, Subcommand};
use command::{policy, validator};
use ed25519_dalek::ed25519::signature;
use serde_json::from_str as parse_json_str;
use solana_cli_config::{Config, CONFIG_FILE};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::fs::read_to_string as read_path;
use std::{str::FromStr, time::Duration};
use yellowstone_blocklist_client::types::PermissionStrategy;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    name = "Yellowstone blocklist CLI",
    about = "CLI for managing Yellowstone blocklist policies"
)]
pub struct Args {
    /// RPC endpoint url to override using the Solana config
    #[arg(short, long, global = true)]
    pub rpc: Option<String>,

    /// Timeout to override default value of 90 seconds
    #[arg(short = 'T', long, global = true, default_value_t = 90)]
    pub timeout: u64,

    /// Log level
    #[arg(short, long, global = true, default_value = "off")]
    pub log_level: String,

    /// Path to the owner keypair file
    #[arg(short, long, global = true)]
    pub keypair: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

pub struct CommandContext {
    pub client: RpcClient,
    pub keypair: Keypair,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage policies
    Policy {
        #[command(subcommand)]
        action: PolicyAction,
    },
    /// Manage validators
    Validator {
        #[command(subcommand)]
        action: ValidatorAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum PolicyAction {
    /// Create a new policy
    Create {
        /// The strategy to use for the policy
        #[arg(long)]
        strategy: PermissionStrategy,
        /// The validator identities to add to the policy
        #[arg(long, value_delimiter = ',')]
        validator_identities: Vec<Pubkey>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ValidatorAction {
    /// Add a validator to a policy
    Add {
        /// The policy to which the validator will be added
        #[arg(long)]
        policy: Pubkey,
        /// The mint address associated with the policy
        #[arg(long)]
        mint: Pubkey,
        /// The validator to add to the policy
        validator_identity: Pubkey,
    },
    /// Remove a validator from a policy
    Remove {
        /// The policy from which the validator will be removed
        #[arg(long)]
        policy: Pubkey,
        /// The mint address associated with the policy
        #[arg(long)]
        mint: Pubkey,
        /// The validator to remove from the policy
        validator_identity: Pubkey,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error("unable to get config file path")]
    ConfigFilePathError,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    ParseCommitmentLevelError(#[from] solana_sdk::commitment_config::ParseCommitmentLevelError),
    #[error("unable to parse keypair")]
    Keypair,
}

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let args = Args::parse();
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();
    let config_file = CONFIG_FILE.as_ref().ok_or(CliError::ConfigFilePathError)?;

    let mut cli_config = Config::load(config_file).map_err::<CliError, _>(Into::into)?;

    if let Some(custom_json_rpc_url) = args.rpc {
        cli_config.json_rpc_url = custom_json_rpc_url;
    }

    if let Some(custom_keypair_path) = args.keypair {
        cli_config.keypair_path = custom_keypair_path;
    }

    cli_config
        .save(config_file)
        .map_err::<CliError, _>(Into::into)?;

    let client = RpcClient::new_with_timeout_and_commitment(
        cli_config.json_rpc_url,
        Duration::from_secs(args.timeout),
        CommitmentConfig::from_str(&cli_config.commitment).map_err::<CliError, _>(Into::into)?,
    );
    let keypair = parse_keypair(&cli_config.keypair_path)?;
    let context = CommandContext { keypair, client };

    match &args.command {
        Command::Policy { action } => match action {
            PolicyAction::Create {
                strategy,
                validator_identities,
            } => policy::CreateCommandBuilder::new()
                .strategy(strategy)
                .validator_identities(validator_identities)
                .run(context)
                .await
                .map_err(Into::into),
        },
        Command::Validator { action } => match action {
            ValidatorAction::Add {
                policy,
                mint,
                validator_identity,
            } => {
                // Add logic to add a validator
                validator::AddCommandBuilder::new()
                    .policy(policy)
                    .mint(mint)
                    .validator_identity(validator_identity)
                    .run(context)
                    .await
                    .map_err(Into::into)
            }
            ValidatorAction::Remove {
                policy,
                mint,
                validator_identity,
            } => validator::RemoveCommandBuilder::new()
                .policy(policy)
                .mint(mint)
                .validator_identity(validator_identity)
                .run(context)
                .await
                .map_err(Into::into),
        },
    }
}

fn parse_keypair(keypair_path: &str) -> Result<Keypair, CliError> {
    let secret_string = read_path(keypair_path).context("Can't find key file")?;
    let secret_bytes = parse_json_str(&secret_string)
        .or_else(|_| decode(&secret_string.trim()).into_vec())
        .map_err(|_| CliError::ConfigFilePathError)?;

    Keypair::from_bytes(&secret_bytes).map_err(|_| CliError::Keypair)
}
