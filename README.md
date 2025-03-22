# Yellowstone Shield

<p align="center">
  <img src="yellowstone-shield.jpg" alt="Yellowstone Shield" style="max-width: 250px;">
</p>

Yellowstone Shield is a Solana program designed to manage allowlists and blocklists of validator identities directly on-chain. It enables transaction senders, such as Agave STS, Helius' Atlas, Mango's lite-rpc, Jito's blockEngine, and Triton's Jet, to control transaction forwarding policies effectively.

## Deployments

| Network | Program ID                                     |
| ------- | ---------------------------------------------- |
| Mainnet | `b1ockYL7X6sGtJzueDbxRVBEEPN4YeqoLW276R3MX8W`  |
| Devnet  | `E47XuTVcCMMYZSmeaybv3BMu6hYktvvFgeZkAJbVgmUx` |

## Why Use Yellowstone Shield?

- On-chain management allows retrieval and updates via standard Solana RPC methods.
- Supports updates via websocket/gRPC.
- Overcomes limitations of Solana's ALTs and Config programs.

## Integration

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

This CLI allows creating policies, adding or removing validators, and managing configurations directly via terminal commands.

## Policy Bound to Token Extensions Asset

Policies are bound to a Token Extensions (TE) asset. Token holders can update validator identities tracked by the policy. The TE asset also contains metadata describing the policy:

- **Name**: Identifier of the policy.
- **Symbol**: Short representation of the policy.
- **URI**: Link to additional policy information.

### Binding Policy to Token Extensions Asset

The policy account uses a Program Derived Address (PDA), derived with the seed:

```
["shield", "policy", {mint_address}]
```

Here, `{mint_address}` is the mint address of the associated Token Extensions asset.

---

## Program State

### Policy Account

Each policy account stores:

- `kind`: Always set to `Policy`.
- `strategy`: Either `Allow` or `Deny`.
- `validatorIdentities`: Validator public keys.

## Instructions

### CreatePolicy

Creates a policy account.

**Accounts:**

- `mint`: Token mint for policy management.
- `tokenAccount`: Authority token account.
- `policy`: Policy account (mutable).
- `payer`: Fee-paying account (mutable, signer).
- `systemProgram`: System program.
- `tokenProgram`: SPL token program.

**Arguments:**

- `strategy`: `Allow` or `Deny`.
- `validatorIdentities`: Validator public keys.

### AddIdentity

Adds a validator to a policy.

**Accounts:**

- `mint`: Token mint.
- `tokenAccount`: Authority account.
- `policy`: Policy account (mutable).
- `payer`: Fee-paying account (mutable, signer).
- `systemProgram`: System program.

**Arguments:**

- `validatorIdentity`: Validator public key.

### RemoveIdentity

Removes a validator from a policy.

**Accounts:**

- `mint`: Token mint.
- `tokenAccount`: Authority account.
- `policy`: Policy account (mutable).
- `payer`: Fee-paying account (mutable, signer).
- `systemProgram`: System program.

**Arguments:**

- `validatorIdentity`: Validator public key.

## Errors

Common errors:

- `DeserializationError`
- `SerializationError`
- `InvalidPda`
- `ValidatorIdentityNotFound`
- `InvalidAssociatedTokenAccount`

## Development Setup

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

This project is developded by [Triton One](https://triton.one/).
