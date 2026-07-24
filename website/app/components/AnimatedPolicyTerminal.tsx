"use client";

import { useEffect, useState } from "react";

const lines = [
  ["REQ", "Exa Search API"],
  ["NET", "Solana Mainnet"],
  ["CST", "0.007 USDC"],
  ["TGT", "12Ec2cJm…vR5w9E"],
];

const checks = [
  "Network pinned",
  "USDC mint verified",
  "Amount ≤ 0.01 USDC",
  "Merchant + fee payer approved",
  "Hostname allowlisted",
];

export function AnimatedPolicyTerminal({ compact = false }: { compact?: boolean }) {
  const [visible, setVisible] = useState(0);

  useEffect(() => {
    const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduced) {
      const frame = window.requestAnimationFrame(() => setVisible(checks.length));
      return () => window.cancelAnimationFrame(frame);
    }
    const timer = window.setInterval(() => {
      setVisible((value) => (value >= checks.length ? 0 : value + 1));
    }, 620);
    return () => window.clearInterval(timer);
  }, []);

  return (
    <div className={compact ? "terminal terminal-compact" : "terminal"} aria-label="Animated example Claw402 policy decision">
      <div className="scanlines" aria-hidden="true" />
      <div className="terminal-head">
        <span>claw402_policy / inspect_offer</span>
        <strong className={visible === checks.length ? "is-ready" : ""}>
          {visible === checks.length ? "ALLOW" : "RUN"}
        </strong>
      </div>
      <div className="terminal-request">
        {lines.map(([key, value]) => (
          <p key={key}><span>{key}</span>{value}</p>
        ))}
      </div>
      <div className="terminal-rule">
        <span className="prompt-symbol">&gt;</span> evaluating operator policy
        <i className="code-cursor" aria-hidden="true" />
      </div>
      <div className="terminal-checks">
        {checks.map((item, index) => (
          <p className={index < visible ? "terminal-ok visible" : "terminal-ok"} key={item}>
            <span>[OK]</span> {item}
          </p>
        ))}
      </div>
      <div className={visible === checks.length ? "terminal-foot visible" : "terminal-foot"}>
        POLICY AUTHORIZED
      </div>
    </div>
  );
}
