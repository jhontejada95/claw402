import {
  ArrowRight,
  CheckCircle,
  Code,
  Fingerprint,
  Globe,
  Key,
  LockKey,
  ShieldCheck,
  SlidersHorizontal,
  WarningOctagon,
} from "@phosphor-icons/react/dist/ssr";
import { SiteFooter } from "../components/SiteFooter";
import { SiteHeader } from "../components/SiteHeader";

const controls = [
  [SlidersHorizontal, "Spending policy", "Atomic-unit request caps enforced by deterministic Rust code."],
  [Globe, "Network + asset pinning", "Unexpected chains and lookalike token mints fail immediately."],
  [Fingerprint, "Identity allowlists", "Merchants, fee payers, and resource hosts require explicit approval."],
  [Code, "WASM isolation", "The component receives only capabilities declared by the ZeroClaw runtime."],
  [CheckCircle, "Machine-readable decisions", "Every allow, approval, and denial carries a reason."],
  [WarningOctagon, "Fail-closed defaults", "Missing configuration disables spending instead of widening it."],
];

export default function SecurityPage() {
  return (
    <div className="site-shell">
      <SiteHeader />
      <main>
        <section className="subpage-hero section-wrap security-hero">
          <div className="section-label">SECURITY / TRUST BOUNDARY</div>
          <h1>Probabilistic intent. <em>Deterministic execution.</em></h1>
          <p>
            Claw402 treats prompts, model output, Bazaar metadata, and external APIs
            as untrusted. Spending authority is defined by operator configuration
            and enforced outside the model.
          </p>
          <div className="button-row">
            <a className="button" href="#architecture">View architecture <ArrowRight size={16} weight="bold" /></a>
            <a className="button button-ghost" href="/product#playground">Test policy</a>
          </div>
        </section>

        <section className="architecture-section" id="architecture">
          <div className="section-wrap">
            <div className="section-heading">
              <div className="section-label">01 / Trust zones</div>
              <h2>Authority narrows at the boundary.</h2>
            </div>
            <div className="trust-diagram">
              <div className="trust-zone untrusted">
                <div className="zone-heading"><WarningOctagon size={23} weight="duotone" /><span>UNTRUSTED / PROBABILISTIC</span></div>
                <div className="zone-items"><span>User prompt</span><span>Language model</span><span>Bazaar metadata</span><span>External API</span></div>
              </div>
              <div className="firewall-core">
                <div className="boundary-label">INTENT <ArrowRight size={16} weight="bold" /></div>
                <ShieldCheck size={42} weight="duotone" />
                <strong>CLAW402</strong><span>POLICY CORE</span>
                <div className="boundary-label">DECISION <ArrowRight size={16} weight="bold" /></div>
              </div>
              <div className="trust-zone trusted">
                <div className="zone-heading"><LockKey size={23} weight="duotone" /><span>TRUSTED / DETERMINISTIC</span></div>
                <div className="zone-items"><span>Operator config</span><span>Policy engine</span><span>Transaction verifier</span><span className="upcoming">Restricted signer · next</span></div>
              </div>
            </div>
            <div className="key-boundary"><Key size={20} weight="duotone" /> Private keys never cross into the model or discovery component.</div>
          </div>
        </section>

        <section className="section-wrap controls-section">
          <div className="section-heading">
            <div className="section-label">02 / Defense in depth</div>
            <h2>Controls you can reason about.</h2>
            <p>Each control constrains a distinct field or runtime boundary; no safety claim depends on prompt compliance.</p>
          </div>
          <div className="controls-grid">
            {controls.map(([Icon, title, copy], index) => (
              <article key={String(title)}>
                <div className="control-index">CONTROL {String(index + 1).padStart(2, "0")}</div>
                <Icon size={27} weight="duotone" />
                <h3>{String(title)}</h3><p>{String(copy)}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="section-wrap security-principle">
          <ShieldCheck size={38} weight="duotone" />
          <blockquote>“A prompt cannot raise its own budget.”</blockquote>
          <p>Policy is owned by the operator, evaluated deterministically, and narrowed on uncertainty.</p>
        </section>

        <section className="section-wrap faq-section">
          <div><div className="section-label">03 / FAQ</div><h2>Direct answers.</h2></div>
          <div className="faq-list">
            {[
              ["Is Claw402 a wallet?", "No. It is a deterministic policy and verification layer between agent intent and a restricted signing boundary."],
              ["Can the model access private keys?", "No. Keys are not part of the prompt, discovery component, or policy input."],
              ["Does it execute mainnet payments today?", "Not yet. Live discovery and policy evaluation work today; restricted signing and settlement are the next milestone."],
              ["What happens when payment requirements change?", "Claw402 denies the offer or asks for operator approval. Changed metadata cannot silently expand authority."],
              ["Why x402 and Solana?", "x402 exposes machine-readable payment challenges; Solana supports the low-value, high-frequency service purchases this architecture targets."],
            ].map(([question, answer]) => (
              <details key={question}><summary>{question}<span>+</span></summary><p>{answer}</p></details>
            ))}
          </div>
        </section>

        <section className="final-cta section-wrap compact-cta">
          <div className="section-label">VERIFY / THEN BUILD</div>
          <h2>Inspect the policy before you grant authority.</h2>
          <div className="button-row"><a className="button" href="/developers">Open developer setup <ArrowRight size={16} weight="bold" /></a></div>
        </section>
      </main>
      <SiteFooter />
    </div>
  );
}
