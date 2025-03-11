use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::str::FromStr;
use yellowstone_blocklist_sdk::{BlocklistClient, ListState};

fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Read environment variables
    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| {
        println!("Using default RPC URL");
        "https://api.mainnet-beta.solana.com".to_string()
    });

    let blocklist_pda = std::env::var("BLOCKLIST_PDA")
        .unwrap_or_else(|_| panic!("BLOCKLIST_PDA environment variable not set"));

    // Setup client
    println!("Connecting to Solana at {}", rpc_url);
    let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    let blocklist_client = BlocklistClient::default();
    println!("Using program ID: {}", blocklist_client.program_id());

    // Parse pubkeys
    let blocklist_pda = Pubkey::from_str(&blocklist_pda).expect("Invalid blocklist PDA address");

    // Get blocklist data
    println!("\n=== Fetching blocklist data ===");
    println!("Blocklist PDA: {}", blocklist_pda);

    let account_data = match rpc_client.get_account_data(&blocklist_pda) {
        Ok(data) => data,
        Err(err) => {
            panic!("Failed to get blocklist account data: {}", err);
        }
    };

    // Use ListState to deserialize both metadata and list
    let list_state =
        ListState::deserialize(&account_data).expect("Failed to deserialize blocklist");

    // Print metadata
    println!("Blocklist type: {:?}", list_state.meta.acl_type);
    println!("Number of items in list: {}", list_state.meta.list_items);

    if let Some(authority) = list_state.meta.authority {
        println!("Blocklist authority: {}", authority);
    } else {
        println!("Blocklist has no authority (likely frozen)");
    }

    // Display all addresses in the list
    println!("\n=== Addresses in the blocklist ===");
    for (i, address) in list_state.list.iter().enumerate() {
        if address.to_bytes() != [0; 32] {
            // Skip zeroed entries
            println!("  {}: {}", i, address);
        }
    }
}
