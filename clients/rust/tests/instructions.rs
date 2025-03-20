#![cfg(feature = "test-sbf")]
use borsh::BorshDeserialize;
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    rent::Rent,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_2022::{
    extension::{metadata_pointer, Extension, ExtensionType},
    instruction::{initialize_mint, initialize_mint2, mint_to},
    state::{Account, Mint, PackedSizeOf},
};
use spl_token_metadata_interface::{
    instruction::initialize as initialize_metadata, state::TokenMetadata,
};
use yellowstone_blocklist_client::{
    accounts::Policy, instructions::CreatePolicyBuilder, CreateAccountBuilder,
    InitializeMint2Builder, InitializeTokenExtensionsAccountBuilder,
    MetadataPointerInitializeBuilder, Size, TokenExtensionsMintToBuilder, TransactionBuilder,
};

#[tokio::test]
async fn test_create() {
    let context = ProgramTest::new(
        "yellowstone_blocklist",
        yellowstone_blocklist_client::ID,
        None,
    )
    .start_with_context()
    .await;

    // Given a PDA derived from the payer's public key.
    let mint = Keypair::new();
    // Create a token account for the payer.
    let payer_token_account = Keypair::new();
    // Mock the validator identity.
    let validator_identity = Keypair::new();

    let _token_metadata = TokenMetadata {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        uri: "https://example.com".to_string(),
        mint: mint.pubkey(),
        update_authority: Some(context.payer.pubkey()).try_into().unwrap(),
        additional_metadata: Vec::new(),
    };

    // Calculate the space required for the mint account with extensions.
    let space = ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::MetadataPointer])
        .unwrap();

    let create_mint_ix = CreateAccountBuilder::build()
        .payer(&context.payer.pubkey())
        .account(&mint.pubkey())
        .space(space)
        .owner(&spl_token_2022::id())
        .instruction();

    // Initialize metadata pointer extension.
    let init_metadata_pointer_ix = MetadataPointerInitializeBuilder::build()
        .mint(&mint.pubkey())
        .metadata(mint.pubkey())
        .authority(context.payer.pubkey())
        .instruction();

    let init_mint_ix = InitializeMint2Builder::build()
        .mint(&mint.pubkey())
        .mint_authority(&context.payer.pubkey())
        .instruction();

    // Create the policy account.
    let address = Policy::find_pda(&mint.pubkey()).0;
    let create_policy_ix = CreatePolicyBuilder::new()
        .policy(address)
        .mint(mint.pubkey())
        .payer(context.payer.pubkey())
        .token_account(payer_token_account.pubkey())
        .validator_identities(vec![validator_identity.pubkey()])
        .strategy(yellowstone_blocklist_client::types::PermissionStrategy::Allow)
        .instruction();

    // TODO: Figure out spacing to be able to create the metadata in same transaction.
    // Create metadata.
    let _create_metadata_ix = initialize_metadata(
        &spl_token_2022::id(),
        &mint.pubkey(),
        &context.payer.pubkey(),
        &mint.pubkey(),
        &context.payer.pubkey(),
        "Test".to_string(),
        "TEST".to_string(),
        "https://example.com".to_string(),
    );

    // Create a token account for the payer.
    let create_payer_token_account_ix = CreateAccountBuilder::build()
        .payer(&context.payer.pubkey())
        .account(&payer_token_account.pubkey())
        .space(spl_token_2022::state::Account::SIZE_OF)
        .owner(&spl_token_2022::id())
        .instruction();

    // Initialize the payer's token account.
    let init_payer_token_account_ix = InitializeTokenExtensionsAccountBuilder::build()
        .account(&payer_token_account.pubkey())
        .mint(&mint.pubkey())
        .owner(&context.payer.pubkey())
        .instruction();

    // Mint 1 token to the payer's token account.
    let mint_to_payer_ix = TokenExtensionsMintToBuilder::build()
        .mint(&mint.pubkey())
        .account(&payer_token_account.pubkey())
        .owner(&context.payer.pubkey())
        .amount(1)
        .instruction();

    let tx = TransactionBuilder::build()
        .instruction(create_mint_ix)
        .instruction(init_metadata_pointer_ix)
        .instruction(init_mint_ix)
        .instruction(create_payer_token_account_ix)
        .instruction(init_payer_token_account_ix)
        .instruction(mint_to_payer_ix)
        .instruction(create_policy_ix)
        .signer(&context.payer)
        .signer(&mint)
        .signer(&payer_token_account)
        .payer(&context.payer.pubkey())
        .recent_blockhash(context.last_blockhash)
        .transaction();

    context.banks_client.process_transaction(tx).await.unwrap();

    let policy_account = context.banks_client.get_account(address).await.unwrap();
    assert!(policy_account.is_some());

    let policy_account = policy_account.unwrap();
    let mut policy_account_data = policy_account.data.as_ref();

    let policy = Policy::deserialize(&mut policy_account_data).unwrap();

    assert_eq!(policy_account.data.len(), policy.size());
    assert_eq!(
        policy.validator_identities,
        vec![validator_identity.pubkey()]
    );
    assert_eq!(
        policy.strategy,
        yellowstone_blocklist_client::types::PermissionStrategy::Allow
    );

    let mint_account = context
        .banks_client
        .get_account(mint.pubkey())
        .await
        .unwrap();
    assert!(mint_account.is_some());

    let payer_token_account_data = context
        .banks_client
        .get_account(payer_token_account.pubkey())
        .await
        .unwrap();
    assert!(payer_token_account_data.is_some());
}
