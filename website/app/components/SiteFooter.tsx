import Link from "next/link";

export function SiteFooter() {
  return (
    <footer>
      <Link className="wordmark" href="/">
        Claw<span>402</span>
      </Link>
      <div className="footer-links">
        <Link href="/product">Product</Link>
        <Link href="/security">Security</Link>
        <Link href="/developers">Developers</Link>
      </div>
      <p>Open source · Rust/WASM · 2026</p>
    </footer>
  );
}
