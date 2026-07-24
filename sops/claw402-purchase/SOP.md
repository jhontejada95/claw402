# Claw402 purchase

Transaction rebuilding and restricted buyer signing now pass the local
adversarial suite. The execution tail remains disabled until the trusted RPC
adapter, allowance verification, facilitator calls, and devnet confirmation are
implemented.

## Steps

1. **Discover and evaluate** — Search x402 Bazaar and evaluate the exact offer against hard policy.
   - tools: claw402_policy
   - allow-tools: claw402_policy
   - output: {"type":"object"}
   - next: 2

2. **Human purchase review** — Review provider, merchant, facilitator, amount, and remaining allowance.
   - kind: checkpoint
   - requires_confirmation: true
