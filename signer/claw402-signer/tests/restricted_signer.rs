use std::collections::HashMap;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use claw402_policy::policy::{
    PaymentExtra, PaymentOffer, PolicyConfig, SOLANA_DEVNET, USDC_DEVNET,
};
use claw402_signer::{
    approve_purchase, build_and_sign, BuildContext, SignerError, MEMO_PROGRAM, TOKEN_PROGRAM,
};
use solana_sdk::{
    message::VersionedMessage,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::VersionedTransaction,
};
use spl_token::instruction::TokenInstruction;

const MERCHANT: &str = "12Ec2cJmfR1C9uwejzxcuMhUgEC7wDrLgm1wBvvR5w9E";
const FEE_PAYER: &str = "Hc3sdEAsCGQcpgfivywog9uwtk8gUBUZgsxdME1EJy88";
const BLOCKHASH: &str = "EZ3rST5dvHmbanh75jc4PuLfV96vp9fEYBVeNk4FfM1k";

fn policy() -> PolicyConfig {
    let mut section = HashMap::new();
    section.insert("allowed_networks".into(), SOLANA_DEVNET.into());
    section.insert("allowed_assets".into(), USDC_DEVNET.into());
    section.insert("max_per_request_atomic".into(), "10_000".replace('_', ""));
    section.insert("allowed_merchants".into(), MERCHANT.into());
    section.insert("allowed_fee_payers".into(), FEE_PAYER.into());
    section.insert("allowed_hosts".into(), "api.example.com".into());
    PolicyConfig::from_section(&section)
}

fn offer(amount: &str) -> PaymentOffer {
    PaymentOffer {
        x402_version: 2,
        resource_url: "https://api.example.com/risk".into(),
        scheme: "exact".into(),
        network: SOLANA_DEVNET.into(),
        amount: amount.into(),
        asset: USDC_DEVNET.into(),
        pay_to: MERCHANT.into(),
        max_timeout_seconds: 60,
        extra: PaymentExtra {
            fee_payer: Some(FEE_PAYER.into()),
            memo: Some("claw402-test".into()),
        },
    }
}

fn context() -> BuildContext {
    BuildContext {
        recent_blockhash: BLOCKHASH.into(),
        last_valid_block_height: 291_470_237,
        token_program: TOKEN_PROGRAM.into(),
        mint_decimals: 6,
    }
}

#[test]
fn builds_canonical_partially_signed_x402_transaction() {
    let payer = Keypair::new();
    let approved = approve_purchase(offer("7000"), &policy(), &payer.pubkey().to_string()).unwrap();
    let result = build_and_sign(&approved, &context(), &payer).unwrap();

    let wire = BASE64.decode(&result.payload.transaction).unwrap();
    let transaction: VersionedTransaction = bincode::deserialize(&wire).unwrap();
    let VersionedMessage::V0(message) = &transaction.message else {
        panic!("expected canonical Solana v0 message");
    };

    assert_eq!(result.x402_version, 2);
    assert_eq!(result.last_valid_block_height, 291_470_237);
    assert_eq!(message.instructions.len(), 4);
    assert_eq!(transaction.signatures[0], Signature::default());
    assert_ne!(transaction.signatures[1], Signature::default());
    assert!(
        transaction.signatures[1].verify(payer.pubkey().as_ref(), &transaction.message.serialize())
    );

    let transfer = &message.instructions[2];
    assert_eq!(
        message.account_keys[transfer.program_id_index as usize].to_string(),
        TOKEN_PROGRAM
    );
    assert!(matches!(
        TokenInstruction::unpack(&transfer.data).unwrap(),
        TokenInstruction::TransferChecked {
            amount: 7000,
            decimals: 6
        }
    ));
    let merchant = MERCHANT.parse::<Pubkey>().unwrap();
    let mint = USDC_DEVNET.parse::<Pubkey>().unwrap();
    let token_program = TOKEN_PROGRAM.parse::<Pubkey>().unwrap();
    let expected_destination =
        spl_associated_token_account::get_associated_token_address_with_program_id(
            &merchant,
            &mint,
            &token_program,
        );
    assert_eq!(
        message.account_keys[transfer.accounts[2] as usize],
        expected_destination
    );
    assert_eq!(
        message.account_keys[message.instructions[3].program_id_index as usize].to_string(),
        MEMO_PROGRAM
    );
}

#[test]
fn refuses_to_sign_for_a_different_payer() {
    let approved_payer = Keypair::new();
    let attacker = Keypair::new();
    let approved = approve_purchase(
        offer("7000"),
        &policy(),
        &approved_payer.pubkey().to_string(),
    )
    .unwrap();

    assert!(matches!(
        build_and_sign(&approved, &context(), &attacker),
        Err(SignerError::SignerMismatch)
    ));
}

#[test]
fn refuses_an_offer_above_the_policy_cap_before_signing() {
    let payer = Keypair::new();
    assert!(matches!(
        approve_purchase(offer("10001"), &policy(), &payer.pubkey().to_string()),
        Err(SignerError::PolicyRejected(_))
    ));
}

#[test]
fn refuses_unknown_token_programs() {
    let payer = Keypair::new();
    let approved = approve_purchase(offer("7000"), &policy(), &payer.pubkey().to_string()).unwrap();
    let mut untrusted_context = context();
    untrusted_context.token_program = "11111111111111111111111111111111".into();

    assert!(matches!(
        build_and_sign(&approved, &untrusted_context, &payer),
        Err(SignerError::UnsupportedTokenProgram(_))
    ));
}

#[test]
fn refuses_to_use_the_buyer_as_facilitator_fee_payer() {
    let payer = Keypair::new();
    let mut unsafe_offer = offer("7000");
    unsafe_offer.extra.fee_payer = Some(payer.pubkey().to_string());

    let mut unsafe_policy = policy();
    unsafe_policy
        .allowed_fee_payers
        .insert(payer.pubkey().to_string());
    let approved =
        approve_purchase(unsafe_offer, &unsafe_policy, &payer.pubkey().to_string()).unwrap();

    assert!(matches!(
        build_and_sign(&approved, &context(), &payer),
        Err(SignerError::FeePayerNotIsolated)
    ));
}
