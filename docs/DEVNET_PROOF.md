# Solana devnet settlement proof

Claw402 completed an end-to-end x402 resource acquisition on 2026-07-31.

## Result

- Resource: `https://claw402-agent-firewall.opal-ray-6711.chatgpt.site/api/demo-rpc`
- Network: `solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1`
- Asset: devnet USDC (`4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`)
- Amount: 1,000 atomic units (`0.001 USDC`)
- Payer: `BSbwRgvKuUb3QBKrRug6wgrFAxvDRqKinX6pzVt3DmdN`
- Merchant: `CepLpqTzeN4EWnVE9jFFv79DMG4cGHn6hxLh5cffWvJ`
- HTTP result: `200 application/json`
- Resource SHA-256: `ba2ec914d8309088c27e2b20037897b8f723f0bca5daf3b7b473d2cd82976bc3`
- Transaction: [2UxvhPM4...J9Nd](https://explorer.solana.com/tx/2UxvhPM4n1HXWSEQ4JwUGegFegs8zfTbXi12CShYeo3VtWm5aYKyrkPdiDdL85vVJ2vTzspyY8BFDjQDnp9sJ9Nd?cluster=devnet)
- Finalized slot: `480251365`

The finalized balances independently confirmed the transfer: the payer moved
from `2.000000` to `1.999000 USDC`, while the merchant moved from `1.000000`
to `1.001000 USDC`.

## Acquired response

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "status": "ok",
    "service": "Claw402 devnet demo provider",
    "policy": "payment verified and settled before execution"
  }
}
```

The local receipt binds this response to the policy fingerprint, canonical
message digest, payer, merchant, amount, network, and settlement signature.
Private key bytes were never printed or written to any receipt.
