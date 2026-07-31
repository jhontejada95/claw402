import {
  ArrowRight,
  CheckCircle,
  CircleNotch,
  Compass,
  Funnel,
  GearSix,
  RocketLaunch,
  ShieldCheck,
} from "@phosphor-icons/react/dist/ssr";
import { Marquee } from "../components/Marquee";
import { PolicyPlayground } from "../components/PolicyPlayground";
import { SiteFooter } from "../components/SiteFooter";
import { SiteHeader } from "../components/SiteHeader";

export default function ProductPage() {
  return (
    <div className="site-shell">
      <SiteHeader />
      <main>
        <section className="subpage-hero section-wrap">
          <div className="section-label">PRODUCT / POLICY FIREWALL</div>
          <h1>Every paid API. <em>One policy gate.</em></h1>
          <p>
            Let agents discover and purchase x402 services without giving the model
            unrestricted spending power. Every offer must clear your rules first.
          </p>
          <div className="button-row">
            <a className="button" href="#playground">Test a payment <ArrowRight size={16} weight="bold" /></a>
            <a className="button button-ghost" href="/developers">Install the firewall</a>
          </div>
        </section>
        <Marquee />

        <section className="section-wrap product-steps">
          <div className="section-heading">
            <div className="section-label">01 / Procurement pipeline</div>
            <h2>From paid request to policy verdict.</h2>
          </div>
          <div className="step-rail">
            {[
              [Compass, "Discover", "Read machine-payable service offers from the x402 Bazaar."],
              [Funnel, "Normalize", "Extract network, mint, amount, merchant, fee payer, and host."],
              [ShieldCheck, "Evaluate", "Apply caps, pinned identifiers, and allowlists in deterministic Rust."],
              [GearSix, "Respond", "Return allow, deny, or operator approval with explicit reasons."],
            ].map(([Icon, title, copy], index) => (
              <article key={String(title)}>
                <span>{String(index + 1).padStart(2, "0")}</span>
                <Icon size={27} weight="duotone" />
                <h3>{String(title)}</h3><p>{String(copy)}</p>
              </article>
            ))}
          </div>
        </section>

        <section className="playground-section" id="playground">
          <div className="section-wrap">
            <div className="section-heading playground-heading">
              <div><div className="section-label">02 / Policy playground</div><h2>See Claw402 stop a bad payment.</h2></div>
              <p>Change the amount, recipient, or fee payer and watch policy react. Simulation only—no transaction or wallet connection.</p>
            </div>
            <PolicyPlayground />
          </div>
        </section>

        <section className="section-wrap roadmap-section">
          <div className="section-heading">
            <div className="section-label">03 / Delivery path</div>
            <h2>Today: controlled devnet procurement. Next: production hardening.</h2>
          </div>
          <div className="roadmap-visual">
            <div className="roadmap-axis" aria-hidden="true"><i /><i /><i /></div>
            {[
              [CheckCircle, "SHIPPED", "Policy + discovery", "Rust policy core, ZeroClaw WASM component, Bazaar discovery, adversarial tests."],
              [CheckCircle, "SHIPPED", "Restricted devnet payments", "Deterministic SVM builder, x402 settlement, persistent budget, and verified receipts."],
              [CircleNotch, "NEXT", "Production hardening", "Operational custody, on-chain allowances, observability, and provider reputation."],
            ].map(([Icon, status, title, copy]) => (
              <article className={status === "SHIPPED" ? "complete" : "current"} key={`${String(status)}-${String(title)}`}>
                <div className="roadmap-marker"><Icon size={25} weight={status === "NEXT" ? "bold" : "duotone"} /></div>
                <span>{String(status)}</span><h3>{String(title)}</h3><p>{String(copy)}</p>
              </article>
            ))}
          </div>
          <div className="release-note">
            <RocketLaunch size={24} weight="duotone" />
            <div><strong>Current release boundary</strong><p>Production signing and autonomous mainnet settlement are intentionally not enabled yet.</p></div>
          </div>
        </section>

        <section className="final-cta section-wrap compact-cta">
          <div className="section-label">NEXT / INTEGRATE</div>
          <h2>Put a policy gate in front of agent spending.</h2>
          <div className="button-row"><a className="button" href="/developers#install">Install the firewall <ArrowRight size={16} weight="bold" /></a></div>
        </section>
      </main>
      <SiteFooter />
    </div>
  );
}
