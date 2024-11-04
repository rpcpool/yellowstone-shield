#[cfg(test)]
mod tests {
    use std::assert_eq;

    use crate::*;
    use borsh::BorshSerialize;
    use instruction::{
        AclPayload, DeleteListPayload, EditListPayload, ExtendListPayload, IndexPubkey,
        InitializeListPayload,
    };
    use pda::find_pda;
    use solana_sdk::signature::Keypair;
    use state::{AclType, EnumListState, ListState, ZEROED};
    use {
        solana_program::hash::Hash,
        solana_program_test::*,
        solana_sdk::{
            instruction::{AccountMeta, Instruction},
            pubkey::Pubkey,
            signature::Signer,
            system_program::ID as SYSTEM_PROGRAM_ID,
            transaction::Transaction,
        },
    };

    #[tokio::test]
    async fn test_sanity() {
        assert_eq!(true, true)
    }
    // Initialize account tests

    #[tokio::test]
    async fn test_initialize_1() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;

        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());
        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        // create Initialize instruction
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts,
            signing_keypairs,
        )
        .await
        .unwrap();

        // confirm state
        let state = banks_client
            .get_account_data_with_borsh::<EnumListState>(pda_key)
            .await
            .unwrap();

        match state {
            EnumListState::Uninitialized => panic!("It should be initialized"),
            EnumListState::ListStateV1(meta_list) => assert_eq!(meta_list.acl_type, AclType::Deny),
        }
    }

    // Check authority initialize
    #[tokio::test]
    async fn test_initialize_2() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());
        let authority = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(authority.pubkey(), true),
        ];
        let signing_keypairs = &[&payer, &authority];

        // create Initialize instruction
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts,
            signing_keypairs,
        )
        .await
        .unwrap();

        let state = banks_client
            .get_account_data_with_borsh::<EnumListState>(pda_key)
            .await
            .unwrap();

        match state {
            EnumListState::Uninitialized => panic!("It should be initialized"),
            EnumListState::ListStateV1(meta_list) => {
                assert_eq!(meta_list.authority, Some(authority.pubkey()))
            }
        }
    }

    // Transaction failed because trying to create second account
    #[tokio::test]
    async fn test_initialize_3() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer.insecure_clone()];
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let initialize_ix =
            instruction::ConfigInstructions::InitializeList(InitializeListPayload {
                acl_type: state::AclType::Allow,
            });
        let mut initialize_ix_data = Vec::new();
        initialize_ix.serialize(&mut initialize_ix_data).unwrap();

        let res = make_transaction(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            initialize_ix_data,
        )
        .await;
        assert!(res.is_err());
    }

    // Update account tests
    // Update list
    #[tokio::test]
    async fn test_extend_1() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        // create Initialize instruction
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let mut new_list = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        // Second list is to check that is possible to concat more pubkeys
        let mut concat_list = vec![Pubkey::new_unique()];

        extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            new_list.clone(),
        )
        .await
        .unwrap();
        extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            concat_list.clone(),
        )
        .await
        .unwrap();

        // send tx
        let data = banks_client
            .get_account(pda_key)
            .await
            .unwrap()
            .unwrap()
            .data;

        new_list.append(&mut concat_list);

        let state = ListState::deserialize(data.as_ref()).unwrap();

        assert_eq!(state.list, new_list.as_slice());
    }

    // Update with wrong authorization
    #[tokio::test]
    async fn test_extend_2() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let authority = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(authority.pubkey(), true),
        ];
        let signing_keypairs = &[&payer, &authority];

        let wrong_accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs_wrong = &[&payer];

        let new_list = vec![Pubkey::new_unique(), Pubkey::new_unique()];

        // create Initialize instruction
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts,
            signing_keypairs,
        )
        .await
        .unwrap();

        let res = extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            wrong_accounts.clone(),
            signing_keypairs_wrong,
            new_list.clone(),
        )
        .await;

        // send tx
        assert!(res.is_err());
    }

    // Update with uninitialized account
    #[tokio::test]
    async fn test_extend_3() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let authority = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(authority.pubkey(), true),
        ];
        let signing_keypairs = &[&payer, &authority];

        let new_list = vec![Pubkey::new_unique(), Pubkey::new_unique()];

        let res = extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts,
            signing_keypairs,
            new_list.clone(),
        )
        .await;

        // send tx
        assert!(res.is_err());
    }

    // Update freeze account
    #[tokio::test]
    async fn test_extend_4() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let authority = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(authority.pubkey(), true),
        ];
        let signing_keypairs = &[&payer, &authority];

        // create Initialize instruction
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        freeze_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        // send tx
        let new_list = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let res = extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts,
            signing_keypairs,
            new_list.clone(),
        )
        .await;

        // send tx
        assert!(res.is_err());
    }

    // Acl type tests with uninitialized account
    #[tokio::test]
    async fn test_update_acl_1() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let update_acl_type = instruction::ConfigInstructions::UpdateAclType(AclPayload {
            acl_type: state::AclType::Allow,
        });
        let mut update_acl_type_data = Vec::new();
        update_acl_type
            .serialize(&mut update_acl_type_data)
            .unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(pda_key, false),
                    AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                ],
                data: update_acl_type_data,
            }],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );

        // send tx
        assert!(banks_client.process_transaction(transaction).await.is_err());
    }

    // Freeze acl update
    #[tokio::test]
    async fn test_update_acl_2() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();
        freeze_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let update_acl_type = instruction::ConfigInstructions::UpdateAclType(AclPayload {
            acl_type: state::AclType::Allow,
        });
        let mut update_acl_type_data = Vec::new();
        update_acl_type
            .serialize(&mut update_acl_type_data)
            .unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[Instruction {
                program_id,
                accounts,
                data: update_acl_type_data,
            }],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );

        // send tx
        assert!(banks_client.process_transaction(transaction).await.is_err());
    }

    // Wrong  authorization acl update
    #[tokio::test]
    async fn test_update_acl_3() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());
        let authority = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(authority.pubkey(), true),
        ];
        let signing_keypairs = &[&payer, &authority];

        let wrong_accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(payer.pubkey(), true),
        ];
        let signing_keypairs_wrong = &[&payer];

        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();
        let update_acl_type = instruction::ConfigInstructions::UpdateAclType(AclPayload {
            acl_type: state::AclType::Allow,
        });
        let mut update_acl_type_data = Vec::new();
        update_acl_type
            .serialize(&mut update_acl_type_data)
            .unwrap();

        let res = make_transaction(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            wrong_accounts,
            signing_keypairs_wrong,
            update_acl_type_data,
        )
        .await;

        // send tx
        assert!(res.is_err());
    }

    // Wrong  authorization acl update
    #[tokio::test]
    async fn test_update_acl_4() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());
        let authority = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(authority.pubkey(), true),
        ];
        let signing_keypairs = &[&payer, &authority];

        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();
        let update_acl_type = instruction::ConfigInstructions::UpdateAclType(AclPayload {
            acl_type: state::AclType::Allow,
        });
        let mut update_acl_type_data = Vec::new();
        update_acl_type
            .serialize(&mut update_acl_type_data)
            .unwrap();

        make_transaction(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts,
            signing_keypairs,
            update_acl_type_data,
        )
        .await
        .unwrap();

        let data = banks_client
            .get_account(pda_key)
            .await
            .unwrap()
            .unwrap()
            .data;

        let state = ListState::deserialize(data.as_ref()).unwrap();
        // send tx
        assert_eq!(state.meta.acl_type, AclType::Allow);
    }
    // Freeze account tests
    #[tokio::test]
    async fn test_freeze_1() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());
        let authority = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(authority.pubkey(), true),
        ];
        let signing_keypairs = &[&payer, &authority];

        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();
        freeze_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();
        let data = banks_client
            .get_account(pda_key)
            .await
            .unwrap()
            .unwrap()
            .data;

        let state = ListState::deserialize(data.as_ref()).unwrap();
        // send tx
        assert_eq!(state.meta.authority, None);
    }

    // Wrong authority
    #[tokio::test]
    async fn test_freeze_2() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());
        let authority = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(authority.pubkey(), true),
        ];
        let signing_keypairs = &[&payer, &authority];

        let wrong_accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(payer.pubkey(), true),
        ];
        let signing_keypairs_wrong = &[&payer];

        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let res = freeze_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            wrong_accounts,
            signing_keypairs_wrong,
        )
        .await;

        assert!(res.is_err());
    }

    // Not initialized
    #[tokio::test]
    async fn test_freeze_3() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());
        let authority = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(authority.pubkey(), true),
        ];
        let signing_keypairs = &[&payer, &authority];

        let freeze_account = instruction::ConfigInstructions::FreezeAccount;
        let mut freeze_account_data = Vec::new();
        freeze_account.serialize(&mut freeze_account_data).unwrap();

        let res = make_transaction(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts,
            signing_keypairs,
            freeze_account_data,
        )
        .await;

        assert!(res.is_err());
    }

    // Freeze already freeze account
    #[tokio::test]
    async fn test_freeze_4() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());
        let authority = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(authority.pubkey(), true),
        ];
        let signing_keypairs = &[&payer, &authority];

        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        freeze_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let hash: Hash = banks_client.get_latest_blockhash().await.unwrap();

        let res = freeze_acc(
            program_id,
            payer.insecure_clone(),
            hash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await;

        assert!(res.is_err())
    }

    // Close account
    #[tokio::test]
    async fn test_close_1() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());
        let get_pay = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();
        let pda_lamports = banks_client.get_balance(pda_key).await.unwrap();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new(get_pay.pubkey(), false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        close_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts,
            signing_keypairs,
        )
        .await
        .unwrap();

        let acc_closed_lamports = banks_client.get_balance(get_pay.pubkey()).await.unwrap();

        assert_eq!(acc_closed_lamports, pda_lamports);
    }

    // Close account wrong authorization
    #[tokio::test]
    async fn test_close_2() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());
        let get_pay = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new(get_pay.pubkey(), false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(get_pay.pubkey(), true),
        ];

        let signing_keypairs = &[&payer, &get_pay];

        let res = close_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts,
            signing_keypairs,
        )
        .await;

        assert!(res.is_err());
    }

    // Close account freeze
    #[tokio::test]
    async fn test_close_3() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());
        let get_pay = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        freeze_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new(get_pay.pubkey(), false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];

        let signing_keypairs = &[&payer];

        let res = close_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts,
            signing_keypairs,
        )
        .await;

        assert!(res.is_err());
    }

    // Close account uninitialized
    #[tokio::test]
    async fn test_close_4() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());
        let get_pay = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new(get_pay.pubkey(), false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];

        let signing_keypairs = &[&payer];

        let res = close_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts,
            signing_keypairs,
        )
        .await;

        assert!(res.is_err());
    }

    // Update list
    #[tokio::test]
    async fn test_update_1() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        // create Initialize instruction
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let new_list = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let update_list = vec![IndexPubkey {
            index: 0,
            key: Pubkey::new_unique(),
        }];
        let cmp_list = vec![update_list[0].key, new_list[1]];

        extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            new_list.clone(),
        )
        .await
        .unwrap();

        update_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            update_list.clone(),
        )
        .await
        .unwrap();

        let data = banks_client
            .get_account(pda_key)
            .await
            .unwrap()
            .unwrap()
            .data;

        let state = ListState::deserialize(data.as_ref()).unwrap();

        assert_eq!(state.list, cmp_list);
    }

    // Check for wrong index

    #[tokio::test]
    async fn test_update_2() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        // create Initialize instruction
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let new_list = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let update_list = vec![IndexPubkey {
            index: 3,
            key: Pubkey::new_unique(),
        }];

        extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            new_list.clone(),
        )
        .await
        .unwrap();

        let res = update_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            update_list.clone(),
        )
        .await;

        assert!(res.is_err());
    }

    // Check for wrong authority
    #[tokio::test]
    async fn test_update_3() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let authority = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(authority.pubkey(), true),
        ];
        let signing_keypairs = &[&payer, &authority];

        let wrong_accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs_wrong = &[&payer];

        // create Initialize instruction
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let new_list = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let update_list = vec![IndexPubkey {
            index: 0,
            key: Pubkey::new_unique(),
        }];

        extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            new_list,
        )
        .await
        .unwrap();

        let res = update_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            wrong_accounts,
            signing_keypairs_wrong,
            update_list.clone(),
        )
        .await;

        assert!(res.is_err());
    }

    // Check for freeze account
    #[tokio::test]
    async fn test_update_4() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        // create Initialize instruction
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let new_list = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let update_list = vec![IndexPubkey {
            index: 0,
            key: Pubkey::new_unique(),
        }];

        extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            new_list.clone(),
        )
        .await
        .unwrap();

        freeze_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let res = update_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            update_list.clone(),
        )
        .await;

        assert!(res.is_err());
    }

    // Check for uninitialized account

    #[tokio::test]
    async fn test_update_5() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        // create Initialize instruction
        let update_list = vec![IndexPubkey {
            index: 0,
            key: Pubkey::new_unique(),
        }];

        let res = update_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            update_list.clone(),
        )
        .await;

        assert!(res.is_err());
    }

    // Delete items

    #[tokio::test]
    async fn test_remove_1() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        // create Initialize instruction
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let new_list = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let remove = 0;
        let cmp_list = vec![Pubkey::from(ZEROED), new_list[1]];

        extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            new_list.clone(),
        )
        .await
        .unwrap();

        delete_item_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            remove,
        )
        .await
        .unwrap();

        let data = banks_client
            .get_account(pda_key)
            .await
            .unwrap()
            .unwrap()
            .data;

        let state = ListState::deserialize(data.as_ref()).unwrap();

        assert_eq!(state.list, cmp_list);
    }

    // Wrong index at deletion
    #[tokio::test]
    async fn test_remove_2() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        // create Initialize instruction
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let new_list = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let remove = 3;

        extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            new_list.clone(),
        )
        .await
        .unwrap();

        let res = delete_item_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            remove,
        )
        .await;

        assert!(res.is_err());
    }

    // Wrong authority
    #[tokio::test]
    async fn test_remove_3() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let authority = Keypair::new();

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new(authority.pubkey(), true),
        ];
        let signing_keypairs = &[&payer, &authority];

        let wrong_accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs_wrong = &[&payer];

        // create Initialize instruction
        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let new_list = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let remove = 0;

        extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            new_list,
        )
        .await
        .unwrap();

        let res = delete_item_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            wrong_accounts,
            signing_keypairs_wrong,
            remove,
        )
        .await;

        assert!(res.is_err());
    }

    // Freeze account
    #[tokio::test]
    async fn test_remove_4() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        initialize_account(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let new_list = vec![Pubkey::new_unique(), Pubkey::new_unique()];
        let remove = 0;

        extend_list_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            new_list.clone(),
        )
        .await
        .unwrap();

        freeze_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
        )
        .await
        .unwrap();

        let res = delete_item_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            remove,
        )
        .await;

        assert!(res.is_err());
    }

    // Check for uninitialized account

    #[tokio::test]
    async fn test_remove_5() {
        let (program_id, mut banks_client, payer, recent_blockhash) = start_program_test().await;
        // create counter
        let (pda_key, _) = find_pda(&program_id, &payer.pubkey());

        let accounts = vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_key, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];
        let signing_keypairs = &[&payer];

        let remove = 0;

        let res = delete_item_acc(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            &mut banks_client,
            accounts.clone(),
            signing_keypairs,
            remove,
        )
        .await;

        assert!(res.is_err());
    }

    // Helper functions
    async fn start_program_test() -> (Pubkey, BanksClient, Keypair, Hash) {
        let program_id = Pubkey::new_unique();
        let program_test = ProgramTest::new(
            // .so fixture is  retrieved from /target/deploy
            "solana_yellowstone_blocklist",
            program_id,
            // shank is incompatible with instantiating the BuiltInFunction
            None,
        );

        let (banks_client, payer, recent_blockhash) = program_test.start().await;
        (program_id, banks_client, payer, recent_blockhash)
    }

    async fn initialize_account(
        program_id: Pubkey,
        payer: Keypair,
        recent_blockhash: Hash,
        banks_client: &mut BanksClient,
        accounts: Vec<AccountMeta>,
        signing_keypairs: &[&Keypair],
    ) -> Result<(), BanksClientError> {
        let initialize_ix =
            instruction::ConfigInstructions::InitializeList(InitializeListPayload {
                acl_type: state::AclType::Deny,
            });
        let mut initialize_ix_data = Vec::new();
        initialize_ix.serialize(&mut initialize_ix_data).unwrap();

        return make_transaction(
            program_id,
            payer,
            recent_blockhash,
            banks_client,
            accounts,
            signing_keypairs,
            initialize_ix_data,
        )
        .await;
    }

    async fn make_transaction(
        program_id: Pubkey,
        payer: Keypair,
        recent_blockhash: Hash,
        banks_client: &mut BanksClient,
        accounts: Vec<AccountMeta>,
        signing_keypairs: &[&Keypair],
        instruction: Vec<u8>,
    ) -> Result<(), BanksClientError> {
        let transaction = Transaction::new_signed_with_payer(
            &[Instruction {
                program_id,
                accounts,
                data: instruction,
            }],
            Some(&payer.pubkey()),
            signing_keypairs,
            recent_blockhash,
        );
        return banks_client.process_transaction(transaction).await;
    }

    async fn extend_list_acc(
        program_id: Pubkey,
        payer: Keypair,
        recent_blockhash: Hash,
        banks_client: &mut BanksClient,
        accounts: Vec<AccountMeta>,
        signing_keypairs: &[&Keypair],
        list: Vec<Pubkey>,
    ) -> Result<(), BanksClientError> {
        let extend_list = instruction::ConfigInstructions::ExtendList(ExtendListPayload { list });
        let mut extend_list_data = Vec::new();
        extend_list.serialize(&mut extend_list_data).unwrap();

        return make_transaction(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            banks_client,
            accounts,
            signing_keypairs,
            extend_list_data,
        )
        .await;
    }

    async fn update_list_acc(
        program_id: Pubkey,
        payer: Keypair,
        recent_blockhash: Hash,
        banks_client: &mut BanksClient,
        accounts: Vec<AccountMeta>,
        signing_keypairs: &[&Keypair],
        list: Vec<IndexPubkey>,
    ) -> Result<(), BanksClientError> {
        let update_list = instruction::ConfigInstructions::UpdateList(EditListPayload { list });
        let mut update_list_data = Vec::new();
        update_list.serialize(&mut update_list_data).unwrap();

        return make_transaction(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            banks_client,
            accounts,
            signing_keypairs,
            update_list_data,
        )
        .await;
    }

    async fn freeze_acc(
        program_id: Pubkey,
        payer: Keypair,
        recent_blockhash: Hash,
        banks_client: &mut BanksClient,
        accounts: Vec<AccountMeta>,
        signing_keypairs: &[&Keypair],
    ) -> Result<(), BanksClientError> {
        let freeze_instruction = instruction::ConfigInstructions::FreezeAccount;
        let mut update_freeze_data = Vec::new();
        freeze_instruction
            .serialize(&mut update_freeze_data)
            .unwrap();

        return make_transaction(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            banks_client,
            accounts.clone(),
            signing_keypairs,
            update_freeze_data,
        )
        .await;
    }

    async fn delete_item_acc(
        program_id: Pubkey,
        payer: Keypair,
        recent_blockhash: Hash,
        banks_client: &mut BanksClient,
        accounts: Vec<AccountMeta>,
        signing_keypairs: &[&Keypair],
        index: usize,
    ) -> Result<(), BanksClientError> {
        let delete_list =
            instruction::ConfigInstructions::RemoveItemList(DeleteListPayload { index });
        let mut delete_list_data = Vec::new();
        delete_list.serialize(&mut delete_list_data).unwrap();

        return make_transaction(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            banks_client,
            accounts,
            signing_keypairs,
            delete_list_data,
        )
        .await;
    }

    async fn close_acc(
        program_id: Pubkey,
        payer: Keypair,
        recent_blockhash: Hash,
        banks_client: &mut BanksClient,
        accounts: Vec<AccountMeta>,
        signing_keypairs: &[&Keypair],
    ) -> Result<(), BanksClientError> {
        let close_account = instruction::ConfigInstructions::CloseAccount;
        let mut close_account_data = Vec::new();
        close_account.serialize(&mut close_account_data).unwrap();

        return make_transaction(
            program_id,
            payer.insecure_clone(),
            recent_blockhash,
            banks_client,
            accounts.clone(),
            signing_keypairs,
            close_account_data,
        )
        .await;
    }
}
