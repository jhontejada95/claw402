import {
  ArrowRight,
  CheckCircle,
  CircleNotch,
  Compass,
  Flag,
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
          <h1>Turn paid APIs into <em>policy-controlled capabilities.</em></h1>
          <p>
            Claw402 discovers x402 offers, normalizes payment requirements, and
            evaluates them against deterministic operator policy before a signer
            is ever involved.
          </p>
          <div className="button-row">
            <a className="button" href="#playground">Try the playground <ArrowRight size={16} weight="bold" /></a>
            <a className="button button-ghost" href="/developers">Developer setup</a>
          </div>
        </section>
        <Marquee />

        <section className="section-wrap product-steps">
          <div className="section-heading">
            <div className="section-label">01 / Procurement pipeline</div>
            <h2>One controlled path from intent to decision.</h2>
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
              <div><div className="section-label">02 / Policy playground</div><h2>Try to break the firewall.</h2></div>
              <p>Change the offer and see the deterministic decision update. Frontend simulation only—no transaction or wallet connection.</p>
            </div>
            <PolicyPlayground />
          </div>
        </section>

        <section className="section-wrap roadmap-section">
          <div className="section-heading">
            <div className="section-label">03 / Delivery path</div>
            <h2>From policy firewall to agent procurement.</h2>
          </div>
          <div className="roadmap-visual">
            <div className="roadmap-axis" aria-hidden="true"><i /><i /><i /></div>
            {[
              [CheckCircle, "SHIPPED", "Policy + discovery", "Rust policy core, ZeroClaw WASM component, Bazaar discovery, adversarial tests."],
              [CircleNotch, "IN PROGRESS", "Restricted payments", "Deterministic SVM builder, transaction verification, allowance, devnet settlement."],
              [Flag, "FRONTIER", "Agent procurement", "Controlled micropayments, receipts, provider reputation, multi-service purchasing."],
            ].map(([Icon, status, title, copy], index) => (
              <article className={index === 0 ? "complete" : index === 1 ? "current" : ""} key={String(status)}>
                <div className="roadmap-marker"><Icon size={25} weight={index === 1 ? "bold" : "duotone"} /></div>
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
          <h2>Bring deterministic policy into ZeroClaw.</h2>
          <div className="button-row"><a className="button" href="/developers#install">Install component <ArrowRight size={16} weight="bold" /></a></div>
        </section>
      </main>
      <SiteFooter />
    </div>
  );
}
