# Transaction Allow/Blocklists

Program: `b1ockYL7X6sGtJzueDbxRVBEEPN4YeqoLW276R3MX8W`

This repo contains a Solana program and associated CLI for maintaining allow and blocklists on chain. An allowlist or blocklists contains a set of pubkeys and the associated action (allow/deny).

The intended usage is for transaction senders (whether the Agave Solana STS service or specialised senders like Helius' atlas, Mango's lite-rpc, Jito's blockEngine or Triton's Jet) to be able to read these block lists and apply these as filters for which downstream validators they are willing to forward transactions to.

### Why custom program?

The idea with using a separate program for this on-chain provides a couple of benefits:

1. Transaction senders can cache all the possible allow/blocklists by doing getProgramAccounts
2. Allow/blocklists can be kept updated via websocket/grpc
3. For integration into Agave, there is already a bank available to access allow/blocklists similar to ALTs

### Why not ALTs or the Config program?

Both of these programs have limitations that make them less suitable for this type of list, including not allowing partial updates/expansions of lists.

## Proposed RPC interface for integrating with this program

Currently, discussions on changes to the RPC interface are ongoing. Please feel free to leave issue comments on this repository regarding the RPC interface. The goal is to standardise across providers so that end-users and developers can submit transactions, having the lists respected, regardless of which provider they use.

The present suggestions are:

### Addition of a parameter to sendTransaction

`forwardingPolicy`: list of forwarding policies to apply to the sending of this transaction

The parameter could take values in the form of a URI or a list of account IDs. The list will have to be flattened by the transaction sender into a final filter that can be used to choose which validators to send to. 

The transaction sender would need to interpret this list. In the case of URI, a separate format for this content would need to be agreed upon (proposed: JSON URI).

```
curl https://api.devnet.solana.com -X POST -H "Content-Type: application/json" -d '
  {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "sendTransaction",
    "params": [
      "4hXTCkRzt9WyecNzV1XPgCDfGAZzQKNxLXgynz5QDuWWPSAZBZSHptvWRL3BjCvzUXRdKvHL2b7yGrRQcWyaqsaBCncVG7BFggS8w9snUts67BSh3EqKpXLUm5UMHfD7ZBe9GhARjbNQMLJ1QD3Spr6oMTBU6EhdB4RD8CP2xUxr2u3d6fos36PD98XS6oX8TQjLpsMwncs5DAMiD4nNnR8NBfyghGCWvCVifVwvA8B8TJxE1aiyiv2L429BCWfyzAme5sZW8rDb14NeCQHhZbtNqfXhcp2tAnaAT",
        {"encoding": "base64",
         "skipPreflight": true,
        "forwardingPolicy": ["block1vzrYbzLMRdu58ou5XTby4qAqVRLmqo36NKPTg"]}
    ]
  }
'
```

Using the recently released web3.js 2.0 version should allow this custom addition easily.

Suggested rules for flattening (**TODO**);

- Explicitly specified 'allow' implicitly suggests that all others are blocked by default (default closed if allow is provided). This would mean 'allow' should mostly be used with a larger list.
- Explicitly specified block overrides explicitly specified allow? This would allow you to specify a large 'allow' list but narrow it down by selectively blocking specific accounts.

### Addition of a header

A `Solana-ForwardingPolicy` header containing a comma-separated list of policies in the same way the parameter would, but would more easily support older web3.js without having to resort to building custom RPC requests.
