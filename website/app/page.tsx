"use client";

import { useMemo, useState } from "react";

const SOLANA_MAINNET = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";
const USDC_MINT = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const EXA_MERCHANT = "12Ec2cJmfR1C9uwejzxcuMhUgEC7wDrLgm1wBvvR5w9E";
const EXA_FEE_PAYER = "GVJJ7rdGiXr5xaYbRwRbjfaJL7fmwRygFi1H6aGqDveb";

type PolicyForm = {
  amount: string;
  network: string;
  asset: string;
  merchant: string;
  feePayer: string;
  host: string;
};

const validOffer: PolicyForm = {
  amount: "7000",
  network: SOLANA_MAINNET,
  asset: USDC_MINT,
  merchant: EXA_MERCHANT,
  feePayer: EXA_FEE_PAYER,
  host: "api.exa.ai",
};

const installSteps = {
  build: `rustup target add wasm32-wasip2\ncd plugin/claw402-policy\ncargo test\ncargo build --release --target wasm32-wasip2`,
  install: `zeroclaw plugin install ./plugin/claw402-policy\nzeroclaw config set plugins.enabled true\nzeroclaw plugin info claw402-policy`,
  configure: `[plugins.entries.config]\nmax_per_request_atomic = "10000"\nallowed_networks = "${SOLANA_MAINNET}"\nallowed_assets = "${USDC_MINT}"\nallowed_hosts = "api.exa.ai"`,
};

function short(value: string) {
  return value.length > 18 ? `${value.slice(0, 9)}…${value.slice(-6)}` : value;
}

