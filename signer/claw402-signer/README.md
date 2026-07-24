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

After signing, the HTTPS-only facilitator adapter:

1. binds the x402 payload back to the sealed policy fingerprint;
2. calls `/verify`;
3. checks that the verified payer is the expected buyer;
4. calls `/settle` only after valid verification;
5. refuses duplicate or concurrent settlement of one signed message;
6. writes a receipt containing the policy fingerprint, message digest, amount,
   network, payer, merchant, and transaction signature.

Run the deterministic test suite:

```bash
cargo test --manifest-path signer/claw402-signer/Cargo.toml
```

Resolve a live, read-only devnet context without loading a wallet:

```bash
cargo run --manifest-path signer/claw402-signer/Cargo.toml \
  --example resolve_devnet
```

Create the disposable local wallet used only for the devnet demo:

```bash
cargo run --manifest-path signer/claw402-signer/Cargo.toml \
  --example create_devnet_wallet
```

The command refuses to overwrite an existing wallet and never prints secret
bytes. Its output path is under `.tmp/`, which is excluded from Git.

Execute the reviewed devnet offer only after funding the disposable wallet and
replacing the placeholders in `config/claw402.devnet.toml`:

```bash
cargo run --manifest-path signer/claw402-signer/Cargo.toml \
  --example settle_devnet -- \
  --offer .tmp/claw402-devnet/offer.json \
  --policy config/claw402.devnet.toml \
  --confirm SETTLE_DEVNET
```

The runner refuses non-devnet offers, non-HTTPS RPC/facilitator endpoints,
policy mismatches, invalid verification, payer mismatches, and settlement
without the explicit confirmation phrase.
