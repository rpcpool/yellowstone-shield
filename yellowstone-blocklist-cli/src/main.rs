use std::{fs, str::FromStr};

use {
    borsh::BorshSerialize,
    clap::{Parser, Subcommand},
    solana_client::{
        client_error::ClientErrorKind,
        rpc_client::RpcClient,
        rpc_request::{RpcError, RpcResponseErrorData},
    },
    solana_sdk::{
        hash::Hash,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_program,
        transaction::Transaction,
    },
    solana_yellowstone_blocklist::{
        instruction::{
            AclPayload, ConfigInstructions, DeleteListPayload, IndexPubkey, InitializeListPayload,
        },
        pda::find_pda,
        state::{AclType as AclTypeContract, ListState, ZEROED},
    },
};

mod config;
use anyhow::{anyhow, Context};
use config::{load_config, parse_keypair_file, parse_pubkey_file, ConfigCli};
use solana_client::rpc_client::SerializableTransaction;
use solana_sdk::bs58;
use solana_yellowstone_blocklist::instruction::AddListPayload;

const MAX_PUBKEYS_PER_TRANSACTION: usize = 20;

#[derive(Debug, Parser)]
struct Args {
    /// Pass a base58 keypair or a file with uint8 array
    #[clap(long)]
    authority: Option<String>,

    /// Pass a base58 keypair or a file with uint8 array
    #[clap(long)]
    payer: Option<String>,

    /// Pass a base58 pubkey or a file with uint8 array
    #[clap(long)]
    program_id: Option<String>,

    /// RPC url
    #[clap(long)]
    rpc_url: Option<String>,

    /// Config file used to run the CLI program
    #[clap(long, default_value = "./config_list.yaml")]
    config: String,

    #[command(subcommand)]
    action: ActionType,
}

#[derive(Clone, Debug, Subcommand)]
enum ActionType {
    /// Used to initialize PDA account. It is required to pass deny or allow type list
    Initialize {
        #[command(subcommand)]
        acl_type: AclType,
    },
    /// Pass a file with pubkeys for each line or a base58 pubkey
    Add {
        #[arg(default_value = "pubkeys_file.txt")]
        keys: String,
    },

    /// Pass a file with pubkeys for each line or a base58 pubkey
    Delete {
        #[arg(default_value = "pubkeys_file.txt")]
        keys: String,
    },
    /// Close account and send lamports to destination account. In case destination account is not passed, payer will be used by default.
    Close {
        #[clap(long)]
        dest_key: Option<String>,
    },
    /// Freeze account
    Freeze,
    /// Update list type.  It is required to pass deny or allow type list
    UpdateAcl {
        #[command(subcommand)]
        acl_type: AclType,
    },
    /// Returns PDA account state. The pubkey arg is optional, if is not used, then the PDA calculated by the payer will be used.
    State { pubkey: Option<String> },
    /// Returns PDA key
    PdaKey,
}

impl Default for ActionType {
    fn default() -> Self {
        Self::Initialize {
            acl_type: AclType::Deny,
        }
    }
}

#[derive(Debug, Clone, Default, Subcommand)]
enum AclType {
    #[default]
    Deny,
    Allow,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let res = load_config::<ConfigCli>(args.config);

    let (autho, payer, rpc_url, program_id) = match res {
        Ok(ConfigCli {
            authority,
            payer,
            rpc_url,
            program_id,
        }) => {
            let autho = if let Some(auth) = args.authority {
                identify_string_keypair(&auth)?
            } else {
                authority
            };

            let payer = if let Some(pay_keypair) = args.payer {
                identify_string_keypair(&pay_keypair)?
            } else {
                payer
            };

            let rpc_url = if let Some(url) = args.rpc_url {
                url
            } else {
                rpc_url
            };

            let program_id = if let Some(pro_id) = args.program_id {
                identify_string_pubkey(&pro_id)?[0]
            } else {
                program_id
            };
            (autho, payer, rpc_url, program_id)
        }
        Err(_) => {
            let autho = if let Some(auth) = args.authority {
                identify_string_keypair(&auth)?
            } else {
                return Err(anyhow!("Authority is required"));
            };

            let payer = if let Some(pay_keypair) = args.payer {
                identify_string_keypair(&pay_keypair)?
            } else {
                return Err(anyhow!("Payer is required"));
            };

            let rpc_url = if let Some(url) = args.rpc_url {
                url
            } else {
                return Err(anyhow!("RPC url is required"));
            };

            let program_id = if let Some(pro_id) = args.program_id {
                identify_string_pubkey(&pro_id)?[0]
            } else {
                return Err(anyhow!("Program id is required"));
            };

            (autho, payer, rpc_url, program_id)
        }
    };

    let rpc = RpcClient::new(rpc_url);
    let hash = rpc.get_latest_blockhash()?;

