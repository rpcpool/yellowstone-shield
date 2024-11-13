# jet-cli Commands

jet-cli is a tool to interact with the allow/deny list program. This tool needs a configuration file. It can be used the following way or by creating the config_list.yaml in the root:

```
jet-cli --config <path>
```

It can use authority, payer, rpc-url and program-id arguments to bypass the ones established in the config.
Example:

```
jet-cli --authority <path/keypair> --payer <path/keypair> --rpc-url <url> --program-id <pubkey>
```

The CLI has 9 actions to perform: 

- Initialize: starts the account with a certain type of list.
Example: 
```
jet-cli initialize <deny/allow>
```
- Add: update any item inside the pubkey passing the index and pubkey
```
jet-cli add <path/pubkey>
```
- Delete: remove items from the list
```
jet-cli delete <path/pubkey>
```
- Close: close account and transfer lamports in PDA account to desired recipient account
```
jet-cli close
```
- Freeze
```
jet-cli freeze
```
- Update-acl: updates list type
```
jet-cli update-acl <deny/allow> 
```
- State: shows PDA account state. 
```
jet-cli state
// There is an optional argument to parse data from a different account than the PDA calculated by payer keypair, is called using --pubkey <pubkey>
```
- Pda-key: shows PDA pubkey
```
jet-cli pda-key
```

NOTE: remember that if you do not have a config_list.yml or do not provide the flag `--config <path>`, each of the actions must receive the following parameters:
`--authority <path/keypair>`
`--payer <path/keypair>`
`--rpc-url <url>`
`--program-id <pubkey>`