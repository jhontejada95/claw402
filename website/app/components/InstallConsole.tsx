"use client";

import { Check, Copy } from "@phosphor-icons/react";
import { useState } from "react";

const SOLANA = "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp";
const USDC = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

const steps = {
  build: `rustup target add wasm32-wasip2\ncd plugin/claw402-policy\ncargo test\ncargo build --release --target wasm32-wasip2`,
  install: `zeroclaw plugin install ./plugin/claw402-policy\nzeroclaw config set plugins.enabled true\nzeroclaw plugin info claw402-policy`,
  configure: `[plugins.entries.config]\nmax_per_request_atomic = "10000"\nallowed_networks = "${SOLANA}"\nallowed_assets = "${USDC}"\nallowed_hosts = "api.exa.ai"`,
};

export function InstallConsole() {
  const [tab, setTab] = useState<keyof typeof steps>("build");
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    await navigator.clipboard.writeText(steps[tab]);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1400);
  };

  return (
    <div className="install-terminal">
      <div className="tab-list" role="tablist" aria-label="Installation method">
        {(Object.keys(steps) as Array<keyof typeof steps>).map((item) => (
          <button role="tab" aria-selected={tab === item} className={tab === item ? "selected" : ""} type="button" onClick={() => setTab(item)} key={item}>
            {item}
          </button>
        ))}
        <button className="copy-button" type="button" onClick={copy}>
          {copied ? <Check size={15} weight="bold" /> : <Copy size={15} weight="bold" />}
          {copied ? "Copied" : "Copy"}
        </button>
      </div>
      <pre><code>{steps[tab]}</code></pre>
    </div>
  );
}