    let (pda, _) = find_pda(&program_id.to_bytes(), &payer.pubkey().to_bytes());

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(Pubkey::from(pda), false),
        AccountMeta::new_readonly(system_program::ID, false),
        AccountMeta::new(autho.pubkey(), true),
    ];

    let signing_keypairs = &[&payer, &autho];

    let transaction = match args.action {
        ActionType::Initialize { acl_type } => {
            let acl_type = match acl_type {
                AclType::Deny => AclTypeContract::Deny,
                AclType::Allow => AclTypeContract::Allow,
            };
            let payload = ConfigInstructions::InitializeList(InitializeListPayload { acl_type });
            let mut instruction = vec![];
            payload.serialize(&mut instruction)?;
            make_transaction(
                program_id,
                payer.insecure_clone(),
                hash,
                accounts,
                signing_keypairs,
                instruction,
            )
        }

        ActionType::Delete { keys } => {
            let list = identify_string_pubkey(&keys)?;
            let data = rpc.get_account_data(&Pubkey::from(pda))?;
            let state = ListState::deserialize(&data)
                .map_err(|err| anyhow!("Error deserializing account data: {:?}", err))?;
            let mut vec_index: Vec<usize> = vec![];

            for (index, state_key) in state.list.iter().enumerate() {
                for key in list.clone() {
                    if state_key.eq(&key.to_bytes()) {
                        vec_index.push(index);
                        break;
                    }
                }
            }

            let payload = ConfigInstructions::RemoveItemList(DeleteListPayload { vec_index });
            let mut instruction = vec![];
            payload.serialize(&mut instruction)?;
            make_transaction(
                program_id,
                payer.insecure_clone(),
                hash,
                accounts,
                signing_keypairs,
                instruction,
            )
        }
        ActionType::Close { dest_key } => {
            let dest_key = if let Some(acc) = dest_key {
                acc.parse()?
            } else {
                payer.pubkey()
            };

            let accounts = vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(Pubkey::from(pda), false),
                AccountMeta::new(dest_key, false),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new(autho.pubkey(), true),
            ];

            let payload = ConfigInstructions::CloseAccount;
            let mut instruction = vec![];
            payload.serialize(&mut instruction)?;
            make_transaction(
                program_id,
                payer.insecure_clone(),
                hash,
                accounts,
                signing_keypairs,
                instruction,
            )
        }
        ActionType::Freeze => {
            let payload = ConfigInstructions::FreezeAccount;
            let mut instruction = vec![];
            payload.serialize(&mut instruction)?;
            make_transaction(
                program_id,
                payer.insecure_clone(),
                hash,
                accounts,
                signing_keypairs,
                instruction,
            )
        }
        ActionType::UpdateAcl { acl_type } => {
            let acl_type = match acl_type {
                AclType::Deny => AclTypeContract::Deny,
                AclType::Allow => AclTypeContract::Allow,
            };
            let payload = ConfigInstructions::UpdateAclType(AclPayload { acl_type });
            let mut instruction = vec![];
            payload.serialize(&mut instruction)?;

            make_transaction(
                program_id,
                payer.insecure_clone(),
                hash,
                accounts,
                signing_keypairs,
                instruction,
            )
        }
        ActionType::State { pubkey } => {
            let key = if let Some(key_str) = pubkey {
                identify_string_pubkey(&key_str)?[0]
            } else {
                Pubkey::from(pda)
            };
            let data = rpc.get_account_data(&Pubkey::from(key))?;
            let state = ListState::deserialize(&data)
                .map_err(|err| anyhow!("Error deserializing account data: {:?}", err))?;
            let list: Vec<(usize, &[u8; 32])> = state
                .list
                .iter()
                .enumerate()
                .filter_map(|(index, item)| (item.ne(&ZEROED)).then(|| (index, item)))
                .collect();
            println!(
                "
Authority: {:?}
Account List Type: {:?}
Account List Size: {:?}
Account List where format is (index, pubkey)",
                state.meta.authority, state.meta.acl_type, state.meta.list_items
            );

            for (index, pubkey) in list {
                println!("- Index: {:?}, Pubkey: {:?}", index, Pubkey::from(*pubkey));
            }

            return Ok(());
        }
        ActionType::Add { keys } => {
            let list_parsed = identify_string_pubkey(&keys)?;

            let list = get_list_add(&rpc, list_parsed, &Pubkey::from(pda))?;
            let list_len = list.len();
            if list_len > MAX_PUBKEYS_PER_TRANSACTION {
                for i in 0..(list_len / MAX_PUBKEYS_PER_TRANSACTION) + 1 {
                    let start = i * MAX_PUBKEYS_PER_TRANSACTION;
                    let end = if start + MAX_PUBKEYS_PER_TRANSACTION < list_len {
                        start + MAX_PUBKEYS_PER_TRANSACTION
                    } else {
                        list_len
                    };
                    let payload = ConfigInstructions::Add(AddListPayload {
                        list: list[start..end].to_vec(),
                    });
                    let mut instruction = vec![];
                    payload.serialize(&mut instruction)?;
                    let transaction = make_transaction(
                        program_id,
                        payer.insecure_clone(),
                        hash,
                        accounts.clone(),
                        signing_keypairs,
                        instruction,
                    );
                    send_transaction(&rpc, &transaction)?;
                }
                return Ok(());
            }
            let payload = ConfigInstructions::Add(AddListPayload { list });
            let mut instruction = vec![];
            payload.serialize(&mut instruction)?;
            make_transaction(
                program_id,
                payer.insecure_clone(),
                hash,
                accounts,
                signing_keypairs,
                instruction,
            )
        }
        ActionType::PdaKey => {
            println!("PDA key: {:?}", pda);
            return Ok(());
        }
    };
    send_transaction(&rpc, &transaction)
}

