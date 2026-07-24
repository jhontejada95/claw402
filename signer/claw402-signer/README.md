# Claw402 restricted signer

Native signing boundary for x402 v2 payments on Solana. It is intentionally
separate from the model-facing WASM plugin.

The signer:

1. re-runs `claw402-policy`;
2. seals an allowlisted `PaymentOffer` to one payer;
3. derives the source and destination associated token accounts;
4. builds only the canonical x402 instruction sequence:
   `ComputeUnitLimit`, `ComputeUnitPrice`, `TransferChecked`, `Memo`;
5. signs the buyer authority locally and leaves the facilitator fee-payer
   signature empty;
6. returns a base64 serialized `VersionedTransaction` for the x402 payload.

It never accepts arbitrary serialized transactions and exposes no key-loading
API. The host must supply an isolated signer implementation and trusted chain
facts (blockhash, mint owner program, and decimals).

Run the deterministic test suite:

```bash
cargo test --manifest-path signer/claw402-signer/Cargo.toml
```

Resolve a live, read-only devnet context without loading a wallet:

```bash
cargo run --manifest-path signer/claw402-signer/Cargo.toml \
  --example resolve_devnet
```
