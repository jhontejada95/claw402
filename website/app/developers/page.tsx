import {
  ArrowRight,
  CheckCircle,
  CodeBlock,
  DownloadSimple,
  FileCode,
  Flask,
  Package,
  TerminalWindow,
} from "@phosphor-icons/react/dist/ssr";
import { InstallConsole } from "../components/InstallConsole";
import { SiteFooter } from "../components/SiteFooter";
import { SiteHeader } from "../components/SiteHeader";

export default function DevelopersPage() {
  return (
    <div className="site-shell">
      <SiteHeader />
      <main>
        <section className="subpage-hero section-wrap developer-hero">
          <div className="section-label">DEVELOPERS / RUST + WASM</div>
          <h1>Inspect the policy. <em>Run the tests.</em></h1>
          <p>
            Build Claw402 as a WASI component, install it into ZeroClaw, and keep
            payment authority in machine-readable operator configuration.
          </p>
          <div className="button-row">
            <a className="button" href="#install">Install from source <ArrowRight size={16} weight="bold" /></a>
            <a className="button button-ghost" href="/downloads/claw402-policy.wasm" download>Download WASM</a>
          </div>
        </section>

        <section className="developer-band">
          <div className="section-wrap developer-band-grid">
            {[
              [Package, "WASI component", "Portable policy boundary"],
              [FileCode, "Rust core", "Deterministic evaluation"],
              [Flask, "Adversarial tests", "Fail-closed behavior"],
              [TerminalWindow, "ZeroClaw native", "Declared capabilities"],
            ].map(([Icon, title, copy]) => (
              <div key={String(title)}><Icon size={24} weight="duotone" /><strong>{String(title)}</strong><span>{String(copy)}</span></div>
            ))}
          </div>
        </section>

        <section className="section-wrap install-page-section" id="install">
          <div className="install-page-copy">
            <div className="section-label">01 / Quickstart</div>
            <h2>From source to policy component.</h2>
            <p>
              The current release covers live Bazaar discovery and deterministic
              policy evaluation. Production signing and autonomous mainnet
              settlement remain intentionally disabled.
            </p>
            <div className="install-checks">
              <span><CheckCircle size={18} weight="fill" /> Build the WASI target</span>
              <span><CheckCircle size={18} weight="fill" /> Run native policy tests</span>
              <span><CheckCircle size={18} weight="fill" /> Configure explicit allowlists</span>
            </div>
          </div>
          <InstallConsole />
        </section>

        <section className="section-wrap config-section">
          <div className="section-heading">
            <div className="section-label">02 / Operator configuration</div>
            <h2>Authority is explicit, reviewable, and versionable.</h2>
          </div>
          <div className="config-grid">
            {[
              ["max_per_request_atomic", "Caps every request in the asset’s atomic unit."],
              ["allowed_networks", "Pins the network identifier accepted by policy."],
              ["allowed_assets", "Rejects lookalike or unexpected token mints."],
              ["allowed_hosts", "Binds payment metadata to approved API resources."],
            ].map(([key, copy]) => (
              <article key={key}><CodeBlock size={22} weight="duotone" /><code>{key}</code><p>{copy}</p></article>
            ))}
          </div>
        </section>

        <section className="section-wrap download-card">
          <div className="download-icon"><DownloadSimple size={34} weight="duotone" /></div>
          <div><div className="section-label">PREBUILT ARTIFACT</div><h2>Download the WASM component.</h2><p>Use the artifact for evaluation, or build from source to inspect the exact policy implementation.</p></div>
          <a className="button" href="/downloads/claw402-policy.wasm" download>Download WASM <DownloadSimple size={17} weight="bold" /></a>
        </section>

        <section className="final-cta section-wrap compact-cta">
          <div className="section-label">TEST / INSPECT / INTEGRATE</div>
          <h2>Make payment authority a runtime decision—not a prompt instruction.</h2>
          <div className="button-row"><a className="button" href="/product#playground">Try policy playground <ArrowRight size={16} weight="bold" /></a></div>
        </section>
      </main>
      <SiteFooter />
    </div>
  );
}
