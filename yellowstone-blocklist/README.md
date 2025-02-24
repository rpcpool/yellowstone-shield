# Blocklists smart contract

Smart contract used to store the blocklist created by a client.

## How to deploy smart contract
1. First run
```
    cargo build-sbf
```
This command line argument comes with the Solana CLI.

2. After that, you must use
```
solana program deploy <PROGRAM_FILEPATH> --keypair <KEYPAIR_FILEPATH>
```
The keypair will be the owner and authority of the program to update it. In case you want to set another upgrade authority, it is possible to use --upgrade-authority. The keypair must be funded in SOL to pay the transaction and rent-exempt.

## How to update smart contract
1. To update a program, it is mandatory to use this command that creates a buffer-account
```
solana program write-buffer <PROGRAM_FILEPATH> --buffer-authority <BUFFER_AUTHORITY_FILEPATH>
```
where the buffer authority must be the same that the upgrade authority.

2. After using the first command, it'll return the buffer keypair and we have to use it with
```
solana program upgrade <BUFFER_ACCOUNT_KEYPAIR> <SMART_CONTRACT_PUBKEY> --upgrade-authority <UPGRADE_AUTHORITY_FILEPATH>
```
