use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use claw402_policy::policy::{PaymentOffer, PolicyConfig, SOLANA_DEVNET};
use claw402_signer::{
    approve_purchase, build_and_sign,
    facilitator::{FacilitatorClient, PaymentPayloadV2},
    rpc::TrustedRpcClient,
};
use solana_sdk::{signature::read_keypair_file, signer::Signer};

const CONFIRMATION: &str = "SETTLE_DEVNET";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse()?;
    if args.confirm != CONFIRMATION {
        return Err(format!(
            "refusing settlement: pass --confirm {CONFIRMATION} after reviewing the offer and policy"
        )
        .into());
    }

    let offer: PaymentOffer = serde_json::from_slice(&fs::read(&args.offer)?)?;
    if offer.network != SOLANA_DEVNET {
        return Err("the devnet runner refuses non-devnet offers".into());
    }
    let policy = load_policy(&args.policy)?;
    let payer = read_keypair_file(&args.wallet)
        .map_err(|error| format!("failed to read isolated devnet wallet: {error}"))?;

    let approved = approve_purchase(offer, &policy, &payer.pubkey().to_string())?;
    let context = TrustedRpcClient::new(args.rpc)?.resolve_context(&approved.offer().asset)?;
    let signature = build_and_sign(&approved, &context, &payer)?;
    let payment =
        PaymentPayloadV2::from_approved(&approved, &signature, args.description, args.mime_type)?;

    let receipt = FacilitatorClient::new(args.facilitator)?
        .verify_and_settle(&approved, &signature, &payment)?;
    if let Some(parent) = args.receipt.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&args.receipt, serde_json::to_vec_pretty(&receipt)?)?;

    println!("settlement_success=true");
    println!("network={}", receipt.network);
    println!("payer={}", receipt.payer);
    println!("amount_atomic={}", receipt.amount);
    println!("asset={}", receipt.asset);
    println!("transaction={}", receipt.transaction);
    println!("receipt_path={}", args.receipt.display());
    println!("secret_printed=false");
    Ok(())
}

fn load_policy(path: &Path) -> Result<PolicyConfig, Box<dyn std::error::Error>> {
    let document: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
    let section = document
        .get("claw402_policy")
        .and_then(toml::Value::as_table)
        .ok_or("policy file has no [claw402_policy] section")?;
    let mut values = HashMap::new();
    for (key, value) in section {
        let value = value
            .as_str()
            .ok_or_else(|| format!("policy value {key} must be a quoted string"))?;
        values.insert(key.clone(), value.to_string());
    }
    Ok(PolicyConfig::from_section(&values))
}

struct Args {
    offer: PathBuf,
    policy: PathBuf,
    wallet: PathBuf,
    receipt: PathBuf,
    facilitator: String,
    rpc: String,
    description: String,
    mime_type: String,
    confirm: String,
}

impl Args {
    fn parse() -> Result<Self, Box<dyn std::error::Error>> {
        let mut values = env::args().skip(1);
        let mut args = Self {
            offer: ".tmp/claw402-devnet/offer.json".into(),
            policy: "config/claw402.devnet.toml".into(),
            wallet: ".tmp/claw402-devnet/payer.json".into(),
            receipt: ".tmp/claw402-devnet/receipt.json".into(),
            facilitator: "https://x402.org/facilitator".into(),
            rpc: "https://api.devnet.solana.com".into(),
            description: "Claw402 devnet purchase".into(),
            mime_type: "application/json".into(),
            confirm: String::new(),
        };
        while let Some(flag) = values.next() {
            let value = values
                .next()
                .ok_or_else(|| format!("missing value for {flag}"))?;
            match flag.as_str() {
                "--offer" => args.offer = value.into(),
                "--policy" => args.policy = value.into(),
                "--wallet" => args.wallet = value.into(),
                "--receipt" => args.receipt = value.into(),
                "--facilitator" => args.facilitator = value,
                "--rpc" => args.rpc = value,
                "--description" => args.description = value,
                "--mime-type" => args.mime_type = value,
                "--confirm" => args.confirm = value,
                _ => return Err(format!("unknown argument: {flag}").into()),
            }
        }
        Ok(args)
    }
}
