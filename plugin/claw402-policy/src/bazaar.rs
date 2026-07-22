use serde::{Deserialize, Serialize};

use crate::policy::{evaluate_offer, Decision, PaymentExtra, PaymentOffer, PolicyConfig};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BazaarSearchResponse {
    #[serde(default)]
    pub resources: Vec<BazaarResource>,
    #[serde(default)]
    pub partial_results: bool,
    #[serde(default)]
    pub search_method: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BazaarResource {
    pub resource: String,
    #[serde(default)]
    pub service_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub curated: bool,
    #[serde(default)]
    pub quality: BazaarQuality,
    #[serde(default)]
    pub accepts: Vec<BazaarAcceptance>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BazaarQuality {
    #[serde(default)]
    pub l30_days_total_calls: u64,
    #[serde(default)]
    pub l30_days_unique_payers: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BazaarAcceptance {
    pub scheme: String,
    pub network: String,
    pub amount: String,
    pub asset: String,
    pub pay_to: String,
    pub max_timeout_seconds: u64,
    #[serde(default)]
    pub extra: PaymentExtra,
}

#[derive(Debug, Clone, Serialize)]
pub struct RankedResource {
    pub resource: String,
    pub service_name: Option<String>,
    pub amount_atomic: u64,
    pub curated: bool,
    pub unique_payers_30d: u64,
    pub calls_30d: u64,
}

pub fn rank_resources(resources: &[BazaarResource], policy: &PolicyConfig) -> Vec<RankedResource> {
    let mut ranked = Vec::new();
    for resource in resources {
        for acceptance in &resource.accepts {
            let offer = PaymentOffer {
                x402_version: 2,
                resource_url: resource.resource.clone(),
                scheme: acceptance.scheme.clone(),
                network: acceptance.network.clone(),
                amount: acceptance.amount.clone(),
                asset: acceptance.asset.clone(),
                pay_to: acceptance.pay_to.clone(),
                max_timeout_seconds: acceptance.max_timeout_seconds,
                extra: acceptance.extra.clone(),
            };
            let evaluation = evaluate_offer(&offer, policy);
            if evaluation.decision == Decision::Allow {
                ranked.push(RankedResource {
                    resource: resource.resource.clone(),
                    service_name: resource.service_name.clone(),
                    amount_atomic: evaluation.amount_atomic.unwrap_or(u64::MAX),
                    curated: resource.curated,
                    unique_payers_30d: resource.quality.l30_days_unique_payers,
                    calls_30d: resource.quality.l30_days_total_calls,
                });
            }
        }
    }

    ranked.sort_by(|left, right| {
        right
            .curated
            .cmp(&left.curated)
            .then_with(|| right.unique_payers_30d.cmp(&left.unique_payers_30d))
            .then_with(|| left.amount_atomic.cmp(&right.amount_atomic))
            .then_with(|| left.resource.cmp(&right.resource))
    });
    ranked
}
