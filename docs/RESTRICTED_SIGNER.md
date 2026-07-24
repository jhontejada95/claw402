# Restricted signer

## Purpose

`signer/claw402-signer` is the native custody boundary for Claw402. The ZeroClaw
WASM plugin remains read-only and model-facing; this crate is the only component
designed to receive a buyer signer from an isolated host.

## Contract

The boundary has two explicit stages:

```text
PaymentOffer + PolicyConfig + payer pubkey
                    |
                    v
             approve_purchase
                    |
                    v
       immutable ApprovedPurchase
                    |
           trusted chain context
                    |
                    v
             build_and_sign
                    |
                    v
 partially signed x402 SVM payload
```

`approve_purchase` re-runs the hard policy and refuses `deny` or
`approval_required`. The resulting type has private fields, so callers cannot
change the merchant, amount, asset, network, or fee payer before signing.

`build_and_sign` requires:

- an exact matching buyer signer;
- a recent blockhash from the trusted RPC adapter;
- the mint owner program (`spl-token` or `token-2022`);
- mint decimals from the trusted RPC adapter.

It derives both associated token accounts and creates exactly:

1. `SetComputeUnitLimit(20_000)`;
2. `SetComputeUnitPrice(1 microlamport)`;
3. `TransferChecked`;
4. the seller memo, or a deterministic policy fingerprint.

The transaction is a Solana v0 message. The buyer signs the token authority
slot; the allowlisted facilitator fee-payer slot remains empty, matching the
x402 SVM exact flow.

## Secret handling

The crate does not load environment variables, keypair files, seed phrases, or
remote secrets. An embedding process must provide a `Signer` implementation
from an isolated custody mechanism. No signing key belongs in ZeroClaw config,
plugin config, model context, logs, or HTTP payloads.

## Validation

```bash
cargo test --manifest-path signer/claw402-signer/Cargo.toml
cargo clippy --manifest-path signer/claw402-signer/Cargo.toml --all-targets -- -D warnings
cargo run --manifest-path signer/claw402-signer/Cargo.toml --example resolve_devnet
```

## Next integration

The remaining devnet adapter must resolve the trusted chain context, call the
x402 facilitator `verify` and `settle` endpoints, confirm the transaction, and
write an auditable receipt. It must not introduce an arbitrary
`sign_transaction(bytes)` interface.
