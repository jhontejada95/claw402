# Claw402 purchase

The execution tail remains intentionally disabled until transaction rebuilding,
allowance verification, and signing pass the adversarial test suite.

## Steps

1. **Discover and evaluate** — Search x402 Bazaar and evaluate the exact offer against hard policy.
   - tools: claw402_policy
   - allow-tools: claw402_policy
   - output: {"type":"object"}
   - next: 2

2. **Human purchase review** — Review provider, merchant, facilitator, amount, and remaining allowance.
   - kind: checkpoint
   - requires_confirmation: true

