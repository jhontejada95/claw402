use std::collections::HashMap;

use claw402_policy::bazaar::{
    rank_resources, BazaarAcceptance, BazaarQuality, BazaarResource, BazaarSearchResponse,
};
use claw402_policy::policy::{PaymentExtra, PolicyConfig, SOLANA_MAINNET, USDC_MAINNET};

const MERCHANT: &str = "12Ec2cJmfR1C9uwejzxcuMhUgEC7wDrLgm1wBvvR5w9E";
const FEE_PAYER: &str = "Hc3sdEAsCGQcpgfivywog9uwtk8gUBUZgsxdME1EJy88";

fn policy() -> PolicyConfig {
    let mut section = HashMap::new();
    section.insert("max_per_request_atomic".into(), "10000".into());
    section.insert("allowed_merchants".into(), MERCHANT.into());
    section.insert("allowed_fee_payers".into(), FEE_PAYER.into());
    section.insert("allowed_hosts".into(), "one.example,two.example".into());
    PolicyConfig::from_section(&section)
}

fn resource(host: &str, amount: &str, curated: bool, payers: u64) -> BazaarResource {
    BazaarResource {
        resource: format!("https://{host}/query"),
        service_name: Some(host.into()),
        description: None,
        curated,
        quality: BazaarQuality {
            l30_days_total_calls: payers * 10,
            l30_days_unique_payers: payers,
        },
        accepts: vec![BazaarAcceptance {
            scheme: "exact".into(),
            network: SOLANA_MAINNET.into(),
            amount: amount.into(),
            asset: USDC_MAINNET.into(),
            pay_to: MERCHANT.into(),
            max_timeout_seconds: 60,
            extra: PaymentExtra {
                fee_payer: Some(FEE_PAYER.into()),
                memo: None,
            },
        }],
    }
}

#[test]
fn ranking_prefers_curated_then_usage_then_price() {
    let resources = vec![
        resource("one.example", "1000", false, 500),
        resource("two.example", "5000", true, 20),
    ];
    let ranked = rank_resources(&resources, &policy());
    assert_eq!(ranked.len(), 2);
    assert_eq!(ranked[0].resource, "https://two.example/query");
}

#[test]
fn out_of_policy_resources_never_enter_ranking() {
    let mut expensive = resource("one.example", "1000000", true, 9999);
    expensive.accepts[0].asset = "So11111111111111111111111111111111111111112".into();
    assert!(rank_resources(&[expensive], &policy()).is_empty());
}

#[test]
fn parses_and_screens_a_captured_live_bazaar_response() {
    let captured: BazaarSearchResponse =
        serde_json::from_str(include_str!("fixtures/bazaar-exa-mainnet.json")).unwrap();
    assert_eq!(captured.resources.len(), 1);

    let mut section = HashMap::new();
    section.insert("max_per_request_atomic".into(), "10000".into());
    section.insert("allowed_merchants".into(), MERCHANT.into());
    section.insert("allowed_fee_payers".into(), FEE_PAYER.into());
    section.insert("allowed_hosts".into(), "api.exa.ai".into());
    let ranked = rank_resources(&captured.resources, &PolicyConfig::from_section(&section));

    assert_eq!(ranked.len(), 1);
    assert_eq!(ranked[0].amount_atomic, 7000);
    assert_eq!(ranked[0].unique_payers_30d, 127);
}
