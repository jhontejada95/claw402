# Claw402

**Safe autonomous procurement for the agent economy.**

Claw402 lets a self-hosted ZeroClaw agent discover and purchase x402 services
on Solana without giving the language model unrestricted control of a wallet.
Payment requirements are checked by deterministic Rust code before any signing
path is allowed to run.

## Current milestone

The current milestone implements the fail-closed policy core, ZeroClaw WASM
tool boundary, and a separate native restricted signer. It can:

- parse x402 v2 SVM `exact` payment requirements;
- enforce network, mint, amount, timeout, merchant, fee-payer, and host rules;
- rank Bazaar resources only after their payment offers pass policy;
- query the public x402 Bazaar over the jailed ZeroClaw HTTP capability;
- return `allow`, `approval_required`, or `deny` with machine-readable reasons;
- compile the same pure core for native tests and `wasm32-wasip2`.
- seal an allowed offer to one buyer wallet;
- derive the exact source and merchant associated token accounts;
- rebuild a canonical x402 Solana v0 `TransferChecked` transaction;
- partially sign as the buyer while leaving the facilitator fee-payer signature
  empty for verification and settlement.
- resolve the latest blockhash, mint owner program, and decimals through a
  read-only HTTPS Solana RPC adapter.

An end-to-end host smoke test on 2026-07-22 loaded the component in ZeroClaw,
screened 15 live Bazaar results, and selected one policy-compliant Solana offer
for Exa Search at 7,000 atomic USDC (0.007 USDC). See
[Smoke test](docs/SMOKE_TEST.md).

The restricted signer is implemented and adversarially tested, but it is not
yet connected to a funded wallet, RPC adapter, facilitator, or settlement
endpoint. On-chain allowance reads and live devnet settlement remain disabled.

## Security invariant

The model proposes a purchase. The plugin decides whether that purchase is
permitted. Missing policy denies by default. A prompt can never raise a cap,
add a mint, trust a merchant, or trust a facilitator fee payer.

See [Architecture](docs/ARCHITECTURE.md),
[Threat model](docs/THREAT_MODEL.md), and the native signing contract in
[Restricted signer](docs/RESTRICTED_SIGNER.md).

## Test

```bash
cargo test --manifest-path plugin/claw402-policy/Cargo.toml
cargo test --manifest-path signer/claw402-signer/Cargo.toml
cargo run --manifest-path signer/claw402-signer/Cargo.toml --example resolve_devnet
```

## Build the ZeroClaw component

```bash
rustup target add wasm32-wasip2
cd plugin/claw402-policy
cargo build --release --target wasm32-wasip2
```

The resulting component is
`plugin/claw402-policy/target/wasm32-wasip2/release/claw402_policy.wasm`.

## Status

Claw402 is an active bounty prototype. Do not fund its operational wallet on
mainnet until the signing milestone and adversarial test suite are complete.
