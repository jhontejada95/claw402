"use client";

import { CheckCircle, ShieldCheck, WarningCircle, XCircle } from "@phosphor-icons/react";
import { useMemo, useState } from "react";

const SOLANA = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";
const USDC = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const MERCHANT = "12Ec2cJmfR1C9uwejzxcuMhUgEC7wDrLgm1wBvvR5w9E";
const FEE_PAYER = "GVJJ7rdGiXr5xaYbRwRbjfaJL7fmwRygFi1H6aGqDveb";

type Offer = {
  amount: string;
  network: string;
  asset: string;
  merchant: string;
  feePayer: string;
  host: string;
};

const validOffer: Offer = {
  amount: "7000",
  network: SOLANA,
  asset: USDC,
  merchant: MERCHANT,
  feePayer: FEE_PAYER,
  host: "api.exa.ai",
};

export function PolicyPlayground() {
  const [offer, setOffer] = useState(validOffer);
  const [preset, setPresetName] = useState("valid");

  const evaluation = useMemo(() => {
    const checks = [
      { label: "Request cap", pass: Number(offer.amount) > 0 && Number(offer.amount) <= 10_000 },
      { label: "Network pinned", pass: offer.network === SOLANA },
      { label: "Approved USDC mint", pass: offer.asset === USDC },
      { label: "Merchant allowlisted", pass: offer.merchant === MERCHANT },
      { label: "Fee payer approved", pass: offer.feePayer === FEE_PAYER },
      { label: "Hostname approved", pass: offer.host === "api.exa.ai" },
    ];
    const hardFailure = checks.slice(0, 3).some((check) => !check.pass);
    const approval = !hardFailure && checks.slice(3).some((check) => !check.pass);
    return { checks, decision: hardFailure ? "DENY" : approval ? "APPROVAL REQUIRED" : "ALLOW" };
  }, [offer]);

  const setPreset = (next: "valid" | "cap" | "drift") => {
    setPresetName(next);
    setOffer(
      next === "valid"
        ? validOffer
        : next === "cap"
          ? { ...validOffer, amount: "25000" }
          : { ...validOffer, feePayer: "D6ZhtNQ5nT9ZnTHUbqXZsTx5MH2rPFiBBggX4hY1WePM" },
    );
  };

  const statusClass = evaluation.decision.toLowerCase().replaceAll(" ", "-");
  const StatusIcon = evaluation.decision === "ALLOW" ? CheckCircle : evaluation.decision === "DENY" ? XCircle : WarningCircle;

  return (
    <div className="playground-shell">
      <div className="preset-row" role="group" aria-label="Offer presets">
        {[
          ["valid", "Valid offer"],
          ["cap", "Cap exceeded"],
          ["drift", "Fee payer drift"],
        ].map(([key, label]) => (
          <button className={preset === key ? "selected" : ""} type="button" onClick={() => setPreset(key as "valid" | "cap" | "drift")} key={key}>
            {label}
          </button>
        ))}
      </div>
      <div className="playground-grid">
        <form className="policy-form" onSubmit={(event) => event.preventDefault()}>
          <div className="form-header">
            <ShieldCheck size={22} weight="duotone" />
            <div><strong>Incoming x402 offer</strong><span>Editable simulation · no wallet connected</span></div>
          </div>
          {[
            ["amount", "Amount · atomic USDC"],
            ["network", "Network"],
            ["asset", "Asset mint"],
            ["merchant", "Merchant"],
            ["feePayer", "Facilitator fee payer"],
            ["host", "Resource host"],
          ].map(([key, label]) => (
            <label key={key}>{label}
              <input value={offer[key as keyof Offer]} onChange={(event) => setOffer({ ...offer, [key]: event.target.value })} />
            </label>
          ))}
        </form>
        <div className={`decision-card ${statusClass}`} aria-live="polite">
          <div className="decision-header">
            <span>CLAW402 / POLICY DECISION</span>
            <StatusIcon size={24} weight="fill" />
          </div>
          <strong>{evaluation.decision}</strong>
          <div className="decision-reasons">
            {evaluation.checks.map((check, index) => (
              <p className={check.pass ? "passed" : "failed"} key={check.label}>
                <span>{String(index + 1).padStart(2, "0")}</span>
                {check.label}
                <b>{check.pass ? "PASS" : "BLOCK"}</b>
              </p>
            ))}
          </div>
          <pre>{JSON.stringify({
            decision: evaluation.decision.toLowerCase().replaceAll(" ", "_"),
            amountAtomic: Number(offer.amount) || null,
            host: offer.host,
          }, null, 2)}</pre>
        </div>
      </div>
    </div>
  );
}
