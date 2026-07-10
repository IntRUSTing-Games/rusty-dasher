# IntRUSTing Games — launch checklist

Tracking for **RustyDasher** and the studio brand. Check items off as we go.

## Product (RustyDasher)

- [ ] Polish remaining UX from beta feedback
- [ ] Pick a few beta testers (friends / Rust community / small closed group)
- [ ] Collect feedback, fix blockers
- [ ] Once solid: **share the game publicly** (repo is ready; also itch / socials as desired)

## Brand & creative

- [ ] Generate cool logos
  - [ ] IntRUSTing Games mark (wordmark + icon)
  - [ ] RustyDasher key art / app icon variants
- [ ] Store logo assets under a future `brand/` or org assets repo

## Web presence

- [ ] Get a domain for **IntRUSTing Games** (studio)
- [ ] Get a domain for **RustyDasher** (or a path under the studio domain)
- [ ] Create an **SEO-optimized site** for IntRUSTing Games (separate domain)
  - Studio story, games grid, contact / socials
  - Meta titles, descriptions, OG images, sitemap, robots.txt
- [ ] Find a server / host to run RustyDasher **publicly**
  - Static WASM on Pages / Cloudflare / S3+CDN is enough for the game
  - Studio site can be static too (Astro, Zola, plain HTML, etc.)

## Ops

- [ ] Point DNS when domains are ready
- [ ] HTTPS everywhere
- [ ] Optional: CI deploy of `dist/` on push to `main`
- [ ] Optional: analytics (privacy-friendly) once public

## Done

- [x] Gameplay vertical slice (modes, difficulty, dash, hearts, WASM)
- [x] GitHub org **IntRUSTing-Games**
- [x] Public repo **rusty-dasher**
- [x] MIT license + README + CI scaffold

---

*Last updated: wrapping the session — next time starts with domains / logos / beta.*
