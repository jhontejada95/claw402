use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use claw402_policy::policy::{PaymentOffer, PolicyConfig, SOLANA_DEVNET};
use claw402_signer::{
    approve_purchase,
    budget::BudgetLedger,
    build_and_sign,
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
    let (policy, daily_cap_atomic) = load_config(&args.policy)?;
    let payer = read_keypair_file(&args.wallet)
        .map_err(|error| format!("failed to read isolated devnet wallet: {error}"))?;

    let approved = approve_purchase(offer, &policy, &payer.pubkey().to_string())?;
    let context = TrustedRpcClient::new(args.rpc)?.resolve_context(&approved.offer().asset)?;
    let amount_atomic = approved.offer().amount.parse::<u64>()?;
    let purchase_id = format!("{}:{}", approved.fingerprint(), context.recent_blockhash);
    let mut budget = BudgetLedger::open(&args.budget, daily_cap_atomic)?;
    let reservation = budget.reserve(&purchase_id, amount_atomic)?;

    let prepared = (|| -> Result<_, Box<dyn std::error::Error>> {
        let signature = build_and_sign(&approved, &context, &payer)?;
        let payment = PaymentPayloadV2::from_approved(
            &approved,
            &signature,
            args.description,
            args.mime_type,
        )?;
        Ok((signature, payment))
    })();
    let (signature, payment) = match prepared {
        Ok(prepared) => prepared,
        Err(error) => {
            budget.release(&purchase_id)?;
            return Err(error);
        }
    };

    // Once the signed payload can be submitted, any facilitator/network error
    // is ambiguous. Keep the reservation pending so a lost success response
    // cannot silently restore spending capacity.
    let receipt = FacilitatorClient::new(args.facilitator)?
        .verify_and_settle(&approved, &signature, &payment)
        .map_err(|error| {
            format!("{error}; budget reservation {purchase_id} remains pending for reconciliation")
        })?;
    budget.settle(&purchase_id, &receipt.transaction)?;
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
    println!("daily_cap_atomic={}", reservation.cap_atomic);
    println!("daily_remaining_atomic={}", reservation.remaining_atomic);
    println!("receipt_path={}", args.receipt.display());
    println!("secret_printed=false");
    Ok(())
}

fn load_config(path: &Path) -> Result<(PolicyConfig, u64), Box<dyn std::error::Error>> {
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
    let daily_cap_atomic = document
        .get("claw402_budget")
        .and_then(toml::Value::as_table)
        .and_then(|budget| budget.get("daily_cap_atomic"))
        .and_then(toml::Value::as_str)
        .ok_or("policy file has no quoted [claw402_budget].daily_cap_atomic")?
        .parse::<u64>()?;
    Ok((PolicyConfig::from_section(&values), daily_cap_atomic))
}

struct Args {
    offer: PathBuf,
    policy: PathBuf,
    wallet: PathBuf,
    budget: PathBuf,
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
            budget: ".tmp/claw402-devnet/budget.sqlite".into(),
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
                "--budget" => args.budget = value.into(),
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
