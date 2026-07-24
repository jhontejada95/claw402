use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use url::Url;

pub const SOLANA_MAINNET: &str = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";
pub const SOLANA_DEVNET: &str = "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1";
pub const USDC_MAINNET: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const USDC_DEVNET: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU";
pub const MAX_X402_MEMO_BYTES: usize = 256;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentOffer {
    pub x402_version: u32,
    pub resource_url: String,
    pub scheme: String,
    pub network: String,
    pub amount: String,
    pub asset: String,
    pub pay_to: String,
    pub max_timeout_seconds: u64,
    #[serde(default)]
    pub extra: PaymentExtra,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentExtra {
    pub fee_payer: Option<String>,
    pub memo: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PolicyConfig {
    pub allowed_networks: HashSet<String>,
    pub allowed_assets: HashSet<String>,
    pub allowed_merchants: HashSet<String>,
    pub allowed_fee_payers: HashSet<String>,
    pub allowed_hosts: HashSet<String>,
    pub max_per_request_atomic: u64,
    pub max_timeout_seconds: u64,
}

impl PolicyConfig {
    pub fn from_section(section: &HashMap<String, String>) -> Self {
        Self {
            allowed_networks: csv_set(section.get("allowed_networks"), &[SOLANA_MAINNET]),
            allowed_assets: csv_set(section.get("allowed_assets"), &[USDC_MAINNET]),
            allowed_merchants: csv_set(section.get("allowed_merchants"), &[]),
            allowed_fee_payers: csv_set(section.get("allowed_fee_payers"), &[]),
            allowed_hosts: csv_set_lower(section.get("allowed_hosts")),
            max_per_request_atomic: parse_u64(section.get("max_per_request_atomic")).unwrap_or(0),
            max_timeout_seconds: parse_u64(section.get("max_timeout_seconds")).unwrap_or(60),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Allow,
    ApprovalRequired,
    Deny,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyEvaluation {
    pub decision: Decision,
    pub amount_atomic: Option<u64>,
    pub reasons: Vec<String>,
}

pub fn evaluate_offer(offer: &PaymentOffer, policy: &PolicyConfig) -> PolicyEvaluation {
    let mut denied = Vec::new();
    let mut approval = Vec::new();

    if offer.x402_version != 2 {
        denied.push("only x402 version 2 is supported".to_string());
    }
    if offer.scheme != "exact" {
        denied.push("only the exact payment scheme is supported".to_string());
    }
    if !policy.allowed_networks.contains(&offer.network) {
        denied.push("network is not allowlisted".to_string());
    }
    if !policy.allowed_assets.contains(&offer.asset) {
        denied.push("asset mint is not allowlisted".to_string());
    }

    let amount = offer.amount.parse::<u64>().ok();
    match amount {
        None => denied.push("amount is not a valid unsigned integer".to_string()),
        Some(0) => denied.push("zero-value payments are not supported".to_string()),
        Some(_) if policy.max_per_request_atomic == 0 => denied
            .push("spending is disabled until max_per_request_atomic is configured".to_string()),
        Some(value) if value > policy.max_per_request_atomic => {
            denied.push("amount exceeds the per-request cap".to_string())
        }
        Some(_) => {}
    }

    if offer.max_timeout_seconds == 0 || offer.max_timeout_seconds > policy.max_timeout_seconds {
        denied.push("payment timeout exceeds policy".to_string());
    }
    if !is_pubkey(&offer.pay_to) {
        denied.push("merchant address is not a valid 32-byte Solana public key".to_string());
    } else if !policy.allowed_merchants.contains(&offer.pay_to) {
        approval.push("merchant is valid but not trusted yet".to_string());
    }

    match offer.extra.fee_payer.as_deref() {
        None | Some("") => denied.push("fee payer is required for SVM x402".to_string()),
        Some(value) if !is_pubkey(value) => {
            denied.push("fee payer is not a valid 32-byte Solana public key".to_string())
        }
        Some(value) if !policy.allowed_fee_payers.contains(value) => {
            approval.push("facilitator fee payer is valid but not trusted yet".to_string())
        }
        Some(_) => {}
    }

    if let Some(memo) = &offer.extra.memo {
        if memo.len() > MAX_X402_MEMO_BYTES {
            denied.push("memo exceeds the x402 SVM limit".to_string());
        }
    }

    match Url::parse(&offer.resource_url) {
        Err(_) => denied.push("resource URL is invalid".to_string()),
        Ok(url) if url.scheme() != "https" => {
            denied.push("resource URL must use HTTPS".to_string())
        }
        Ok(url) => match url.host_str() {
            None => denied.push("resource URL has no host".to_string()),
            Some(host) if !policy.allowed_hosts.contains(&host.to_ascii_lowercase()) => {
                approval.push("resource host is valid but not trusted yet".to_string())
            }
            Some(_) => {}
        },
    }

    if !denied.is_empty() {
        return PolicyEvaluation {
            decision: Decision::Deny,
            amount_atomic: amount,
            reasons: denied,
        };
    }
    if !approval.is_empty() {
        return PolicyEvaluation {
            decision: Decision::ApprovalRequired,
            amount_atomic: amount,
            reasons: approval,
        };
    }
    PolicyEvaluation {
        decision: Decision::Allow,
        amount_atomic: amount,
        reasons: vec!["offer matches every configured hard policy".to_string()],
    }
}

pub fn is_pubkey(value: &str) -> bool {
    bs58::decode(value)
        .into_vec()
        .map(|bytes| bytes.len() == 32)
        .unwrap_or(false)
}

fn csv_set(value: Option<&String>, defaults: &[&str]) -> HashSet<String> {
    match value {
        Some(value) => value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect(),
        None => defaults.iter().map(|item| (*item).to_string()).collect(),
    }
}

fn csv_set_lower(value: Option<&String>) -> HashSet<String> {
    value
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_ascii_lowercase)
                .collect()
        })
        .unwrap_or_default()
}

fn parse_u64(value: Option<&String>) -> Option<u64> {
    value.and_then(|value| value.parse().ok())
}