fn make_transaction(
    program_id: Pubkey,
    payer: Keypair,
    recent_blockhash: Hash,
    accounts: Vec<AccountMeta>,
    signing_keypairs: &[&Keypair],
    instruction: Vec<u8>,
) -> Transaction {
    Transaction::new_signed_with_payer(
        &[Instruction {
            accounts,
            program_id,
            data: instruction,
        }],
        Some(&payer.pubkey()),
        signing_keypairs,
        recent_blockhash,
    )
}

fn send_transaction(
    rpc: &RpcClient,
    transaction: &impl SerializableTransaction,
) -> anyhow::Result<()> {
    match rpc.send_and_confirm_transaction(transaction) {
        Ok(signature) => {
            println!("https://explorer.solana.com/tx/{:?}", signature);
            Ok(())
        }
        Err(err) => {
            // println!("Raw error: {:?}", err);

            match err.kind {
                ClientErrorKind::RpcError(rpc_error) => match rpc_error {
                    RpcError::RpcResponseError {
                        code,
                        message,
                        data,
                    } => {
                        println!(
                            "
Code: {}
Message: {}
",
                            code, message
                        );
                        match data {
                            RpcResponseErrorData::SendTransactionPreflightFailure(values) => {
                                println!("LOGS:");
                                if let Some(logs) = values.logs {
                                    logs.iter().for_each(|log| println!("{:?}", log));
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                },
                _ => println!("Other error: {:?}", err),
            }

            Ok(())
        }
    }
}

fn get_list_add(
    rpc: &RpcClient,
    list_parsed: Vec<Pubkey>,
    pda: &Pubkey,
) -> anyhow::Result<Vec<IndexPubkey>> {
    let data = rpc.get_account_data(pda)?;
    let state: ListState<'_> = ListState::deserialize(&data)
        .map_err(|err| anyhow!("Error deserializing account data: {:?}", err))?;
    let indexes: Vec<usize> = state
        .list
        .iter()
        .enumerate()
        .filter_map(|(index, key)| (key.eq(&ZEROED)).then(|| index))
        .collect();

    let list_len = list_parsed.len();

    let mut list = vec![];
    let mut it_list = list_parsed.iter();
    let mut iter_i = indexes.iter();
    loop {
        match (it_list.next(), iter_i.next()) {
            (Some(key), Some(index)) => list.push(IndexPubkey {
                index: *index as u64,
                key: key.to_bytes(),
            }),
            (Some(key), None) => list.push(IndexPubkey {
                index: (state.meta.list_items + list_len) as _,
                key: key.to_bytes(),
            }),
            (None, Some(_)) => break,
            (None, None) => break,
        }
    }
    Ok(list)
}

fn identify_string_keypair(input: &str) -> anyhow::Result<Keypair> {
    if fs::metadata(input).is_ok() {
        let keypair = parse_keypair_file(input)?;
        return Ok(keypair);
    }

    // Check if it's base58 by attempting to decode it
    if bs58::decode(input).into_vec().is_ok() {
        let keypair = Keypair::from_base58_string(input);
        return Ok(keypair);
    }

    Err(anyhow!("Unknown keypair"))
}

// Function to check if a string is a valid file path or a base58-encoded string
fn identify_string_pubkey(input: &str) -> anyhow::Result<Vec<Pubkey>> {
    // Check if it's a file path by trying to access it
    if fs::metadata(input).is_ok() {
        let contents = fs::read(input).with_context(|| "failed to read file")?;
        let bytes = serde_json::from_slice::<Vec<u8>>(&contents);
        match bytes {
            Ok(bytes) => {
                let key = Pubkey::try_from(bytes).map_err(|_| anyhow!("Invalid Pubkey"))?;
                return Ok(vec![key]);
            }
            Err(_) => {
                let pubkeys = parse_pubkey_file(input)?;
                return Ok(pubkeys);
            }
        }
    }

    // Check if it's base58 by attempting to decode it
    if bs58::decode(input).into_vec().is_ok() {
        let pubkey = Pubkey::from_str(input)?;
        return Ok(vec![pubkey]);
    }

    Err(anyhow!("Unknown pubkey format"))
}
