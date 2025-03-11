//! Full lifecycle example for Yellowstone Blocklist
//!
//! This example demonstrates the full lifecycle of a blocklist:
//! - Initialize blocklist
//! - Add addresses
//! - Update ACL type
//! - Remove addresses
//! - Freeze blocklist
//! - Close blocklist

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

    let authority_keypair_path = std::env::var("AUTHORITY_KEYPAIR")
        .unwrap_or_else(|_| panic!("AUTHORITY_KEYPAIR environment variable not set"));

    // Setup clients
    println!("Connecting to Solana at {}", rpc_url);
    let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    let blocklist_client = BlocklistClient::default();
    println!("Using program ID: {}", blocklist_client.program_id());

    // Load keypairs
    println!("Loading authority keypair from {}", authority_keypair_path);
    let authority =
        Keypair::read_from_file(&authority_keypair_path).expect("Failed to read authority keypair");
    let payer_keypair =
        Keypair::read_from_file(&authority_keypair_path).expect("Failed to read payer keypair");

    // Create some test addresses to block/allow
    let test_address1 = Keypair::new();
    let test_address2 = Keypair::new();

    println!("Generated test addresses:");
    println!("Test address 1: {}", test_address1.pubkey());
    println!("Test address 2: {}", test_address2.pubkey());

    // Initialize blocklist
    let pda = initialize_blocklist(&rpc_client, &blocklist_client, &payer_keypair, &authority);

    // Add addresses to blocklist
    add_to_blocklist(
        &rpc_client,
        &blocklist_client,
        &payer_keypair,
        &authority,
        &pda,
        &test_address1,
        &test_address2,
    );

    // Update ACL type
    update_acl_type(
        &rpc_client,
        &blocklist_client,
        &payer_keypair,
        &authority,
        &pda,
    );

    // Remove an address
    remove_from_blocklist(
        &rpc_client,
        &blocklist_client,
        &payer_keypair,
        &authority,
        &pda,
    );

    // Note: Once frozen, the account becomes permanently immutable
    println!("\nNOTE: Freezing is permanent! The account cannot be closed after freezing.");
    println!("Do you want to freeze the blocklist? This cannot be undone. (y/n)");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    if input.trim().to_lowercase() == "y" {
        // Freeze blocklist
        freeze_blocklist(
            &rpc_client,
            &blocklist_client,
            &payer_keypair,
            &authority,
            &pda,
        );
    } else {
        // Close blocklist if not freezing
        close_blocklist(
            &rpc_client,
            &blocklist_client,
            &payer_keypair,
            &authority,
            &pda,
        );
    }

    println!("Blocklist lifecycle complete!");
}

fn initialize_blocklist(
    rpc_client: &RpcClient,
    blocklist_client: &BlocklistClient,
    payer: &Keypair,
    authority: &Keypair,
) -> solana_sdk::pubkey::Pubkey {
    println!("\n=== Initializing Blocklist ===");

    // Get PDA without creating initialization instruction
    let (pda, _) = blocklist_client.get_blocklist_pda(&payer.pubkey());

    // Check if account already exists
    match rpc_client.get_account(&pda) {
        Ok(_) => {
            println!("ℹ️  Blocklist already exists at PDA: {}", pda);
            return pda;
        }
        Err(_) => {
            println!("Creating a new blocklist with ACL type: Deny");
        }
    }

    // Create and send initialization transaction
    let (init_ix, _) = blocklist_client
        .create_initialize_instruction(&payer.pubkey(), Some(authority.pubkey()), AclType::Deny)
        .expect("Failed to create initialization instruction");

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .expect("Failed to get recent blockhash");

    let init_tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&payer.pubkey()),
        &[payer, authority],
        recent_blockhash,
    );

    match rpc_client.send_and_confirm_transaction(&init_tx) {
        Ok(signature) => {
            println!("✅ Blocklist initialized successfully");
            println!("Transaction signature: {}", signature);
            println!("Blocklist PDA: {}", pda);
            pda
        }
        Err(err) => {
            panic!("❌ Failed to initialize blocklist: {}", err);
        }
    }
}

fn add_to_blocklist(
    rpc_client: &RpcClient,
    blocklist_client: &BlocklistClient,
    payer: &Keypair,
    authority: &Keypair,
    pda: &solana_sdk::pubkey::Pubkey,
    test_address1: &Keypair,
    test_address2: &Keypair,
) -> solana_sdk::signature::Signature {
    println!("\n=== Adding Addresses to Blocklist ===");
    println!("Adding 2 addresses to the blocklist:");
    println!("  - Index 0: {}", test_address1.pubkey());
    println!("  - Index 1: {}", test_address2.pubkey());

    let add_ix = blocklist_client
        .create_add_instruction(
            &payer.pubkey(),
            &authority.pubkey(),
            pda,
            vec![
                IndexPubkey {
                    index: 0,
                    key: test_address1.pubkey(),
                },
                IndexPubkey {
                    index: 1,
                    key: test_address2.pubkey(),
                },
            ],
        )
        .expect("Failed to create add instruction");

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .expect("Failed to get recent blockhash");

    let add_tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&payer.pubkey()),
        &[payer, authority],
        recent_blockhash,
    );

    match rpc_client.send_and_confirm_transaction(&add_tx) {
        Ok(signature) => {
            println!("✅ Addresses added successfully");
            println!("Transaction signature: {}", signature);
            signature
        }
        Err(err) => {
            panic!("❌ Failed to add addresses: {}", err);
        }
    }
}

