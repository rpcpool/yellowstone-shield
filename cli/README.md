# Yellowstone Shield CLI

## Overview

The Yellowstone Shield CLI is a command-line tool for managing access policies for Solana validators. It allows users to create and manage policies, add or remove validators, and configure various settings related to the policy.

## Installation

To install the CLI, ensure you have Rust and Cargo installed on your system. Then, clone the repository and build the project:

```bash
git clone https://github.com/rpcpool/yellowstone-shield
cd yellowstone-shield
cargo build --release --bin cli
```

## Usage

The CLI provides several commands to interact with the shield policies and validators. Below are the available commands and their options:

### General Options

- `-r, --rpc <URL>`: Specify the RPC endpoint URL.
- `-T, --timeout <SECONDS>`: Set the timeout duration (default is 90 seconds).
- `-l, --log-level <LEVEL>`: Set the log level (default is "off").
- `-k, --keypair <FILE>`: Path to the owner keypair file.

### Commands

#### Policy Management

- **Create a Policy**

  ```bash
  yellowstone-shield policy create --strategy <STRATEGY> --validator-identities <IDENTITIES>
  ```

  - `--strategy <STRATEGY>`: The strategy to use for the policy.
  - `--validator-identities <IDENTITIES>`: Comma-separated list of validator identities to add to the policy.

#### Validator Management

- **Add a Validator**

  ```bash
  yellowstone-shield validator add --policy <POLICY> --mint <MINT> --validator-identity <IDENTITY>
  ```

  - `--policy <POLICY>`: The policy to which the validator will be added.
  - `--mint <MINT>`: The mint address associated with the policy.
  - `--validator-identity <IDENTITY>`: The validator to add to the policy.

- **Remove a Validator**

  ```bash
  yellowstone-shield validator remove --policy <POLICY> --mint <MINT> --validator-identity <IDENTITY>
  ```

  - `--policy <POLICY>`: The policy from which the validator will be removed.
  - `--mint <MINT>`: The mint address associated with the policy.
  - `--validator-identity <IDENTITY>`: The validator to remove from the policy.

## Configuration

The CLI uses the Solana CLI configuration file to manage RPC endpoints and keypair paths. You can override these settings using the command-line options provided.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.

## License

This project is licensed under the MIT License. See the [LICENSE](../LICENSE) file for details.

## Contact

For questions or support, please open an issue on github.
