//! Native restricted signer for x402 v2 SVM payments.
//!
//! This crate intentionally lives outside the WASM policy plugin. The model-facing
//! plugin can inspect offers, but only this native boundary can access a signer.
//! It never accepts serialized transactions: it rebuilds the exact transfer from
//! an allowlisted [`PaymentOffer`] and returns a partially signed x402 payload.

pub mod rpc;

use std::str::FromStr;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use claw402_policy::policy::{
    evaluate_offer, Decision, PaymentOffer, PolicyConfig, MAX_X402_MEMO_BYTES,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    message::VersionedMessage,
    pubkey::Pubkey,
    signature::Signature,
    signer::{null_signer::NullSigner, Signer},
    transaction::VersionedTransaction,
};
use thiserror::Error;

pub const DEFAULT_COMPUTE_UNIT_LIMIT: u32 = 20_000;
pub const DEFAULT_COMPUTE_UNIT_PRICE_MICROLAMPORTS: u64 = 1;
pub const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
pub const MEMO_PROGRAM: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";

#[derive(Debug, Error)]
pub enum SignerError {
    #[error("policy rejected purchase: {0}")]
    PolicyRejected(String),
    #[error("invalid {field} public key: {value}")]
    InvalidPubkey { field: &'static str, value: String },
    #[error("invalid recent blockhash")]
    InvalidBlockhash,
    #[error("unsupported token program: {0}")]
    UnsupportedTokenProgram(String),
    #[error("memo exceeds the x402 SVM 256-byte limit")]
    MemoTooLong,
    #[error("signer does not match the approved payer")]
    SignerMismatch,
    #[error("facilitator fee payer must be isolated from the buyer authority")]
    FeePayerNotIsolated,
    #[error("failed to partially sign transaction: {0}")]
    Signing(String),
    #[error("signed transaction message differs from the approved canonical message")]
    MessageMismatch,
    #[error("failed to serialize transaction: {0}")]
    Serialization(String),
}

/// Trusted chain facts resolved outside model-controlled input.
#[derive(Debug, Clone)]
pub struct BuildContext {
    pub recent_blockhash: String,
    pub last_valid_block_height: u64,
    pub token_program: String,
    pub mint_decimals: u8,
}

/// A policy-sealed purchase. Its fields are private so callers cannot mutate the
/// approved intent between evaluation and signing.
#[derive(Debug, Clone)]
pub struct ApprovedPurchase {
    offer: PaymentOffer,
    payer: Pubkey,
    fingerprint: String,
}

impl ApprovedPurchase {
    pub fn offer(&self) -> &PaymentOffer {
        &self.offer
    }