fn update_acl_type(
    rpc_client: &RpcClient,
    blocklist_client: &BlocklistClient,
    payer: &Keypair,
    authority: &Keypair,
    pda: &solana_sdk::pubkey::Pubkey,
) -> solana_sdk::signature::Signature {
    println!("\n=== Updating ACL Type ===");
    println!("Changing ACL type from Deny to Allow");

    let update_ix = blocklist_client
        .create_update_acl_type_instruction(
            &payer.pubkey(),
            &authority.pubkey(),
            pda,
            AclType::Allow,
        )
        .expect("Failed to create update ACL type instruction");

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .expect("Failed to get recent blockhash");

    let update_tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&payer.pubkey()),
        &[payer, authority],
        recent_blockhash,
    );

    match rpc_client.send_and_confirm_transaction(&update_tx) {
        Ok(signature) => {
            println!("✅ ACL type updated successfully");
            println!("Transaction signature: {}", signature);
            signature
        }
        Err(err) => {
            panic!("❌ Failed to update ACL type: {}", err);
        }
    }
}

fn remove_from_blocklist(
    rpc_client: &RpcClient,
    blocklist_client: &BlocklistClient,
    payer: &Keypair,
    authority: &Keypair,
    pda: &solana_sdk::pubkey::Pubkey,
) -> solana_sdk::signature::Signature {
    println!("\n=== Removing Address from Blocklist ===");
    println!("Removing address at index 0 from blocklist");

    let remove_ix = blocklist_client
        .create_remove_instruction(
            &payer.pubkey(),
            &authority.pubkey(),
            pda,
            vec![0], // Remove first address
        )
        .expect("Failed to create remove instruction");

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .expect("Failed to get recent blockhash");

    let remove_tx = Transaction::new_signed_with_payer(
        &[remove_ix],
        Some(&payer.pubkey()),
        &[payer, authority],
        recent_blockhash,
    );

    match rpc_client.send_and_confirm_transaction(&remove_tx) {
        Ok(signature) => {
            println!("✅ Address removed successfully");
            println!("Transaction signature: {}", signature);
            signature
        }
        Err(err) => {
            panic!("❌ Failed to remove address: {}", err);
        }
    }
}

fn freeze_blocklist(
    rpc_client: &RpcClient,
    blocklist_client: &BlocklistClient,
    payer: &Keypair,
    authority: &Keypair,
    pda: &solana_sdk::pubkey::Pubkey,
) -> solana_sdk::signature::Signature {
    println!("\n=== Freezing Blocklist ===");
    println!("Making the blocklist immutable");

    let freeze_ix = blocklist_client
        .create_freeze_instruction(&payer.pubkey(), &authority.pubkey(), pda)
        .expect("Failed to create freeze instruction");

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .expect("Failed to get recent blockhash");

    let freeze_tx = Transaction::new_signed_with_payer(
        &[freeze_ix],
        Some(&payer.pubkey()),
        &[payer, authority],
        recent_blockhash,
    );

    match rpc_client.send_and_confirm_transaction(&freeze_tx) {
        Ok(signature) => {
            println!("✅ Blocklist frozen successfully");
            println!("Transaction signature: {}", signature);
            signature
        }
        Err(err) => {
            eprintln!("❌ Failed to freeze blocklist: {}", err);
            eprintln!("Note: This may be expected if the account is already frozen");
            panic!("Failed to freeze blocklist");
        }
    }
}

fn close_blocklist(
    rpc_client: &RpcClient,
    blocklist_client: &BlocklistClient,
    payer: &Keypair,
    authority: &Keypair,
    pda: &solana_sdk::pubkey::Pubkey,
) -> solana_sdk::signature::Signature {
    println!("\n=== Closing Blocklist ===");
    println!("Closing the blocklist and recovering rent");

    let close_ix = blocklist_client
        .create_close_instruction(
            &payer.pubkey(),
            &authority.pubkey(),
            pda,
            &authority.pubkey(), // Send rent to authority
        )
        .expect("Failed to create close instruction");

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .expect("Failed to get recent blockhash");

    let close_tx = Transaction::new_signed_with_payer(
        &[close_ix],
        Some(&payer.pubkey()),
        &[payer, authority],
        recent_blockhash,
    );

    match rpc_client.send_and_confirm_transaction(&close_tx) {
        Ok(signature) => {
            println!("✅ Blocklist closed successfully");
            println!("Transaction signature: {}", signature);
            signature
        }
        Err(err) => {
            eprintln!("❌ Failed to close blocklist: {}", err);
            eprintln!("Note: This may be expected if the account is frozen");
            panic!("Failed to close blocklist");
        }
    }
}
