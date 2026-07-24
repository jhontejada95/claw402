# Architecture

Claw402 separates language-model intent from financial authority.

```text
Telegram / CLI
      |
      v
ZeroClaw agent -----> x402 Bazaar discovery (read only)
      |
      | proposed resource + PaymentRequirements
      v
claw402-policy.wasm
      |
      +-- deny: malformed or outside hard policy
      +-- approval_required: valid but new host/merchant
      +-- allow: exact match to operator policy
      |
      v
claw402-signer ------> partially signed x402 payload
      |
      v
facilitator verify/settle -----> Solana devnet
      |
      v
receipt: provider, amount, transaction signature, output hash
```

## Components

### ZeroClaw

Owns the conversational interface, memory, SOP checkpoints, and tool receipts.
It never receives the operational wallet secret.

### `claw402-policy` plugin

A `wasm32-wasip2` ZeroClaw tool plugin. Its pure Rust core validates x402 SVM
offers and ranks Bazaar resources. The host injects only this plugin's jailed
configuration through `__config`.

### `claw402-signer`

A native boundary separate from the WASM plugin. It re-runs policy, seals the
offer to one buyer public key, derives both associated token accounts, builds a
canonical Solana v0 transaction, and signs only the buyer authority. It never
accepts serialized transaction bytes from the model, resource server, or
facilitator.

The output is an x402-compatible partially signed transaction. The approved
facilitator remains the fee payer and its signature slot stays empty until
verification and settlement.

### Trusted RPC adapter

Resolves trusted chain facts required by the builder: recent blockhash, mint
owner program, and decimals. It accepts HTTPS endpoints only and has no signing
capability.

### Facilitator adapter (next)

Submits the partially signed payload to `/verify`, then `/settle`, confirms the
transaction, and persists the receipt.

### Solana allowance (planned)

The operator authorizes a capped, expiring allowance on-chain. This is the
aggregate budget boundary. Local per-request limits are an additional layer,
not the sole accounting mechanism.

## Why the stages are separate

Discovery results, skill documents, API responses, and model messages are all
untrusted. None of them can alter plugin configuration. Signing consumes a
private-field `ApprovedPurchase`, not arbitrary model-generated transaction
bytes. Blockhash and mint metadata are supplied through a distinct trusted
chain context.
