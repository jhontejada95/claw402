//! HTTPS x402 resource acquisition after a restricted SVM signature.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;
use url::Url;

use crate::{
    facilitator::{PaymentPayloadV2, SettleResponse},
    ApprovedPurchase, RestrictedSignature,
};

pub const MAX_RESOURCE_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("resource URL must use HTTPS")]
    InsecureUrl,
    #[error("invalid resource URL")]
    InvalidUrl,
    #[error("signed payload does not match the policy-approved purchase")]
    ApprovalMismatch,
    #[error("failed to encode payment payload: {0}")]
    Encoding(String),
    #[error("resource request failed: {0}")]
    Transport(String),
    #[error("resource rejected payment with HTTP 402: {0}")]
    PaymentRequired(String),
    #[error("successful resource response omitted PAYMENT-RESPONSE")]
    MissingPaymentResponse,
    #[error("PAYMENT-RESPONSE is not valid base64 JSON: {0}")]
    InvalidPaymentResponse(String),
    #[error("resource settlement does not match the approved purchase: {0}")]
    SettlementMismatch(String),
    #[error("resource body exceeded the one MiB safety limit")]
    BodyTooLarge,
    #[error("system clock is before the Unix epoch")]
    InvalidSystemClock,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceReceipt {
    pub created_at_unix_seconds: u64,
    pub resource_url: String,
    pub http_status: u16,
    pub content_type: Option<String>,
    pub resource_sha256: String,
    pub resource_bytes: usize,
    pub payer: String,
    pub pay_to: String,
    pub amount: String,
    pub asset: String,
    pub network: String,
    pub policy_fingerprint: String,
    pub message_sha256: String,
    pub transaction: String,
    pub settlement: SettleResponse,
}

#[derive(Debug)]
pub struct ResourceOutput {
    pub body: Vec<u8>,
    pub receipt: ResourceReceipt,
}

#[derive(Debug, Clone)]
pub struct ResourceClient {
    agent: ureq::Agent,
}

impl ResourceClient {
    pub fn new() -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(90)))
            .https_only(true)
            .http_status_as_error(false)
            .build();
        Self {
            agent: ureq::Agent::new_with_config(config),
        }
    }

    pub fn execute_json(
        &self,
        approved: &ApprovedPurchase,
        signature: &RestrictedSignature,
        payment: &PaymentPayloadV2,
        request_body: &serde_json::Value,
    ) -> Result<ResourceOutput, ResourceError> {
        ensure_binding(approved, signature, payment)?;
        let resource_url = &approved.offer().resource_url;
        let parsed = Url::parse(resource_url).map_err(|_| ResourceError::InvalidUrl)?;
        if parsed.scheme() != "https" {
            return Err(ResourceError::InsecureUrl);
        }

        let payment_header = encode_payment(payment)?;
        let mut response = self
            .agent
            .post(resource_url)
            .header("PAYMENT-SIGNATURE", payment_header)
            .send_json(request_body)
            .map_err(|error| ResourceError::Transport(error.to_string()))?;

        let status = response.status().as_u16();
        if status == 402 {
            let body = response
                .body_mut()
                .with_config()
                .limit(64 * 1024)
                .read_to_string()
                .unwrap_or_else(|error| format!("unreadable rejection body: {error}"));
            return Err(ResourceError::PaymentRequired(body));
        }
        if status >= 400 {
            return Err(ResourceError::Transport(format!(
                "resource returned HTTP {status}"
            )));
        }
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string);
        let payment_response = response
            .headers()
            .get("PAYMENT-RESPONSE")
            .and_then(|value| value.to_str().ok())
            .ok_or(ResourceError::MissingPaymentResponse)?;
        let settlement = decode_settlement(payment_response)?;
        validate_settlement(approved, signature, &settlement)?;

        let body = response
            .body_mut()
            .with_config()
            .limit(MAX_RESOURCE_BYTES)
            .read_to_vec()
            .map_err(|error| {
                if error.to_string().contains("larger than request limit") {
                    ResourceError::BodyTooLarge
                } else {
                    ResourceError::Transport(error.to_string())
                }
            })?;
        let created_at_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ResourceError::InvalidSystemClock)?
            .as_secs();

        Ok(ResourceOutput {
            receipt: ResourceReceipt {
                created_at_unix_seconds,
                resource_url: resource_url.clone(),
                http_status: status,
                content_type,
                resource_sha256: hex_digest(&body),
                resource_bytes: body.len(),
                payer: signature.payer.clone(),
                pay_to: approved.offer().pay_to.clone(),
                amount: approved.offer().amount.clone(),
                asset: approved.offer().asset.clone(),
                network: approved.offer().network.clone(),
                policy_fingerprint: approved.fingerprint().to_string(),
                message_sha256: signature.message_sha256.clone(),
                transaction: settlement.transaction.clone(),
                settlement,
            },
            body,
        })
    }
}

