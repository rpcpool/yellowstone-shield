//! Simple blocklist creation example
//!
//! This example demonstrates how to create a basic blocklist and add addresses to it.

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    signer::EncodableKey,
    transaction::Transaction,
};
use yellowstone_blocklist_sdk::{AclType, BlocklistClient, IndexPubkey};

fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Read environment variables
    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| {
        println!("Using default RPC URL");
        "http://localhost:8899".to_string()
    });

    let authority_keypair_path = std::env::var("AUTHORITY_KEYPAIR").unwrap_or_else(|_| {
        panic!("AUTHORITY_KEYPAIR environment variable not set. Please set it to the path of your keypair file.")
    });

    // Display configuration
    println!("Using RPC URL: {}", rpc_url);
    println!("Using keypair from: {}", authority_keypair_path);

    // Setup client
    let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    let blocklist_client = BlocklistClient::default();

    println!(
        "Using Yellowstone Blocklist Program ID: {}",
        blocklist_client.program_id()
    );

    // Load keypairs
    let authority =
        Keypair::read_from_file(&authority_keypair_path).expect("Failed to read authority keypair");
    let payer =
        Keypair::read_from_file(&authority_keypair_path).expect("Failed to read payer keypair");

    // Initialize a new blocklist with Deny type
    println!("\n=== Creating new deny list... ===");
    let (init_ix, pda) = blocklist_client
        .create_initialize_instruction(&payer.pubkey(), Some(authority.pubkey()), AclType::Deny)
        .expect("Failed to create initialization instruction");

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .expect("Failed to get recent blockhash");

    let init_tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&payer.pubkey()),
        &[&payer, &authority],
        recent_blockhash,
    );

    let signature = rpc_client
        .send_and_confirm_transaction(&init_tx)
        .expect("Failed to initialize blocklist");
    println!("Blocklist created successfully!");
    println!("Transaction signature: {}", signature);
    println!("Blocklist PDA: {}", pda);

    // Add some addresses to the blocklist
    println!("\n=== Adding addresses to blocklist... ===");
    let addresses = [
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
        Keypair::new().pubkey(),
    ];

    for (i, addr) in addresses.iter().enumerate() {
        println!("Address {}: {}", i, addr);
    }

    let add_ix = blocklist_client
        .create_add_instruction(
            &payer.pubkey(),
            &authority.pubkey(),
            &pda,
            addresses
                .iter()
                .enumerate()
                .map(|(i, key)| IndexPubkey {
                    index: i as u64,
                    key: *key,
                })
                .collect(),
        )
        .expect("Failed to create add instruction");

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .expect("Failed to get recent blockhash");

    let add_tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&payer.pubkey()),
        &[&payer, &authority],
        recent_blockhash,
    );

    let signature = rpc_client
        .send_and_confirm_transaction(&add_tx)
        .expect("Failed to add addresses");
    println!("Addresses added to blocklist successfully!");
    println!("Transaction signature: {}", signature);

    println!("\nDeny list is now active and can be used by other programs.");
    println!("Blocklist PDA address (save this for later): {}", pda);
    println!("\nRun the query_blocklist example to check addresses");
}
