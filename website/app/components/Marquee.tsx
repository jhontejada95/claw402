const foundations = [
  "Deterministic policy engine",
  "Fail-closed by default",
  "No private keys in prompts",
  "x402 v2",
  "Solana",
  "Rust + WebAssembly",
];

export function Marquee() {
  return (
    <section className="trust-strip" aria-label="Technical foundations">
      <div className="marquee-track">
        {[0, 1].map((group) => (
          <div className="marquee-group" aria-hidden={group === 1} key={group}>
            {foundations.map((item) => <span key={`${group}-${item}`}>{item}</span>)}
          </div>
        ))}
      </div>
    </section>
  );
}
