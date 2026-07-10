# IntRUSTing Games — launch checklist

Tracking for **RustyDasher** and the studio brand. Check items off as we go.

## Product (RustyDasher)

- [ ] Polish remaining UX from beta feedback
- [ ] Pick a few beta testers (friends / Rust community / small closed group)
- [ ] Collect feedback, fix blockers
- [ ] Once solid: **share the game publicly** (repo + Pages link; itch / socials optional)

## Brand & creative

- [ ] Generate cool logos
  - [ ] IntRUSTing Games mark (wordmark + icon)
  - [ ] RustyDasher key art / app icon variants
- [ ] Store logo assets under a future `brand/` or org assets repo

## Web presence

- [x] **Host RustyDasher publicly on GitHub Pages** (no separate game server for now)
  - Live: https://intrusting-games.github.io/rusty-dasher/
  - Deploy: CI on every push to `main` (Trunk → Pages)
- [x] **Cross off custom domains / separate public server** (not needed for now)
- [x] **SEO for now = this repo’s README** (+ org/repo description & topics)
  - Later optional: dedicated studio marketing site / custom domain

## Ops

- [x] HTTPS via GitHub Pages
- [x] CI deploy of `dist/` on push to `main`
- [ ] **Update Node.js in CI** — Actions still target Node 20 (deprecated; runners force Node 24). Bump `actions/checkout`, `upload-pages-artifact` / related actions when stable Node 24-ready versions land (or pin newer majors).
- [ ] Optional: analytics (privacy-friendly) once public
- [ ] Optional: custom domain later if we want a prettier URL

## Done

- [x] Gameplay vertical slice (modes, difficulty, dash, hearts, WASM)
- [x] GitHub org **IntRUSTing-Games**
- [x] Public repo **rusty-dasher**
- [x] MIT license + README + CI scaffold
- [x] GitHub Pages live play URL

---

*Hosting decision: GitHub Pages only. No separate domain or VPS until we choose to invest in brand polish.*
