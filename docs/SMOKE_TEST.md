# ZeroClaw host smoke test

Date: 2026-07-22

## Scope

This test exercised the actual component boundary, not only the native Rust
library:

1. build ZeroClaw master with `plugins-wasm,plugins-wasm-cranelift`;
2. build `claw402-policy` for `wasm32-wasip2`;
3. install the component into an isolated ZeroClaw config directory;
4. expose only `claw402_policy` to a supervised test agent;
5. call `discover` against the public CDP x402 Bazaar.

No wallet, private key, payment signature, or settlement request was involved.

## Result

The query `Exa search` returned 15 resources. After applying the configured
Solana mainnet, USDC mint, 0.01 USDC cap, merchant, fee-payer, and hostname
allowlists, one offer remained eligible:

```text
eligible_count: 1
resources_screened: 15
resource: https://api.exa.ai/search
amount_atomic: 7000
network: Solana mainnet
```

## Fail-closed drift check

The historical test fixture had Exa's facilitator fee payer as
`Hc3sdEAsCGQcpgfivywog9uwtk8gUBUZgsxdME1EJy88`. The live Bazaar response used
`GVJJ7rdGiXr5xaYbRwRbjfaJL7fmwRygFi1H6aGqDveb` for the same resource.

Before the operator allowlist was updated, Claw402 returned zero eligible
offers. After explicitly approving the current fee payer, the same live flow
returned exactly one. This demonstrates that facilitator metadata drift cannot
silently expand spending authority.
