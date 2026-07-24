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
            <h1>Let your agents buy. <em>Keep your wallet locked.</em></h1>
            <p>
              Claw402 lets autonomous agents purchase paid APIs while deterministic
              policy controls every amount, recipient, asset, and network.
            </p>
            <div className="button-row">
              <a className="button" href="/developers#install">Install the firewall <ArrowRight size={16} weight="bold" /></a>
              <a className="button button-ghost" href="/product#playground">See it block a payment</a>
            </div>
            <div className="hero-facts" aria-label="Current release facts">
              <span><b>CAPS</b> enforced</span>
              <span><b>0</b> keys in model</span>
              <span><b>FAIL</b> closed</span>
            </div>
          </div>
          <AnimatedPolicyTerminal />
        </section>

        <Marquee />

        <section className="section-wrap problem-section">
          <div className="problem-intro">
            <div className="section-label">01 / The control gap</div>
            <h2>Your agents need access. <em>Your wallet needs boundaries.</em></h2>
            <p>
              Blocking paid services makes agents less useful. Giving a model direct
              wallet control creates unacceptable risk. Claw402 gives them a narrow,
              enforceable path to buy what they need.
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
            <h2>The agent asks. <em>Policy decides.</em></h2>
            <p>Every paid request passes through one deterministic gate.</p>
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
              <h2>See every payment decision before value moves.</h2>
              <p>
                Every allow, denial, or approval request explains exactly why it
                passed or stopped. Test a live-shaped offer, then bring the same
                controls into ZeroClaw.
              </p>
              <ul className="feature-list">
                <li><CheckCircle size={19} weight="fill" /> Atomic-unit spending caps</li>
                <li><CheckCircle size={19} weight="fill" /> Network and asset pinning</li>
                <li><CheckCircle size={19} weight="fill" /> Merchant, fee-payer, and host approval</li>
              </ul>
              <a className="button" href="/product#playground">Test a payment <ArrowRight size={16} weight="bold" /></a>
            </div>
            <AnimatedPolicyTerminal compact />
          </div>
        </section>

        <section className="section-wrap capability-section">
          <div className="section-heading">
            <div className="section-label">04 / Built as a boundary</div>
            <h2>Rules prompts cannot rewrite.</h2>
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
          <h2>Give agents a budget—not your wallet.</h2>
          <p>Install the policy gate, define the limits, and inspect every decision.</p>
          <div className="button-row">
            <a className="button" href="/developers#install">Install the firewall <ArrowRight size={16} weight="bold" /></a>
            <a className="button button-ghost" href="/security">See how it stays locked</a>
          </div>
        </section>
      </main>
      <SiteFooter />
    </div>
  );
}
