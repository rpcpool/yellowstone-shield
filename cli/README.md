# Yellowstone Shield CLI

## Overview

The Yellowstone Shield CLI is a command-line tool for managing access policies for Solana identities, such as validators, wallets, or programs. It allows users to create and manage policies, add or remove identites, and configure various settings related to the policy.

## Installation

To install the CLI, ensure you have Rust and Cargo installed on your system. Then, clone the repository and build the project:

```bash
git clone https://github.com/rpcpool/yellowstone-shield
cd yellowstone-shield
cargo build --release --bin cli
```

## Usage

The CLI provides several commands to interact with the shield policies and identities. Below are the available commands and their options:

### General Options

- `-r, --rpc <URL>`: Specify the RPC endpoint URL.
- `-T, --timeout <SECONDS>`: Set the timeout duration (default is 90 seconds).
- `-l, --log-level <LEVEL>`: Set the log level (default is "off").
- `-k, --keypair <FILE>`: Path to the owner keypair file.

### Commands

- **Create a Policy**

  ```bash
  yellowstone-shield policy create [OPTIONS] --strategy <STRATEGY> --name <NAME> --symbol <SYMBOL> --uri <URI>
  ```

  - `-r, --rpc <RPC>`: RPC endpoint URL to override using the Solana config.
  - `--strategy <STRATEGY>`: The strategy to use for the policy.
  - `-l, --log-level <LOG_LEVEL>`: Log level (default is "off").
  - `--name <NAME>`: The name of the policy.
  - `-k, --keypair <KEYPAIR>`: Path to the owner keypair file.
  - `--symbol <SYMBOL>`: The symbol of the policy.
  - `--uri <URI>`: The URI of the policy.
  - `-h, --help`: Print help.

- **Add Identities**

  ```bash
  yellowstone-shield identities add --mint <MINT> --identities-path <IDENTITIES>
  ```

  - `--mint <MINT>`: The mint address linked to the shield policy.
  - `--identities-path <IDENTITIES>`: File path to a list of public keys, each on a new line, to be added.

- **Remove Identities**

  ```bash
  yellowstone-shield identities remove --mint <MINT> --identities-path <IDENTITIES>
  ```

  - `--mint <MINT>`: The mint address linked to the shield policy.
  - `--identities-path <IDENTITIES>`: File path to a list of public keys, each on a new line, to be removed.

## Configuration

The CLI uses the Solana CLI configuration file to manage RPC endpoints and keypair paths. You can override these settings using the command-line options provided.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.

## License

This project is licensed under the MIT License. See the [LICENSE](../LICENSE) file for details.

## Contact

For questions or support, please open an issue on github.