export default function Home() {
  const [offer, setOffer] = useState<PolicyForm>(validOffer);
  const [installTab, setInstallTab] = useState<keyof typeof installSteps>("build");
  const [copied, setCopied] = useState(false);

  const evaluation = useMemo(() => {
    const reasons: string[] = [];
    let decision: "ALLOW" | "APPROVAL REQUIRED" | "DENY" = "ALLOW";
    const amount = Number(offer.amount);

    if (!Number.isSafeInteger(amount) || amount <= 0) {
      decision = "DENY";
      reasons.push("Amount must be a positive atomic integer");
    } else if (amount > 10_000) {
      decision = "DENY";
      reasons.push("Per-request cap exceeded");
    } else {
      reasons.push("Amount is within the 0.01 USDC cap");
    }

    if (offer.network !== SOLANA_MAINNET) {
      decision = "DENY";
      reasons.push("Network is not pinned to Solana mainnet");
    } else {
      reasons.push("Network matches operator policy");
    }

    if (offer.asset !== USDC_MINT) {
      decision = "DENY";
      reasons.push("Asset mint is not approved USDC");
    } else {
      reasons.push("Asset mint matches approved USDC");
    }

    if (decision !== "DENY") {
      if (offer.merchant !== EXA_MERCHANT) {
        decision = "APPROVAL REQUIRED";
        reasons.push("Merchant is valid but not allowlisted");
      } else {
        reasons.push("Merchant is allowlisted");
      }
      if (offer.feePayer !== EXA_FEE_PAYER) {
        decision = "APPROVAL REQUIRED";
        reasons.push("Facilitator fee payer changed");
      } else {
        reasons.push("Facilitator fee payer is approved");
      }
      if (offer.host !== "api.exa.ai") {
        decision = "APPROVAL REQUIRED";
        reasons.push("Resource hostname requires operator approval");
      } else {
        reasons.push("Resource hostname is approved");
      }
    }

    return { decision, reasons };
  }, [offer]);

  const setPreset = (name: "valid" | "cap" | "drift") => {
    setOffer(
      name === "valid"
        ? validOffer
        : name === "cap"
          ? { ...validOffer, amount: "25000" }
          : { ...validOffer, feePayer: "D6ZhtNQ5nT9ZnTHUbqXZsTx5MH2rPFiBBggX4hY1WePM" },
    );
  };

  const copyInstall = async () => {
    await navigator.clipboard.writeText(installSteps[installTab]);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1500);
  };

  return (
    <div className="site-shell">
      <nav className="topbar" aria-label="Primary navigation">
        <a className="wordmark" href="#top" aria-label="Claw402 home">
          Claw<span>402</span>
        </a>
        <div className="nav-links">
          <a href="#product">Product</a>
          <a href="#security">Security</a>
          <a href="#architecture">Architecture</a>
          <a href="#install">Install</a>
        </div>
        <a className="button button-small" href="/downloads/claw402-policy.wasm" download>
          Download WASM
        </a>
      </nav>

      <main id="top">
        <section className="hero section-wrap">
          <div className="hero-copy">
            <div className="eyebrow"><i /> Open-source · Rust/WASM · Built for ZeroClaw</div>
            <h1>Give agents purchasing power. <em>Not wallet control.</em></h1>
            <p>
              Claw402 lets self-hosted agents discover paid x402 APIs and evaluate
              every Solana payment under deterministic operator policy.
            </p>
            <div className="button-row">
              <a className="button" href="#install">Install Claw402</a>
              <a className="button button-ghost" href="#security">Read the threat model</a>
            </div>
            <div className="hero-facts">
              <span><b>9</b> adversarial tests</span>
              <span><b>15</b> live offers screened</span>
              <span><b>0</b> keys exposed</span>
            </div>
          </div>

          <div className="terminal" aria-label="Example Claw402 policy decision">
            <div className="scanlines" />
            <div className="terminal-head">
              <span>claw402_policy / inspect_offer</span>
              <strong>ALLOW</strong>
            </div>
            <div className="terminal-request">
              <p><span>REQ</span> Exa Search API</p>
              <p><span>NET</span> Solana Mainnet</p>
              <p><span>CST</span> 0.007 USDC</p>
              <p><span>TGT</span> {short(EXA_MERCHANT)}</p>
            </div>
            <div className="terminal-rule">// Evaluating operator policy…</div>
            {[
              "Network pinned",
              "USDC mint verified",
              "Amount ≤ 0.01 USDC",
              "Merchant + fee payer approved",
              "Hostname allowlisted",
            ].map((item) => <p className="terminal-ok" key={item}><span>[OK]</span> {item}</p>)}
            <div className="terminal-foot">POLICY AUTHORIZED</div>
          </div>
        </section>

        <section className="trust-strip" aria-label="Technical foundations">
          {[
            "Deterministic policy engine",
            "Fail-closed by default",
            "No private keys in prompts",
            "x402 v2",
            "Solana",
            "Rust + WebAssembly",
          ].map((item) => <span key={item}>{item}</span>)}
        </section>

        <section className="section-wrap split-section" id="product">
          <div>
            <div className="section-label">01 / Problem</div>
            <h2>Economic access without unrestricted authority.</h2>
          </div>
          <div className="problem-grid">
            <article className="risk-card risk-red">
              <span>01</span><h3>Prompt-driven spending</h3>
              <p>A malicious instruction should never be able to redirect funds or raise its own budget.</p>
            </article>
            <article className="risk-card risk-amber">
              <span>02</span><h3>Payment metadata drift</h3>
              <p>Merchants and facilitators change. Claw402 stops until the operator explicitly approves them.</p>
            </article>
            <article className="risk-card">
              <span>03</span><h3>Unlimited wallet authority</h3>
              <p>The language model proposes an intent. It never receives an unrestricted signing primitive.</p>
            </article>
          </div>
        </section>

        <section className="section-wrap flow-section">
          <div className="section-heading">
            <div className="section-label">02 / Execution path</div>
            <h2>The model proposes. Claw402 verifies.</h2>
          </div>
          <div className="flow-grid">
            {[
              ["01", "Agent intent", "Requests a paid capability"],
              ["02", "Bazaar discovery", "Finds x402 service offers"],
              ["03", "Policy firewall", "Validates every payment field"],
              ["04", "Restricted signer", "Upcoming milestone"],
              ["05", "Paid API", "Returns the purchased result"],
            ].map(([n, title, copy], index) => (
              <article className={index === 2 ? "flow-card active" : "flow-card"} key={n}>
                <span>{n}</span><h3>{title}</h3><p>{copy}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="playground-section" id="playground">
          <div className="section-wrap">
            <div className="section-heading playground-heading">
              <div><div className="section-label">03 / Policy playground</div><h2>Try to break the firewall.</h2></div>
              <p>Frontend simulation. No transaction is created and no wallet is connected.</p>
            </div>
            <div className="preset-row">
              <button onClick={() => setPreset("valid")}>Valid Exa offer</button>
              <button onClick={() => setPreset("cap")}>Cap exceeded</button>
              <button onClick={() => setPreset("drift")}>Fee payer changed</button>
            </div>
            <div className="playground-grid">
              <form className="policy-form" onSubmit={(event) => event.preventDefault()}>
                <label>Amount, atomic USDC<input value={offer.amount} onChange={(e) => setOffer({ ...offer, amount: e.target.value })} /></label>
                <label>Network<input value={offer.network} onChange={(e) => setOffer({ ...offer, network: e.target.value })} /></label>
                <label>Asset mint<input value={offer.asset} onChange={(e) => setOffer({ ...offer, asset: e.target.value })} /></label>
                <label>Merchant<input value={offer.merchant} onChange={(e) => setOffer({ ...offer, merchant: e.target.value })} /></label>
                <label>Facilitator fee payer<input value={offer.feePayer} onChange={(e) => setOffer({ ...offer, feePayer: e.target.value })} /></label>
                <label>Resource host<input value={offer.host} onChange={(e) => setOffer({ ...offer, host: e.target.value })} /></label>
              </form>
              <div className={`decision-card ${evaluation.decision.toLowerCase().replace(" ", "-")}`} aria-live="polite">
                <div className="decision-label">POLICY DECISION</div>
                <strong>{evaluation.decision}</strong>
                <div className="decision-reasons">
                  {evaluation.reasons.map((reason, index) => <p key={`${reason}-${index}`}><span>{index + 1}</span>{reason}</p>)}
                </div>
                <pre>{JSON.stringify({ decision: evaluation.decision.toLowerCase().replace(" ", "_"), amountAtomic: Number(offer.amount) || null, host: offer.host }, null, 2)}</pre>
              </div>
            </div>
          </div>
        </section>

        <section className="section-wrap proof-section">
          <div className="proof-copy">
            <div className="section-label">04 / Live proof</div>
            <h2>Tested against the live x402 Bazaar.</h2>
            <p>
              When Exa&apos;s published facilitator fee payer changed, Claw402 returned
              zero eligible offers until the operator explicitly updated the allowlist.
              External drift could not silently expand spending authority.
            </p>
          </div>
          <div className="metrics-grid">
            <div><strong>15</strong><span>resources screened</span></div>
            <div><strong>1</strong><span>policy-compliant result</span></div>
            <div><strong>$0.007</strong><span>eligible Exa offer</span></div>
            <div><strong>9/9</strong><span>native tests passing</span></div>
          </div>
        </section>

        <section className="section-wrap" id="security">
          <div className="section-heading">
            <div className="section-label">05 / Security model</div>
            <h2>Hard limits live outside the prompt.</h2>
          </div>
          <div className="security-grid">
            {[
              ["Spending caps", "Atomic-unit caps are enforced by deterministic Rust code."],
              ["Network + mint pinning", "Lookalike tokens and unexpected chains fail immediately."],
              ["Merchant allowlists", "New recipients require explicit operator approval."],
              ["Fee-payer verification", "Facilitator rotation cannot pass silently."],
              ["HTTPS host restrictions", "Approved payment metadata stays bound to the expected resource."],
              ["WASM isolation", "The plugin receives only declared ZeroClaw capabilities."],
              ["Machine-readable decisions", "Every allow, approval, and denial carries a reason."],
              ["Fail-closed defaults", "Missing configuration disables spending instead of widening it."],
            ].map(([title, copy], index) => <article key={title}><span>CONTROL {String(index + 1).padStart(2, "0")}</span><h3>{title}</h3><p>{copy}</p></article>)}
          </div>
          <blockquote>“A prompt cannot raise its own budget.”</blockquote>
        </section>

        <section className="architecture-section" id="architecture">
          <div className="section-wrap">
            <div className="section-heading">
              <div className="section-label">06 / Trust boundary</div>
              <h2>Probabilistic intent. Deterministic execution.</h2>
            </div>
            <div className="architecture-grid">
              <div className="zone zone-untrusted">
                <span>UNTRUSTED / PROBABILISTIC</span>
                <div>User prompt</div><div>Language model</div><div>Bazaar metadata</div><div>External API</div>
              </div>
              <div className="boundary"><span>INTENT</span><b>→</b><span>DECISION</span></div>
              <div className="zone zone-trusted">
                <span>TRUSTED / DETERMINISTIC</span>
                <div>Policy core</div><div>Operator config</div><div>Transaction verifier</div><div className="upcoming">Restricted signer · upcoming</div>
              </div>
            </div>
            <p className="architecture-note">Private keys never cross into the model or discovery plugin.</p>
          </div>
        </section>

        <section className="section-wrap install-section" id="install">
          <div className="install-copy">
            <div className="section-label">07 / Open source</div>
            <h2>Inspect the policy. Run the tests.</h2>
            <p>The current release is a policy and discovery milestone. Production signing and autonomous mainnet settlement are not enabled yet.</p>
            <a className="button" href="/downloads/claw402-policy.wasm" download>Download WASM component</a>
          </div>
          <div className="install-terminal">
            <div className="tab-list" role="tablist" aria-label="Installation method">
              {(Object.keys(installSteps) as Array<keyof typeof installSteps>).map((tab) => (
                <button role="tab" aria-selected={installTab === tab} className={installTab === tab ? "selected" : ""} onClick={() => setInstallTab(tab)} key={tab}>{tab}</button>
              ))}
              <button className="copy-button" onClick={copyInstall}>{copied ? "Copied" : "Copy"}</button>
            </div>
            <pre><code>{installSteps[installTab]}</code></pre>
          </div>
        </section>

        <section className="section-wrap roadmap-section">
          <div className="section-heading"><div className="section-label">08 / Roadmap</div><h2>From policy firewall to agent procurement.</h2></div>
          <div className="roadmap-grid">
            <article className="complete"><span>COMPLETED</span><h3>Policy + Discovery</h3><ul><li>Rust policy core</li><li>ZeroClaw WASM component</li><li>Live Bazaar discovery</li><li>Adversarial tests</li></ul></article>
            <article className="current"><span>IN PROGRESS</span><h3>Restricted Payments</h3><ul><li>Deterministic SVM builder</li><li>Transaction verifier</li><li>On-chain allowance</li><li>Devnet settlement</li></ul></article>
            <article><span>NEXT</span><h3>Agent Procurement</h3><ul><li>Controlled micropayments</li><li>Receipts and audit trail</li><li>Provider reputation</li><li>Multi-service purchasing</li></ul></article>
          </div>
        </section>

        <section className="section-wrap faq-section">
          <div><div className="section-label">09 / FAQ</div><h2>Direct answers.</h2></div>
          <div className="faq-list">
            {[
              ["Is Claw402 a wallet?", "No. It is a deterministic policy and verification layer between agent intent and a restricted signing boundary."],
              ["Can the model access private keys?", "No. Keys are not part of the prompt, discovery plugin, or policy input."],
              ["Does it execute mainnet payments today?", "Not yet. Live discovery and policy evaluation work today; signing and settlement are the next milestone."],
              ["What happens when payment requirements change?", "Claw402 denies or requests operator approval. The Exa fee-payer rotation was caught in a live test."],
              ["Why x402 and Solana?", "x402 gives APIs a machine-readable payment challenge, while Solana makes low-value, high-frequency service purchases practical."],
            ].map(([question, answer]) => <details key={question}><summary>{question}<span>+</span></summary><p>{answer}</p></details>)}
          </div>
        </section>

        <section className="final-cta section-wrap">
          <div className="section-label">CLAW402 / OPEN SOURCE</div>
          <h2>The payment firewall for autonomous agents.</h2>
          <p>Build the agent economy without handing prompts an unrestricted wallet.</p>
          <div className="button-row"><a className="button" href="#install">Get Claw402</a><a className="button button-ghost" href="#architecture">Explore the architecture</a></div>
        </section>
      </main>

      <footer>
        <a className="wordmark" href="#top">Claw<span>402</span></a>
        <p>ZeroClaw · x402 · Solana · Rust/WASM</p>
        <p>MIT License · 2026</p>
      </footer>
    </div>
  );
}
