# Yellowstone Shield

<p align="center">
  <img src="yellowstone-shield.jpg" alt="Yellowstone Shield" style="max-width: 250px;">
</p>

Yellowstone Shield is a Solana program that manages on-chain allowlists and blocklists of identities. An identity can be any addressable account in Solana, such as a validator, wallet, or program. This program allows transaction senders, like Agave STS, Helius' Atlas, Mango's lite-rpc, Jito's blockEngine, and Triton's Jet, to effectively control transaction forwarding policies.

## Deployments

| Network | Program ID                                    |
| ------- | --------------------------------------------- |
| Mainnet | `b1ockYL7X6sGtJzueDbxRVBEEPN4YeqoLW276R3MX8W` |
| Devnet  | `b1ockYL7X6sGtJzueDbxRVBEEPN4YeqoLW276R3MX8W` |

## Why Use Yellowstone Shield?

- On-chain management allows retrieval and updates via standard Solana RPC methods.
- Supports updates via websocket/gRPC.
- Overcomes limitations of Solana's ALTs and Config programs.

## Solana RPC Integration

Yellowstone Shield integrates with Solana RPC by introducing:

- A new parameter, `forwardingPolicy`, in the `sendTransaction` RPC method, enabling users to specify allow/blocklists.
- An optional `Solana-ForwardingPolicy` HTTP header to support legacy clients.

Transaction senders interpret these policies to determine validator forwarding behavior, ensuring consistent enforcement of allow/blocklists across different RPC providers.

## SDKs and CLI

### SDKs

Clients are available for interacting with Yellowstone Shield:

- [JavaScript SDK](./clients/js)
- [Rust SDK](./clients/rust)

These SDKs facilitate easy integration and use of Yellowstone Shield in various applications and services.

### Command Line Tool (CLI)

A CLI tool is provided for convenient management of Yellowstone Shield policies, available in the `./cli` directory:

- [CLI Documentation](./cli/README.md)

This CLI allows creating policies, adding or removing identites, and managing configurations directly via terminal commands.

## Rust Policy Store

The Rust Policy Store provides efficient caching and quick retrieval of Yellowstone Shield policies, enabling real-time identity permission checks in transaction forwarders and RPC services. It ensures thread-safe access and updates with atomic snapshots. See the [Policy Store README](./policy-store/README.md) for detailed integration and usage instructions.

## Policy Bound to Token Extensions Asset

Policies are bound to a Token Extensions (TE) asset. Token holders can update identities tracked by the policy. The TE asset also contains metadata describing the policy:

- **Name**: Identifier of the policy.
- **Symbol**: Short representation of the policy.
- **URI**: Link to additional policy information.

The policy account uses a Program Derived Address (PDA), derived with the seed:

```
["shield", "policy", {mint_address}]
```

## Development

Install dependencies:

```sh
pnpm install
```

### Build and Test

```sh
pnpm programs:build
pnpm programs:test
pnpm programs:format
pnpm programs:lint
```

### Generate IDLs and Clients

```sh
pnpm generate:idls
pnpm generate:clients
```

### Local Validator Management

```sh
pnpm validator:start
pnpm validator:restart
pnpm validator:stop
```

## License

AGPL-3.0

## Developers

This project is developed by [Triton One](https://triton.one/).
