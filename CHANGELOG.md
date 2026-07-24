# Changelog

## 0.2.0 - 2026-07-24

- Added a native restricted signer separated from the ZeroClaw WASM boundary.
- Added policy sealing to bind an approved offer to one buyer wallet.
- Added canonical Solana v0 construction for x402 SVM `exact` payments.
- Added partial buyer signing with facilitator fee-payer isolation.
- Added an HTTPS-only, read-only Solana RPC adapter for trusted blockhash and
  mint metadata resolution.
- Added adversarial tests for signer substitution, cap bypass, token-program
  substitution, fee-payer reuse, instruction layout, and merchant ATA derivation.
- Added Solana devnet network and USDC constants plus a secret-free devnet
  configuration example.

## 0.1.0 - 2026-07-22

- Created the Claw402 standalone repository.
- Added a fail-closed Rust policy core for x402 v2 SVM offers.
- Added deterministic Bazaar ranking after policy evaluation.
- Added a ZeroClaw `wasm32-wasip2` tool component with jailed configuration.
- Added live HTTPS discovery through the public x402 Bazaar search endpoint.
- Added nine native tests, including prompt-injection-shaped cap and mint attacks.
- Completed a live component-to-host smoke test against 15 Bazaar results.
- Confirmed fail-closed behavior when a live facilitator fee payer rotated.
- Documented architecture, custody target, threat model, skill rules, and SOP skeleton.
- Pinned Rust 1.96.1 to match the current ZeroClaw master host requirement.
