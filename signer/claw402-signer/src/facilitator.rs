//! Fail-closed x402 facilitator client and auditable settlement receipts.

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use claw402_policy::policy::{PaymentExtra, PaymentOffer};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::{ApprovedPurchase, ExactSvmPayload, RestrictedSignature};

#[derive(Debug, Error)]
pub enum FacilitatorError {
    #[error("facilitator URL must use HTTPS")]
    InsecureUrl,
    #[error("invalid facilitator URL")]
    InvalidUrl,
    #[error("signed payload does not match the policy-approved purchase")]
    ApprovalMismatch,
    #[error("facilitator request failed: {0}")]
    Transport(String),
    #[error("facilitator rejected verification: {0}")]
    VerificationRejected(String),
    #[error("facilitator verified a different payer")]
    PayerMismatch,
    #[error("settlement failed: {0}")]
    SettlementFailed(String),
    #[error("duplicate or concurrent settlement attempt refused")]
    DuplicateSettlement,
    #[error("settlement replay guard is unavailable")]
    ReplayGuardUnavailable,
    #[error("system clock is before the Unix epoch")]
    InvalidSystemClock,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceInfo {
    pub url: String,
    pub description: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirementsV2 {
    pub scheme: String,
    pub network: String,
    pub amount: String,
    pub asset: String,
    pub pay_to: String,
    pub max_timeout_seconds: u64,
    pub extra: PaymentExtra,
}

impl From<&PaymentOffer> for PaymentRequirementsV2 {
    fn from(offer: &PaymentOffer) -> Self {
        Self {
            scheme: offer.scheme.clone(),
            network: offer.network.clone(),
            amount: offer.amount.clone(),
            asset: offer.asset.clone(),
            pay_to: offer.pay_to.clone(),
            max_timeout_seconds: offer.max_timeout_seconds,
            extra: offer.extra.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPayloadV2 {
    pub x402_version: u32,
    pub resource: ResourceInfo,
    pub accepted: PaymentRequirementsV2,
    pub payload: ExactSvmPayload,
}

impl PaymentPayloadV2 {
    /// Binds a restricted signature back to the sealed purchase that produced it.
    pub fn from_approved(
        approved: &ApprovedPurchase,
        signature: &RestrictedSignature,
        description: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Result<Self, FacilitatorError> {
        let offer = approved.offer();
        let expected_fee_payer = offer.extra.fee_payer.as_deref().unwrap_or_default();
        if signature.policy_fingerprint != approved.fingerprint()
            || signature.payer != approved.payer().to_string()
            || signature.fee_payer != expected_fee_payer
            || signature.x402_version != offer.x402_version
        {
            return Err(FacilitatorError::ApprovalMismatch);
        }

        Ok(Self {
            x402_version: offer.x402_version,
            resource: ResourceInfo {
                url: offer.resource_url.clone(),
                description: description.into(),
                mime_type: mime_type.into(),
            },
            accepted: PaymentRequirementsV2::from(offer),
            payload: signature.payload.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FacilitatorRequest<'a> {
    x402_version: u32,
    payment_payload: &'a PaymentPayloadV2,
    payment_requirements: &'a PaymentRequirementsV2,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResponse {
    pub is_valid: bool,
    pub payer: Option<String>,
    pub invalid_reason: Option<String>,
    pub invalid_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleResponse {
    pub success: bool,
    pub transaction: String,
    pub network: String,
    pub payer: Option<String>,
    pub error_reason: Option<String>,
    pub error_message: Option<String>,
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementReceipt {
    pub created_at_unix_seconds: u64,
    pub resource_url: String,
    pub payer: String,
    pub pay_to: String,
    pub amount: String,
    pub asset: String,
    pub network: String,
    pub policy_fingerprint: String,
    pub message_sha256: String,
    pub transaction: String,
    pub verification: VerifyResponse,
    pub settlement: SettleResponse,
}

#[derive(Debug, Clone)]
pub struct FacilitatorClient {
    endpoint: String,
    settlement_guard: Arc<Mutex<HashSet<String>>>,
}

impl FacilitatorClient {
    pub fn new(endpoint: impl Into<String>) -> Result<Self, FacilitatorError> {
        let endpoint = endpoint.into();
        let parsed = Url::parse(&endpoint).map_err(|_| FacilitatorError::InvalidUrl)?;
        if parsed.scheme() != "https" {
            return Err(FacilitatorError::InsecureUrl);
        }
        Ok(Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            settlement_guard: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    pub fn verify(&self, payment: &PaymentPayloadV2) -> Result<VerifyResponse, FacilitatorError> {
        self.post("verify", payment)
    }

    fn settle(&self, payment: &PaymentPayloadV2) -> Result<SettleResponse, FacilitatorError> {
        self.post("settle", payment)
    }

    /// Verifies first, checks the verified payer, then settles. No settle request
    /// is sent when verification is invalid or ambiguous.
    pub fn verify_and_settle(
        &self,
        approved: &ApprovedPurchase,
        signature: &RestrictedSignature,
        payment: &PaymentPayloadV2,
    ) -> Result<SettlementReceipt, FacilitatorError> {
        if signature.policy_fingerprint != approved.fingerprint()
            || payment.accepted.pay_to != approved.offer().pay_to
            || payment.accepted.amount != approved.offer().amount
            || payment.accepted.asset != approved.offer().asset
            || payment.accepted.network != approved.offer().network
        {
            return Err(FacilitatorError::ApprovalMismatch);
        }

        let verification = self.verify(payment)?;
        if !verification.is_valid {
            return Err(FacilitatorError::VerificationRejected(response_reason(
                verification.invalid_reason.as_deref(),
                verification.invalid_message.as_deref(),
            )));
        }
        if verification.payer.as_deref() != Some(signature.payer.as_str()) {
            return Err(FacilitatorError::PayerMismatch);
        }

        self.reserve_settlement(&signature.message_sha256)?;

        let settlement = match self.settle(payment) {
            Ok(settlement) => settlement,
            Err(error) => {
                self.release_settlement(&signature.message_sha256);
                return Err(error);
            }
        };
        if !settlement.success {
            self.release_settlement(&signature.message_sha256);
            return Err(FacilitatorError::SettlementFailed(response_reason(
                settlement.error_reason.as_deref(),
                settlement.error_message.as_deref(),
            )));
        }
        if settlement.payer.as_deref() != Some(signature.payer.as_str()) {
            self.release_settlement(&signature.message_sha256);
            return Err(FacilitatorError::PayerMismatch);
        }
        if settlement.network != payment.accepted.network
            || settlement.transaction.is_empty()
            || settlement
                .amount
                .as_deref()
                .is_some_and(|amount| amount != payment.accepted.amount)
        {
            self.release_settlement(&signature.message_sha256);
            return Err(FacilitatorError::SettlementFailed(
                "facilitator receipt does not match the approved payment".into(),
            ));
        }

        let created_at_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| FacilitatorError::InvalidSystemClock)?
            .as_secs();
        Ok(SettlementReceipt {
            created_at_unix_seconds,
            resource_url: payment.resource.url.clone(),
            payer: signature.payer.clone(),
            pay_to: payment.accepted.pay_to.clone(),
            amount: payment.accepted.amount.clone(),
            asset: payment.accepted.asset.clone(),
            network: payment.accepted.network.clone(),
            policy_fingerprint: signature.policy_fingerprint.clone(),
            message_sha256: signature.message_sha256.clone(),
            transaction: settlement.transaction.clone(),
            verification,
            settlement,
        })
    }

    fn release_settlement(&self, message_sha256: &str) {
        if let Ok(mut guard) = self.settlement_guard.lock() {
            guard.remove(message_sha256);
        }
    }

    fn reserve_settlement(&self, message_sha256: &str) -> Result<(), FacilitatorError> {
        let mut guard = self
            .settlement_guard
            .lock()
            .map_err(|_| FacilitatorError::ReplayGuardUnavailable)?;
        if !guard.insert(message_sha256.to_string()) {
            return Err(FacilitatorError::DuplicateSettlement);
        }
        Ok(())
    }

    fn post<T: for<'de> Deserialize<'de>>(
        &self,
        operation: &str,
        payment: &PaymentPayloadV2,
    ) -> Result<T, FacilitatorError> {
        let request = FacilitatorRequest {
            x402_version: payment.x402_version,
            payment_payload: payment,
            payment_requirements: &payment.accepted,
        };
        let mut response = ureq::post(format!("{}/{operation}", self.endpoint))
            .send_json(&request)
            .map_err(|error| FacilitatorError::Transport(error.to_string()))?;
        response
            .body_mut()
            .read_json()
            .map_err(|error| FacilitatorError::Transport(error.to_string()))
    }
}

fn response_reason(code: Option<&str>, message: Option<&str>) -> String {
    match (code, message) {
        (Some(code), Some(message)) => format!("{code}: {message}"),
        (Some(code), None) => code.to_string(),
        (None, Some(message)) => message.to_string(),
        (None, None) => "facilitator returned no reason".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use claw402_policy::policy::{
        PaymentExtra, PaymentOffer, PolicyConfig, SOLANA_DEVNET, USDC_DEVNET,
    };
    use solana_sdk::{signature::Keypair, signer::Signer};

    use super::*;
    use crate::{approve_purchase, build_and_sign, BuildContext, TOKEN_PROGRAM};

    const MERCHANT: &str = "12Ec2cJmfR1C9uwejzxcuMhUgEC7wDrLgm1wBvvR5w9E";
    const FEE_PAYER: &str = "Hc3sdEAsCGQcpgfivywog9uwtk8gUBUZgsxdME1EJy88";

    fn fixture() -> (ApprovedPurchase, RestrictedSignature) {
        let offer = PaymentOffer {
            x402_version: 2,
            resource_url: "https://api.example.com/risk".into(),
            scheme: "exact".into(),
            network: SOLANA_DEVNET.into(),
            amount: "7000".into(),
            asset: USDC_DEVNET.into(),
            pay_to: MERCHANT.into(),
            max_timeout_seconds: 60,
            extra: PaymentExtra {
                fee_payer: Some(FEE_PAYER.into()),
                memo: Some("claw402-test".into()),
            },
        };
        let mut section = HashMap::new();
        section.insert("allowed_networks".into(), SOLANA_DEVNET.into());
        section.insert("allowed_assets".into(), USDC_DEVNET.into());
        section.insert("max_per_request_atomic".into(), "10000".into());
        section.insert("allowed_merchants".into(), MERCHANT.into());
        section.insert("allowed_fee_payers".into(), FEE_PAYER.into());
        section.insert("allowed_hosts".into(), "api.example.com".into());
        let payer = Keypair::new();
        let approved = approve_purchase(
            offer,
            &PolicyConfig::from_section(&section),
            &payer.pubkey().to_string(),
        )
        .unwrap();
        let signature = build_and_sign(
            &approved,
            &BuildContext {
                recent_blockhash: "EZ3rST5dvHmbanh75jc4PuLfV96vp9fEYBVeNk4FfM1k".into(),
                last_valid_block_height: 291_470_237,
                token_program: TOKEN_PROGRAM.into(),
                mint_decimals: 6,
            },
            &payer,
        )
        .unwrap();
        (approved, signature)
    }

    #[test]
    fn rejects_non_https_facilitators() {
        assert!(matches!(
            FacilitatorClient::new("http://localhost:4020"),
            Err(FacilitatorError::InsecureUrl)
        ));
    }

    #[test]
    fn produces_the_official_v2_facilitator_envelope() {
        let (approved, signature) = fixture();
        let payment =
            PaymentPayloadV2::from_approved(&approved, &signature, "Risk API", "application/json")
                .unwrap();
        let request = FacilitatorRequest {
            x402_version: 2,
            payment_payload: &payment,
            payment_requirements: &payment.accepted,
        };
        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json["x402Version"], 2);
        assert_eq!(
            json["paymentPayload"]["resource"]["url"],
            approved.offer().resource_url
        );
        assert_eq!(json["paymentRequirements"]["payTo"], MERCHANT);
        assert_eq!(
            json["paymentPayload"]["payload"]["transaction"],
            signature.payload.transaction
        );
    }

    #[test]
    fn refuses_a_signature_detached_from_its_approval() {
        let (approved, mut signature) = fixture();
        signature.policy_fingerprint = "attacker-controlled".into();

        assert!(matches!(
            PaymentPayloadV2::from_approved(&approved, &signature, "Risk API", "application/json"),
            Err(FacilitatorError::ApprovalMismatch)
        ));
    }

    #[test]
    fn refuses_duplicate_settlement_reservations() {
        let client = FacilitatorClient::new("https://x402.org/facilitator").unwrap();
        client.reserve_settlement("message-digest").unwrap();

        assert!(matches!(
            client.reserve_settlement("message-digest"),
            Err(FacilitatorError::DuplicateSettlement)
        ));
    }
}
