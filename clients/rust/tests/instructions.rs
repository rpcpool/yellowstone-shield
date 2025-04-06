#![cfg(feature = "test-sbf")]
use borsh::BorshDeserialize;
use solana_program_test::{tokio, ProgramTest};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_pod::optional_keys::OptionalNonZeroPubkey;
use spl_token_2022::{
    extension::{BaseStateWithExtensions, ExtensionType, PodStateWithExtensions},
    pod::PodMint,
    state::Mint,
};
use spl_token_metadata_interface::{
    borsh::BorshDeserialize as MetadataInterfaceBorshDeserialize, state::TokenMetadata,
};
use yellowstone_shield_client::instructions::{ClosePolicyBuilder, ReplaceIdentityBuilder};
use yellowstone_shield_client::types::{Kind, PermissionStrategy};
use yellowstone_shield_client::{
    accounts::Policy,
    instructions::{AddIdentityBuilder, CreatePolicyBuilder, RemoveIdentityBuilder},
    CreateAccountBuilder, CreateAsscoiatedTokenAccountBuilder, InitializeMetadataBuilder,
    InitializeMint2Builder, MetadataPointerInitializeBuilder, TokenExtensionsMintToBuilder,
    TransactionBuilder,
};

#[tokio::test]
async fn test_policy_lifecycle() {
    let context = ProgramTest::new("yellowstone_shield", yellowstone_shield_client::ID, None)
        .start_with_context()
        .await;

    // Given a PDA derived from the payer's public key.
    let mint = Keypair::new();

    let payer_token_account = get_associated_token_address_with_program_id(
        &context.payer.pubkey(),
        &mint.pubkey(),
        &spl_token_2022::ID,
    );
    // Calculate the space required for the mint account with extensions.
    let mint_size =
        ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::MetadataPointer])
            .unwrap();

    let token_metadata = TokenMetadata {
        update_authority: OptionalNonZeroPubkey::try_from(Some(context.payer.pubkey())).unwrap(),
        mint: mint.pubkey(),
        name: "Test".to_string(),
        symbol: "TST".to_string(),
        uri: "https://test.com".to_string(),
        ..Default::default()
    };
    let rent = mint_size + token_metadata.tlv_size_of().unwrap();

    let create_mint_ix = CreateAccountBuilder::build()
        .payer(&context.payer.pubkey())
        .account(&mint.pubkey())
        .space(mint_size)
        .rent(rent)
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
        .freeze_authority(&context.payer.pubkey())
        .instruction();

    let init_metadata_ix = InitializeMetadataBuilder::new()
        .mint(&mint.pubkey())
        .owner(&context.payer.pubkey())
        .update_authority(&context.payer.pubkey())
        .mint_authority(&context.payer.pubkey())
        .name(token_metadata.name)
        .symbol(token_metadata.symbol)
        .uri(token_metadata.uri)
        .instruction();

    // Create the policy account.
    let address = Policy::find_pda(&mint.pubkey()).0;
    let create_policy_ix = CreatePolicyBuilder::new()
        .policy(address)
        .mint(mint.pubkey())
        .payer(context.payer.pubkey())
        .token_account(payer_token_account)
        .owner(context.payer.pubkey())
        .strategy(PermissionStrategy::Allow)
        .instruction();

    // Initialize the payer's token account.
    let create_payer_token_account_ix = CreateAsscoiatedTokenAccountBuilder::build()
        .mint(&mint.pubkey())
        .owner(&context.payer.pubkey())
        .payer(&context.payer.pubkey())
        .instruction();

    // Mint 1 token to the payer's token account.
    let mint_to_payer_ix = TokenExtensionsMintToBuilder::build()
        .mint(&mint.pubkey())
        .account(&payer_token_account)
        .owner(&context.payer.pubkey())
        .amount(1)
        .instruction();

    let tx = TransactionBuilder::build()
        .instruction(create_mint_ix)
        .instruction(init_metadata_pointer_ix)
        .instruction(init_mint_ix)
        .instruction(init_metadata_ix)
        .instruction(create_payer_token_account_ix)
        .instruction(mint_to_payer_ix)
        .instruction(create_policy_ix)
        .signer(&context.payer)
        .signer(&mint)
        .payer(&context.payer.pubkey())
        .recent_blockhash(context.last_blockhash)
        .transaction();

    context.banks_client.process_transaction(tx).await.unwrap();

    let policy_account = context.banks_client.get_account(address).await.unwrap();
    assert!(policy_account.is_some());

    let policy_account = policy_account.unwrap();
    let mut policy_account_data = policy_account.data.as_ref();

    let policy = Policy::deserialize(&mut policy_account_data).unwrap();

    assert_eq!(policy_account.data.len(), Policy::LEN);
    assert_eq!(policy.try_kind().unwrap(), Kind::Policy);
    assert_eq!(policy.try_strategy().unwrap(), PermissionStrategy::Allow);
    assert_eq!(policy.identities_len(), 0);

    let mint_account = context
        .banks_client
        .get_account(mint.pubkey())
        .await
        .unwrap();
    assert!(mint_account.is_some());

    let payer_token_account_data = context
        .banks_client
        .get_account(payer_token_account)
        .await
        .unwrap();
    assert!(payer_token_account_data.is_some());

    let payer_token_account_data = payer_token_account_data.unwrap();
    assert_eq!(payer_token_account_data.owner, spl_token_2022::ID);

    let first = Pubkey::new_unique();

    let first_push_identity_ix = AddIdentityBuilder::new()
        .policy(address)
        .mint(mint.pubkey())
        .payer(context.payer.pubkey())
        .owner(context.payer.pubkey())
        .token_account(payer_token_account)
        .identity(first)
        .instruction();

    let second = Pubkey::new_unique();

    let second_push_identity_ix = AddIdentityBuilder::new()
        .policy(address)
        .mint(mint.pubkey())
        .payer(context.payer.pubkey())
        .owner(context.payer.pubkey())
        .token_account(payer_token_account)
        .identity(second)
        .instruction();

    let tx = TransactionBuilder::build()
        .instruction(first_push_identity_ix)
        .instruction(second_push_identity_ix)
        .signer(&context.payer)
        .payer(&context.payer.pubkey())
        .recent_blockhash(context.last_blockhash)
        .transaction();

    context.banks_client.process_transaction(tx).await.unwrap();

    let policy_account = context.banks_client.get_account(address).await.unwrap();
    assert!(policy_account.is_some());

    let policy_account = policy_account.unwrap();
    let policy_account_data = policy_account.data;
    let policy = Policy::deserialize(&mut &policy_account_data[..Policy::LEN]).unwrap();

    assert_eq!(policy.identities_len(), 2);
    let identites = &policy_account_data[Policy::LEN..];

    let first_bytes = first.to_bytes();
    let second_bytes = second.to_bytes();

    let first_second_identities: Vec<u8> = first_bytes
        .iter()
        .chain(second_bytes.iter())
        .cloned()
        .collect();

    assert_eq!(identites, &first_second_identities[..]);

    let remove_identity_ix = RemoveIdentityBuilder::new()
        .policy(address)
        .mint(mint.pubkey())
        .payer(context.payer.pubkey())
        .owner(context.payer.pubkey())
        .token_account(payer_token_account)
        .index(0)
        .instruction();

    let tx = TransactionBuilder::build()
        .instruction(remove_identity_ix)
        .signer(&context.payer)
        .payer(&context.payer.pubkey())
        .recent_blockhash(context.last_blockhash)
        .transaction();

    context.banks_client.process_transaction(tx).await.unwrap();

    let policy_account = context.banks_client.get_account(address).await.unwrap();
    assert!(policy_account.is_some());

    let policy_account = policy_account.unwrap();
    let policy_account_data = policy_account.data;

    let policy = Policy::deserialize(&mut &policy_account_data[..Policy::LEN]).unwrap();

    assert_eq!(policy.identities_len(), 2);
    let identites = &policy_account_data[Policy::LEN..];

    let zeroed_bytes = [0u8; 32];
    let second_bytes = second.to_bytes();
    let expected_identities: Vec<u8> = zeroed_bytes
        .iter()
        .chain(second_bytes.iter())
        .cloned()
        .collect();

    assert_eq!(identites, &expected_identities);

    let replace_identity_ix = ReplaceIdentityBuilder::new()
        .policy(address)
        .mint(mint.pubkey())
        .payer(context.payer.pubkey())
        .owner(context.payer.pubkey())
        .token_account(payer_token_account)
        .index(0)
        .identity(first)
        .instruction();

    let tx = TransactionBuilder::build()
        .instruction(replace_identity_ix)
        .signer(&context.payer)
        .payer(&context.payer.pubkey())
        .recent_blockhash(context.last_blockhash)
        .transaction();

    context.banks_client.process_transaction(tx).await.unwrap();

    let policy_account = context.banks_client.get_account(address).await.unwrap();
    assert!(policy_account.is_some());

    let policy_account = policy_account.unwrap();
    let policy_account_data = policy_account.data;
    let policy = Policy::deserialize(&mut &policy_account_data[..Policy::LEN]).unwrap();

    assert_eq!(policy.identities_len(), 2);
    let identites = &policy_account_data[Policy::LEN..];

    assert_eq!(identites, &first_second_identities);

    let mint_account = context
        .banks_client
        .get_account(mint.pubkey())
        .await
        .unwrap();
    assert!(mint_account.is_some());

    let mint_account = mint_account.unwrap();
    let mint_account_data = mint_account.data;

    let pod_mint = PodStateWithExtensions::<PodMint>::unpack(&mint_account_data).unwrap();
    let mut mint_bytes = pod_mint.get_extension_bytes::<TokenMetadata>().unwrap();
    let token_metadata = TokenMetadata::try_from_slice(&mut mint_bytes).unwrap();

    assert_eq!(token_metadata.name, "Test".to_string());
    assert_eq!(token_metadata.symbol, "TST".to_string());
    assert_eq!(token_metadata.uri, "https://test.com".to_string());

    let close_policy = ClosePolicyBuilder::new()
        .policy(address)
        .mint(mint.pubkey())
        .payer(context.payer.pubkey())
        .owner(context.payer.pubkey())
        .token_account(payer_token_account)
        .instruction();

    let tx = TransactionBuilder::build()
        .instruction(close_policy)
        .signer(&context.payer)
        .payer(&context.payer.pubkey())
        .recent_blockhash(context.last_blockhash)
        .transaction();

    context.banks_client.process_transaction(tx).await.unwrap();

    let policy_account = context.banks_client.get_account(address).await.unwrap();
    assert!(policy_account.is_none());
}
