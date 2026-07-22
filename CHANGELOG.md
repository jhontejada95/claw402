# Changelog

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
