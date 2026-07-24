# Claw402 redesign QA

## Evidence

- Source visual truth: the 16 browser-comment screenshots supplied by the user in the current task, including the two additional reference images for the problem and execution-flow sections.
- Implementation: local Vinext preview at `http://localhost:4173`.
- Desktop viewport: 1440 × 900 CSS px, device scale factor 1.
- Mobile viewport: 390 × 844 CSS px, device scale factor 1.
- State: dark theme; Home hero animation running; Product playground tested in valid and cap-exceeded states; mobile navigation opened.
- Implementation screenshots:
  - `qa/home-desktop-final.png`
  - `qa/home-mobile-final.png`
  - `qa/product-playground.png`
  - `qa/security-architecture.png`
  - `qa/developers-install.png`
- Source images were supplied inline rather than as filesystem assets. They and the rendered screenshots were compared together in the current visual review context.

## Full-view comparison

The redesigned Home preserves the source's dark crypto-infrastructure language, neon-green state color, terminal-first hero, square controls, and dense mono labels. It intentionally changes the information architecture requested by the user: pitch-only proof is removed from Home, while detailed product, security, and developer material is split into dedicated routes.

## Focused comparison

- Hero: the headline and button labels now use strong display weights; the three facts remain on one row at desktop and 390 px; the terminal reveals policy checks sequentially.
- Problem: the revised two-card risk composition follows the attached reference's stronger left-copy/right-card hierarchy, with metadata drift retained as a compact warning rail.
- Execution flow: equal flat tiles were replaced by a connected three-node flow with Claw402 as the highlighted deterministic gate.
- Product playground: form and decision surfaces share the same visual system and expose clear PASS/BLOCK states.
- Security: trust zones now form a single boundary diagram with the policy core between untrusted and deterministic zones.
- Developers: build/install/configure tabs and copy action are visually and behaviorally integrated.

## Findings and comparison history

### Iteration 1

- P2 — Desktop hero facts collided between the second and third labels.
  - Fix: replaced equal-width grid tracks with content-sized flex tracks and explicit separators.
  - Post-fix evidence: `qa/home-desktop-final.png`; labels are distinct and remain on one line.

- P2 — Original terminal texture depended on a decorative gradient.
  - Fix: removed the gradient and retained product character through real code animation, state color, borders, and typography.
  - Post-fix evidence: `qa/home-desktop-final.png`.

### Final review

- Fonts and typography: Geist/Geist Mono are consistent; display headings and all buttons use bold optical weights; wrapping remains controlled on desktop and mobile.
- Spacing and layout rhythm: hero, cards, flow, playground, and subpage sections align to the same 1180 px content frame and responsive spacing system.
- Colors and tokens: semantic green/amber/red states are consistent; body and muted text remain legible against dark surfaces.
- Image and icon fidelity: no placeholder imagery or handcrafted SVGs are used. Interface icons come from Phosphor and match the technical visual language.
- Copy and content: claims preserve the current milestone boundary and do not imply production signing or mainnet settlement.
- Responsive behavior: tested at 1440 × 900 and 390 × 844; hero facts remain inline and the mobile menu opens correctly.
- Interactions tested: terminal animation, mobile navigation, Product presets, DENY state, developer tabs, and copy-button availability.
- Browser console: no errors or warnings observed.

## Residual P3 polish

- Very small hero fact labels at 390 px are the trade-off required to keep all three on one line, per the explicit product requirement.
- Full-page capture in the in-app browser produced stitching artifacts with fixed navigation; viewport captures were used as the reliable visual evidence.

final result: passed
