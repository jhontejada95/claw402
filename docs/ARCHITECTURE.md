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
future policy signer -----> Solana x402 settlement
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

### Policy signer (planned)

Builds the exact SVM payment transaction and signs only after re-running the
same policy checks. It will verify the serialized transaction byte-for-byte
against the approved intent before releasing a signature.

### Solana allowance (planned)

The operator authorizes a capped, expiring allowance on-chain. This is the
aggregate budget boundary. Local per-request limits are an additional layer,
not the sole accounting mechanism.

## Why the stages are separate

Discovery results, skill documents, API responses, and model messages are all
untrusted. None of them can alter plugin configuration. Signing will consume a
typed `ApprovedPurchase`, not arbitrary model-generated transaction bytes.

