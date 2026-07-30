use std::{env, path::PathBuf};

use claw402_signer::budget::BudgetLedger;

const CONFIRMATION: &str = "RELEASE_PENDING";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse()?;
    if args.confirm != CONFIRMATION {
        return Err(format!(
            "refusing reconciliation: pass --confirm {CONFIRMATION} after proving no settlement occurred"
        )
        .into());
    }

    let mut ledger = BudgetLedger::open(&args.budget, args.daily_cap_atomic)?;
    ledger.release(&args.purchase_id)?;
    let snapshot = ledger.snapshot()?;

    println!("reservation_released=true");
    println!("purchase_id={}", args.purchase_id);
    println!("reserved_atomic={}", snapshot.reserved_atomic);
    println!("remaining_atomic={}", snapshot.remaining_atomic);
    Ok(())
}

struct Args {
    budget: PathBuf,
    daily_cap_atomic: u64,
    purchase_id: String,
    confirm: String,
}

impl Args {
    fn parse() -> Result<Self, Box<dyn std::error::Error>> {
        let mut values = env::args().skip(1);
        let mut budget = None;
        let mut daily_cap_atomic = None;
        let mut purchase_id = None;
        let mut confirm = None;

        while let Some(flag) = values.next() {
            let value = values
                .next()
                .ok_or_else(|| format!("missing value for {flag}"))?;
            match flag.as_str() {
                "--budget" => budget = Some(value.into()),
                "--daily-cap-atomic" => daily_cap_atomic = Some(value.parse()?),
                "--purchase-id" => purchase_id = Some(value),
                "--confirm" => confirm = Some(value),
                _ => return Err(format!("unknown argument: {flag}").into()),
            }
        }

        Ok(Self {
            budget: budget.ok_or("--budget is required")?,
            daily_cap_atomic: daily_cap_atomic.ok_or("--daily-cap-atomic is required")?,
            purchase_id: purchase_id.ok_or("--purchase-id is required")?,
            confirm: confirm.unwrap_or_default(),
        })
    }
}