impl Default for ResourceClient {
    fn default() -> Self {
        Self::new()
    }
}

fn ensure_binding(
    approved: &ApprovedPurchase,
    signature: &RestrictedSignature,
    payment: &PaymentPayloadV2,
) -> Result<(), ResourceError> {
    if signature.policy_fingerprint != approved.fingerprint()
        || payment.resource.url != approved.offer().resource_url
        || payment.accepted.pay_to != approved.offer().pay_to
        || payment.accepted.amount != approved.offer().amount
        || payment.accepted.asset != approved.offer().asset
        || payment.accepted.network != approved.offer().network
        || payment.payload.transaction != signature.payload.transaction
    {
        return Err(ResourceError::ApprovalMismatch);
    }
    Ok(())
}

fn encode_payment(payment: &PaymentPayloadV2) -> Result<String, ResourceError> {
    serde_json::to_vec(payment)
        .map(|json| BASE64.encode(json))
        .map_err(|error| ResourceError::Encoding(error.to_string()))
}

fn decode_settlement(encoded: &str) -> Result<SettleResponse, ResourceError> {
    let decoded = BASE64
        .decode(encoded)
        .map_err(|error| ResourceError::InvalidPaymentResponse(error.to_string()))?;
    serde_json::from_slice(&decoded)
        .map_err(|error| ResourceError::InvalidPaymentResponse(error.to_string()))
}

fn validate_settlement(
    approved: &ApprovedPurchase,
    signature: &RestrictedSignature,
    settlement: &SettleResponse,
) -> Result<(), ResourceError> {
    if !settlement.success {
        return Err(ResourceError::SettlementMismatch(
            "resource reported an unsuccessful settlement".into(),
        ));
    }
    if settlement.transaction.is_empty() {
        return Err(ResourceError::SettlementMismatch(
            "transaction signature is empty".into(),
        ));
    }
    if settlement.network != approved.offer().network {
        return Err(ResourceError::SettlementMismatch(
            "network differs from approval".into(),
        ));
    }
    if settlement.payer.as_deref() != Some(signature.payer.as_str()) {
        return Err(ResourceError::SettlementMismatch(
            "payer differs from restricted signer".into(),
        ));
    }
    if settlement
        .amount
        .as_deref()
        .is_some_and(|amount| amount != approved.offer().amount)
    {
        return Err(ResourceError::SettlementMismatch(
            "amount differs from approval".into(),
        ));
    }
    Ok(())
}

fn hex_digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_a_v2_payment_response() {
        let encoded = BASE64.encode(
            br#"{"success":true,"transaction":"tx","network":"solana:devnet","payer":"payer","amount":"1000","errorReason":null,"errorMessage":null}"#,
        );
        let settlement = decode_settlement(&encoded).unwrap();
        assert!(settlement.success);
        assert_eq!(settlement.transaction, "tx");
    }

    #[test]
    fn rejects_non_base64_payment_responses() {
        assert!(matches!(
            decode_settlement("not base64"),
            Err(ResourceError::InvalidPaymentResponse(_))
        ));
    }
}
