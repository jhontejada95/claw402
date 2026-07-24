import {
  ArrowRight,
  Bug,
  CheckCircle,
  Coins,
  Fingerprint,
  LockKey,
  ShieldCheck,
  Wallet,
  Warning,
} from "@phosphor-icons/react/dist/ssr";
import { AnimatedPolicyTerminal } from "./components/AnimatedPolicyTerminal";
import { Marquee } from "./components/Marquee";
import { SiteFooter } from "./components/SiteFooter";
import { SiteHeader } from "./components/SiteHeader";

export default function Home() {
  return (
    <div className="site-shell">
      <SiteHeader />
      <main id="top">
        <section className="hero section-wrap">
          <div className="hero-copy">
            <div className="eyebrow"><i /> Open source · Rust/WASM · Built for ZeroClaw</div>
            <h1>Give agents purchasing power. <em>Not wallet control.</em></h1>
            <p>
              Claw402 is a deterministic payment firewall for self-hosted agents.
              Discover paid x402 APIs, inspect every payment field, and enforce the
              operator&apos;s policy before value can move.
            </p>
            <div className="button-row">
              <a className="button" href="/developers#install">Install Claw402 <ArrowRight size={16} weight="bold" /></a>
              <a className="button button-ghost" href="/product">Explore product</a>
            </div>
            <div className="hero-facts" aria-label="Current release facts">
              <span><b>09</b> adversarial tests</span>
              <span><b>15</b> live offers screened</span>
              <span><b>00</b> keys exposed</span>
            </div>
          </div>
          <AnimatedPolicyTerminal />
        </section>

        <Marquee />

        <section className="section-wrap problem-section">
          <div className="problem-intro">
            <div className="section-label">01 / The control gap</div>
            <h2>Autonomous agents need <em>economic access.</em></h2>
            <p>
              Giving an LLM direct control over a wallet is reckless. Blocking all
              paid services makes the agent less useful. Claw402 creates the
              deterministic boundary between those two extremes.
            </p>
            <a className="text-link" href="/security">Read the security model <ArrowRight size={15} weight="bold" /></a>
          </div>
          <div className="risk-stack">
            <article className="risk-card risk-red">
              <div className="icon-tile"><Bug size={25} weight="duotone" /></div>
              <span>THREAT 01</span>
              <h3>Prompt-driven spending</h3>
              <p>Malicious instructions can redirect funds or push an agent beyond its intended budget.</p>
            </article>
            <article className="risk-card risk-amber">
              <div className="icon-tile"><Wallet size={25} weight="duotone" /></div>
              <span>THREAT 02</span>
              <h3>Unlimited wallet authority</h3>
              <p>Standard signing access has no context for the agent&apos;s intent, scope, or per-request limit.</p>
            </article>
            <article className="drift-card">
              <Warning size={22} weight="fill" />
              <div><strong>Payment metadata drift</strong><span>Changed recipients and fee payers stop for operator approval.</span></div>
            </article>
          </div>
        </section>

        <section className="section-wrap execution-section">
          <div className="section-heading centered">
            <div className="section-label">02 / Secure execution flow</div>
            <h2>The model proposes. <em>Claw402 verifies.</em></h2>
            <p>Probabilistic intent enters. A machine-readable policy decision leaves.</p>
          </div>
          <div className="flow-system" aria-label="Claw402 secure execution flow">
            <div className="flow-node">
              <div className="flow-icon"><Fingerprint size={28} weight="duotone" /></div>
              <span>01 / INTENT</span><strong>Agent request</strong><p>Requests a paid capability</p>
            </div>
            <div className="flow-connector"><span /><ArrowRight size={20} weight="bold" /></div>
            <div className="flow-node primary">
              <div className="status-chip">DETERMINISTIC GATE</div>
              <div className="flow-icon"><ShieldCheck size={31} weight="duotone" /></div>
              <span>02 / POLICY</span><strong>Claw402 firewall</strong><p>Validates limits and payment fields</p>
            </div>
            <div className="flow-connector muted"><span /><ArrowRight size={20} weight="bold" /></div>
            <div className="flow-node">
              <div className="flow-icon"><Coins size={28} weight="duotone" /></div>
              <span>03 / RESULT</span><strong>Paid API</strong><p>Receives authorized execution</p>
            </div>
          </div>
          <div className="flow-caption">
            <LockKey size={18} weight="duotone" />
            Restricted signer is the next settlement milestone. The current release performs discovery and policy evaluation.
          </div>
        </section>

        <section className="product-preview">
          <div className="section-wrap preview-grid">
            <div>
              <div className="section-label">03 / Product surface</div>
              <h2>Policy decisions you can inspect.</h2>
              <p>
                Every allow, denial, or approval request carries an explicit reason.
                Test offers in the playground, then move the same controls into the
                ZeroClaw runtime.
              </p>
              <ul className="feature-list">
                <li><CheckCircle size={19} weight="fill" /> Atomic-unit spending caps</li>
                <li><CheckCircle size={19} weight="fill" /> Network and asset pinning</li>
                <li><CheckCircle size={19} weight="fill" /> Merchant, fee-payer, and host approval</li>
              </ul>
              <a className="button" href="/product#playground">Open policy playground <ArrowRight size={16} weight="bold" /></a>
            </div>
            <AnimatedPolicyTerminal compact />
          </div>
        </section>

        <section className="section-wrap capability-section">
          <div className="section-heading">
            <div className="section-label">04 / Built as a boundary</div>
            <h2>Hard limits live outside the prompt.</h2>
          </div>
          <div className="capability-grid">
            {[
              [ShieldCheck, "Policy first", "Every payment field is checked against operator-owned configuration."],
              [LockKey, "Keys stay out", "Private keys never enter the prompt, discovery layer, or policy input."],
              [CheckCircle, "Fail closed", "Missing or changed metadata narrows authority instead of expanding it."],
            ].map(([Icon, title, copy]) => (
              <article key={String(title)}>
                <Icon size={30} weight="duotone" />
                <h3>{String(title)}</h3>
                <p>{String(copy)}</p>
              </article>
            ))}
          </div>
          <div className="section-actions">
            <a className="text-link" href="/security">Explore trust boundaries <ArrowRight size={15} weight="bold" /></a>
            <a className="text-link" href="/developers">View developer setup <ArrowRight size={15} weight="bold" /></a>
          </div>
        </section>

        <section className="final-cta section-wrap">
          <div className="section-label">CLAW402 / OPEN SOURCE</div>
          <h2>Procurement rails for agents—without an unrestricted wallet.</h2>
          <p>Install the policy component, run the tests, and inspect every decision.</p>
          <div className="button-row">
            <a className="button" href="/developers#install">Start building <ArrowRight size={16} weight="bold" /></a>
            <a className="button button-ghost" href="/security">Review security</a>
          </div>
        </section>
      </main>
      <SiteFooter />
    </div>
  );
}
