use std::{fs, path::Path};

use anyhow::Context;
use serde::{de, Deserialize, Deserializer};
use solana_sdk::{pubkey::Pubkey, signature::Keypair};

#[derive(Deserialize, Debug)]
pub struct ConfigCli {
    // Keypair to pay transaction and PDA initializer
    #[serde(deserialize_with = "ConfigCli::deserialize_keypair")]
    pub payer: Keypair,
    // RPC used to make transaction
    pub rpc_url: String,
    #[serde(deserialize_with = "ConfigCli::deserialize_keypair")]
    // PDA authority to sign transaction
    pub authority: Keypair,
    // Smart contract pubkey
    #[serde(deserialize_with = "deserialize_pubkey")]
    pub program_id: Pubkey,
}

pub fn load_config<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let contents = fs::read(path).with_context(|| "failed to read config")?;
    serde_yml::from_slice(&contents).map_err(Into::into)
}

impl ConfigCli {
    pub fn deserialize_keypair<'de, D>(deserializer: D) -> Result<Keypair, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Values {
            Bytes(Vec<u8>),
            KeypairStr(String),
        }
        let key = Values::deserialize(deserializer)?;

        match key {
            Values::Bytes(keypair) => Keypair::from_bytes(&keypair).map_err(de::Error::custom),
            Values::KeypairStr(keypair) => Ok(Keypair::from_base58_string(&keypair)),
        }
    }
}

pub fn deserialize_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)?
        .parse()
        .map_err(de::Error::custom)
}

pub fn parse_keypair_file(path: impl AsRef<Path>) -> anyhow::Result<Keypair> {
    let contents = fs::read(path).with_context(|| "failed to read config")?;
    let bytes = serde_json::from_slice::<Vec<u8>>(&contents)?;
    Keypair::from_bytes(&bytes).with_context(|| "failed to parse keypair file")
}

pub fn parse_pubkey_file(path: impl AsRef<Path>) -> anyhow::Result<Vec<Pubkey>> {
    let content = std::fs::read_to_string(path)?;
    let mut pubkeys: Vec<Pubkey> = vec![];
    for pubkey in content.lines() {
        pubkeys.push(pubkey.trim().parse()?);
    }
    Ok(pubkeys)
}
