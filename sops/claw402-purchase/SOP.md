# Claw402 purchase

Operator procedure for acquiring one allowlisted x402 resource from a real
ZeroClaw channel. Claw402 is a **T2 Sign** use case on Solana devnet: the model
can propose an offer, but only deterministic policy, an explicit human
checkpoint, and the restricted native signer can move the session wallet's
limited funds. Autonomous mainnet settlement is disabled.

## Steps

1. **Discover and evaluate** — Search x402 Bazaar or inspect the exact resource
   challenge, then evaluate the immutable payment fields against operator config.
   - tools: claw402_policy
   - allow-tools: claw402_policy
   - output: {"type":"object"}
   - next: 2

2. **Human purchase review** — In the originating channel, show the service,
   HTTPS host, network, mint, atomic amount, merchant, facilitator fee payer,
   and remaining daily budget. A denial is final; approval never changes policy.
   - kind: checkpoint
   - requires_confirmation: true

3. **Restricted settlement** — After approval, the operator invokes
   `settle_devnet` with `--confirm SETTLE_DEVNET`. The native signer re-runs
   policy, reserves the SQLite daily budget, rebuilds the canonical transaction,
   signs only as the buyer, and submits it through the approved x402 resource.
   The raw session key is never sent to the model or channel.

4. **Reconcile and report** — Report success only when the resource returned
   HTTP 200 and `PAYMENT-RESPONSE` matched payer, network, amount, and finalized
   Solana transaction. Return the transaction link and remaining daily budget to
   the same channel. On an ambiguous post-signing failure, stop for reconciliation;
   never retry by silently expanding authority.

## Demo command

```bash
cargo run --manifest-path signer/claw402-signer/Cargo.toml \
  --example settle_devnet -- \
  --offer .tmp/claw402-devnet/offer.json \
  --policy config.local.toml \
  --wallet .tmp/claw402-devnet/payer.json \
  --budget .tmp/claw402-devnet/budget.sqlite \
  --request .tmp/claw402-devnet/request.json \
  --output .tmp/claw402-devnet/resource.json \
  --receipt .tmp/claw402-devnet/receipt.json \
  --confirm SETTLE_DEVNET
```

All `.tmp` artifacts and local policy files are excluded from version control.