    pub fn payer(&self) -> Pubkey {
        self.payer
    }

    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExactSvmPayload {
    pub transaction: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestrictedSignature {
    pub x402_version: u32,
    pub payload: ExactSvmPayload,
    pub payer: String,
    pub fee_payer: String,
    pub policy_fingerprint: String,
    pub message_sha256: String,
    pub last_valid_block_height: u64,
}

/// Re-runs hard policy and seals the resulting purchase to one payer.
pub fn approve_purchase(
    offer: PaymentOffer,
    policy: &PolicyConfig,
    payer: &str,
) -> Result<ApprovedPurchase, SignerError> {
    let evaluation = evaluate_offer(&offer, policy);
    if evaluation.decision != Decision::Allow {
        return Err(SignerError::PolicyRejected(evaluation.reasons.join("; ")));
    }

    let payer = parse_pubkey("payer", payer)?;
    let fingerprint = policy_fingerprint(&offer, &payer)?;

    Ok(ApprovedPurchase {
        offer,
        payer,
        fingerprint,
    })
}

/// Builds the canonical x402 SVM transaction and signs only the buyer authority.
///
/// The facilitator fee-payer signature remains the all-zero default signature,
/// as required for an x402 partially signed payment payload.
pub fn build_and_sign(
    approved: &ApprovedPurchase,
    context: &BuildContext,
    signer: &dyn Signer,
) -> Result<RestrictedSignature, SignerError> {
    if signer.pubkey() != approved.payer {
        return Err(SignerError::SignerMismatch);
    }

    let offer = &approved.offer;
    let mint = parse_pubkey("asset", &offer.asset)?;
    let merchant = parse_pubkey("merchant", &offer.pay_to)?;
    let fee_payer = parse_pubkey(
        "fee payer",
        offer.extra.fee_payer.as_deref().unwrap_or_default(),
    )?;
    if fee_payer == approved.payer {
        return Err(SignerError::FeePayerNotIsolated);
    }
    let blockhash =
        Hash::from_str(&context.recent_blockhash).map_err(|_| SignerError::InvalidBlockhash)?;
    let token_program = parse_token_program(&context.token_program)?;

    let source = spl_associated_token_account::get_associated_token_address_with_program_id(
        &approved.payer,
        &mint,
        &token_program,
    );
    let destination = spl_associated_token_account::get_associated_token_address_with_program_id(
        &merchant,
        &mint,
        &token_program,
    );
    let amount = offer
        .amount
        .parse::<u64>()
        .map_err(|_| SignerError::PolicyRejected("approved amount is invalid".into()))?;

    let transfer = build_transfer_checked(
        &token_program,
        &source,
        &mint,
        &destination,
        &approved.payer,
        amount,
        context.mint_decimals,
    );
    let memo = memo_instruction(offer.extra.memo.as_deref(), &approved.fingerprint)?;
    let instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(DEFAULT_COMPUTE_UNIT_LIMIT),
        ComputeBudgetInstruction::set_compute_unit_price(DEFAULT_COMPUTE_UNIT_PRICE_MICROLAMPORTS),
        transfer,
        memo,
    ];

    let message = VersionedMessage::V0(
        solana_sdk::message::v0::Message::try_compile(&fee_payer, &instructions, &[], blockhash)
            .map_err(|error| SignerError::Signing(error.to_string()))?,
    );
    let canonical_message = message.serialize();
    let null_fee_payer = NullSigner::new(&fee_payer);
    let transaction = VersionedTransaction::try_new(message, &[&null_fee_payer, signer])
        .map_err(|error| SignerError::Signing(error.to_string()))?;

    if transaction.message.serialize() != canonical_message {
        return Err(SignerError::MessageMismatch);
    }
    ensure_partial_signature_shape(&transaction, &fee_payer, &approved.payer)?;

    let message_sha256 = hex_digest(&canonical_message);
    let wire = bincode::serialize(&transaction)
        .map_err(|error| SignerError::Serialization(error.to_string()))?;

    Ok(RestrictedSignature {
        x402_version: offer.x402_version,
        payload: ExactSvmPayload {
            transaction: BASE64.encode(wire),
        },
        payer: approved.payer.to_string(),
        fee_payer: fee_payer.to_string(),
        policy_fingerprint: approved.fingerprint.clone(),
        message_sha256,
        last_valid_block_height: context.last_valid_block_height,
    })
}

fn build_transfer_checked(
    token_program: &Pubkey,
    source: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
    decimals: u8,
) -> Instruction {
    let data =
        spl_token::instruction::TokenInstruction::TransferChecked { amount, decimals }.pack();
    Instruction {
        program_id: *token_program,
        accounts: vec![
            AccountMeta::new(*source, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new(*destination, false),
            AccountMeta::new_readonly(*authority, true),
        ],
        data,
    }
}

fn memo_instruction(memo: Option<&str>, fingerprint: &str) -> Result<Instruction, SignerError> {
    let data = memo
        .map(str::as_bytes)
        .unwrap_or_else(|| fingerprint.as_bytes());
    if data.len() > MAX_X402_MEMO_BYTES {
        return Err(SignerError::MemoTooLong);
    }
    Ok(Instruction {
        program_id: parse_pubkey("memo program", MEMO_PROGRAM)?,
        accounts: Vec::new(),
        data: data.to_vec(),
    })
}

fn ensure_partial_signature_shape(
    transaction: &VersionedTransaction,
    fee_payer: &Pubkey,
    payer: &Pubkey,
) -> Result<(), SignerError> {
    let account_keys = transaction.message.static_account_keys();
    if account_keys.first() != Some(fee_payer) {
        return Err(SignerError::MessageMismatch);
    }
    let payer_index = account_keys
        .iter()
        .position(|key| key == payer)
        .ok_or(SignerError::MessageMismatch)?;
    if transaction.signatures.first() != Some(&Signature::default())
        || transaction.signatures.get(payer_index) == Some(&Signature::default())
    {
        return Err(SignerError::Signing(
            "unexpected partial signature set".into(),
        ));
    }
    Ok(())
}

fn policy_fingerprint(offer: &PaymentOffer, payer: &Pubkey) -> Result<String, SignerError> {
    let encoded = serde_json::to_vec(&(offer, payer.to_string()))
        .map_err(|error| SignerError::Serialization(error.to_string()))?;
    Ok(hex_digest(&encoded))
}

fn hex_digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn parse_pubkey(field: &'static str, value: &str) -> Result<Pubkey, SignerError> {
    Pubkey::from_str(value).map_err(|_| SignerError::InvalidPubkey {
        field,
        value: value.to_string(),
    })
}

fn parse_token_program(value: &str) -> Result<Pubkey, SignerError> {
    if value != TOKEN_PROGRAM && value != TOKEN_2022_PROGRAM {
        return Err(SignerError::UnsupportedTokenProgram(value.to_string()));
    }
    parse_pubkey("token program", value)
}
