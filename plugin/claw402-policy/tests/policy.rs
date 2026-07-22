use std::collections::HashMap;

use claw402_policy::policy::{evaluate_offer, Decision, PaymentExtra, PaymentOffer};
use claw402_policy::policy::{PolicyConfig, SOLANA_MAINNET, USDC_MAINNET};

const MERCHANT: &str = "12Ec2cJmfR1C9uwejzxcuMhUgEC7wDrLgm1wBvvR5w9E";
const FEE_PAYER: &str = "Hc3sdEAsCGQcpgfivywog9uwtk8gUBUZgsxdME1EJy88";

fn configured_policy() -> PolicyConfig {
    let mut section = HashMap::new();
    section.insert("max_per_request_atomic".into(), "5000".into());
    section.insert("allowed_merchants".into(), MERCHANT.into());
    section.insert("allowed_fee_payers".into(), FEE_PAYER.into());
    section.insert("allowed_hosts".into(), "api.example.com".into());
    PolicyConfig::from_section(&section)
}

fn valid_offer() -> PaymentOffer {
    PaymentOffer {
        x402_version: 2,
        resource_url: "https://api.example.com/risk".into(),
        scheme: "exact".into(),
        network: SOLANA_MAINNET.into(),
        amount: "1000".into(),
        asset: USDC_MAINNET.into(),
        pay_to: MERCHANT.into(),
        max_timeout_seconds: 60,
        extra: PaymentExtra {
            fee_payer: Some(FEE_PAYER.into()),
            memo: Some("risk-42".into()),
        },
    }
}

#[test]
fn exact_allowlisted_offer_is_allowed() {
    let result = evaluate_offer(&valid_offer(), &configured_policy());
    assert_eq!(result.decision, Decision::Allow);
    assert_eq!(result.amount_atomic, Some(1000));
}

#[test]
fn empty_config_disables_spending() {
    let result = evaluate_offer(&valid_offer(), &PolicyConfig::from_section(&HashMap::new()));
    assert_eq!(result.decision, Decision::Deny);
    assert!(result
        .reasons
        .iter()
        .any(|reason| reason.contains("spending is disabled")));
}

#[test]
fn prompt_cannot_raise_cap_or_change_mint() {
    let mut attack = valid_offer();
    attack.amount = "50000000".into();
    attack.asset = "So11111111111111111111111111111111111111112".into();
    let result = evaluate_offer(&attack, &configured_policy());
    assert_eq!(result.decision, Decision::Deny);
    assert!(result
        .reasons
        .iter()
        .any(|reason| reason.contains("asset mint")));
    assert!(result
        .reasons
        .iter()
        .any(|reason| reason.contains("per-request cap")));
}

#[test]
fn new_but_valid_merchant_requires_approval() {
    let mut offer = valid_offer();
    offer.pay_to = "2wKupLR9q6wXYppw8Gr2NvWxKBUqm4PPJKkQfoxHDBg4".into();
    let result = evaluate_offer(&offer, &configured_policy());
    assert_eq!(result.decision, Decision::ApprovalRequired);
}

#[test]
fn insecure_resource_url_is_denied() {
    let mut offer = valid_offer();
    offer.resource_url = "http://api.example.com/risk".into();
    let result = evaluate_offer(&offer, &configured_policy());
    assert_eq!(result.decision, Decision::Deny);
}

#[test]
fn malformed_addresses_are_denied() {
    let mut offer = valid_offer();
    offer.pay_to = "not-a-pubkey".into();
    offer.extra.fee_payer = Some("also-not-a-pubkey".into());
    let result = evaluate_offer(&offer, &configured_policy());
    assert_eq!(result.decision, Decision::Deny);
    assert_eq!(
        result
            .reasons
            .iter()
            .filter(|reason| reason.contains("32-byte"))
            .count(),
        2
    );
}
