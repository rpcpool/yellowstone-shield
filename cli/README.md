# Yellowstone Shield CLI

A command-line interface for managing Yellowstone Shield access control policies on Solana. This tool enables you to create permission-based policies that control access for validators, wallets, and programs.

## Overview

Yellowstone Shield provides a token-based access control system where policies are tied to SPL token mints. Each policy can maintain a list of authorized identities (public keys) with either Allow or Deny strategies.

## Installation

### Prerequisites
- Rust and Cargo (latest stable version)
- Solana CLI tools configured with a valid RPC endpoint
- A funded Solana wallet for transaction fees

### Build from Source

```bash
git clone https://github.com/rpcpool/yellowstone-shield
cd yellowstone-shield
cargo build --release --bin yellowstone-shield-cli
```

The binary will be available at `target/release/yellowstone-shield-cli`

## Configuration

The CLI uses your Solana CLI configuration by default. Ensure you have:

```bash
# Set your RPC endpoint
solana config set --url https://api.mainnet-beta.solana.com

# Set your keypair
solana config set --keypair ~/.config/solana/id.json
```

## Usage

### Global Options

- `-r, --rpc <URL>` - Override the RPC endpoint from Solana config
- `-k, --keypair <PATH>` - Override the keypair path from Solana config  
- `-l, --log-level <LEVEL>` - Set log verbosity (default: "off")

### Policy Commands

#### Create a Policy

Create a new access control policy with metadata:

```bash
# Create an Allow policy
yellowstone-shield-cli policy create \
  --strategy Allow \
  --name "Validator Access Policy" \
  --symbol "VAP" \
  --uri "https://example.com/policy-metadata.json"

# Create a Deny policy (blocklist)
yellowstone-shield-cli policy create \
  --strategy Deny \
  --name "Restricted Access" \
  --symbol "BLOCK" \
  --uri "https://example.com/blocklist.json"
```

**Parameters:**
- `--strategy` - Permission strategy: `Allow` (whitelist) or `Deny` (blocklist)
- `--name` - Human-readable policy name
- `--symbol` - Short identifier (like a token symbol)
- `--uri` - Metadata URI for additional policy information

#### Show Policy Details

Display policy information and list all authorized identities:

```bash
yellowstone-shield-cli policy show \
  --mint 7xKXtg2CW87d7TXQ3xgBwSEGD6YA1F3PtdxqMtfqdW4m
```

#### Delete a Policy

Remove a policy (requires ownership):

```bash
yellowstone-shield-cli policy delete \
  --mint 7xKXtg2CW87d7TXQ3xgBwSEGD6YA1F3PtdxqMtfqdW4m
```

### Identity Management Commands

#### Add Identities

Add authorized identities to a policy from a file:

```bash
# Create a file with pubkeys (one per line)
cat > validators.txt << EOF
DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh
ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt
CiDwVBFgWV9E5MvXWoLgnEgn2hK7rJikbvfWavzAQz3
EOF

# Add all identities to the policy
yellowstone-shield-cli identities add \
  --mint 7xKXtg2CW87d7TXQ3xgBwSEGD6YA1F3PtdxqMtfqdW4m \
  --identities-path validators.txt
```

The command will:
- Skip identities that are already in the policy
- Process additions in batches of 20 for efficiency
- Show transaction signatures for each batch

#### Remove Identities

Remove identities from a policy:

```bash
# Create a file with pubkeys to remove
cat > remove_list.txt << EOF
ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt
EOF

yellowstone-shield-cli identities remove \
  --mint 7xKXtg2CW87d7TXQ3xgBwSEGD6YA1F3PtdxqMtfqdW4m \
  --identities-path remove_list.txt
```

## Example Workflows

### 1. Validator Access Control

Create a whitelist for authorized validators:

```bash
# Create policy
yellowstone-shield-cli policy create \
  --strategy Allow \
  --name "Mainnet Validators" \
  --symbol "MVAL" \
  --uri "https://validators.example.com/metadata.json"

# Note the mint address from output
# Add validators
yellowstone-shield-cli identities add \
  --mint <MINT_ADDRESS> \
  --identities-path mainnet_validators.txt
```

### 2. Program Blocklist

Create a blocklist for restricted programs:

```bash
# Create deny policy
yellowstone-shield-cli policy create \
  --strategy Deny \
  --name "Restricted Programs" \
  --symbol "DENY" \
  --uri "https://security.example.com/blocklist.json"

# Add restricted program IDs
yellowstone-shield-cli identities add \
  --mint <MINT_ADDRESS> \
  --identities-path restricted_programs.txt
```

### 3. Dynamic Access Management

Update access lists programmatically:

```bash
#!/bin/bash
POLICY_MINT="7xKXtg2CW87d7TXQ3xgBwSEGD6YA1F3PtdxqMtfqdW4m"

# Add new validators
yellowstone-shield-cli identities add \
  --mint $POLICY_MINT \
  --identities-path new_validators.txt

# Remove deactivated validators  
yellowstone-shield-cli identities remove \
  --mint $POLICY_MINT \
  --identities-path removed_validators.txt

# Show current state
yellowstone-shield-cli policy show --mint $POLICY_MINT
```

## Output Format

The CLI provides formatted output with emojis for better readability:

```
📜 Policy
--------------------------------
🏠 Addresses
  📜 Policy: 5we4Bk6DxGMnMbrUMmVpLjgyHrqh7k7F4vhYVzkeQcH2
  🔑 Mint: 7xKXtg2CW87d7TXQ3xgBwSEGD6YA1F3PtdxqMtfqdW4m
--------------------------------
🔍 Details
  ✅ Strategy: Allow
  🏷️ Name: Validator Access Policy
  🔖 Symbol: VAP
  🌐 URI: https://example.com/policy-metadata.json
--------------------------------
```

## Best Practices

1. **Batch Operations**: When adding/removing many identities, use files to batch operations
2. **Backup Mint Addresses**: Save policy mint addresses for future reference
3. **Metadata URIs**: Host policy metadata on IPFS or Arweave for permanence
4. **Access Strategy**: Choose between Allow (whitelist) and Deny (blocklist) based on your security model
5. **Regular Audits**: Use `policy show` to regularly audit access lists

## Troubleshooting

### Common Issues

1. **Insufficient SOL**: Ensure your wallet has enough SOL for transaction fees
2. **RPC Errors**: Try using a different RPC endpoint with `-r` flag
3. **Large Identity Lists**: Files are processed in batches of 20 to avoid transaction size limits
4. **Permission Errors**: Only the policy owner can modify identities

## License

This project is licensed under the AGPL-3.0 License. See the [LICENSE](../LICENSE) file for details.

## Support

- 📚 [Documentation](https://docs.triton.one/shield-transaction-policies)
- 🐛 [Issue Tracker](https://github.com/rpcpool/yellowstone-shield/issues)