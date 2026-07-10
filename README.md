# RustyDasher

<p align="center">
  <a href="https://intrusting-games.github.io/rusty-dasher/"><strong>▶ Play in the browser</strong></a>
  &nbsp;·&nbsp;
  <code>https://intrusting-games.github.io/rusty-dasher/</code>
</p>

<p align="center">
  <strong>Wanna try out a new 2010's vibe browser game? Check out RustyDasher.</strong>
</p>

<p align="center">
  A polished 2D arcade game in Rust + Bevy 0.19<br/>
  Native desktop · WebAssembly · Phone / tablet / 1080p / 4K
</p>

<p align="center">
  <em>An <a href="https://github.com/IntRUSTing-Games">IntRUSTing Games</a> project</em>
</p>

---

## Links

| | |
|--|--|
| **Play** | [intrusting-games.github.io/rusty-dasher](https://intrusting-games.github.io/rusty-dasher/) |
| **Repo** | [github.com/IntRUSTing-Games/rusty-dasher](https://github.com/IntRUSTing-Games/rusty-dasher) |
| **Studio** | [IntRUSTing Games](https://github.com/IntRUSTing-Games) |
| **CTA** | *Wanna try out a new 2010's vibe browser game? Check out RustyDasher.* |

Dash through the field, collect yellow stars, dodge red hazards, and chain combos.
Same game on desktop and in the browser — layouts target **4K**, **1080p**,
**tablet** (portrait + landscape), and **phone** (portrait + landscape).

## Screenshots

| Menu | Mode select | Playing |
|------|-------------|---------|
| ![Menu](screenshots/01_menu.png) | ![Mode](screenshots/02_mode_select.png) | ![Play](screenshots/03_playing_classic.png) |

More under [`screenshots/`](screenshots/).

## Play

### Desktop

```bash
cargo run                   # play
cargo run -- --screenshots  # refresh screenshots/
cargo build --release
```

### Browser (WASM)

```bash
rustup target add wasm32-unknown-unknown
# install trunk: https://trunkrs.dev /

./scripts/web-build.sh              # local fast (cargo profile wasm-fast) → ./dist
./scripts/web-build.sh --release    # production-like (same as GitHub Pages)
./scripts/web-serve-dist.sh         # http://127.0.0.1:8080/
./scripts/web-serve.sh              # live reload
```

Local builds default to a faster `wasm-fast` profile (no LTO, more codegen units).
CI / Pages still use `trunk build --release` for ship-quality WASM.

Ship `dist/` to any static host (itch.io, GitHub Pages, Cloudflare Pages, nginx).

> `web-build.sh` clears Firefox’s disk cache after each build to avoid stale-WASM errors.

## Controls

On-screen help adapts to the device: **keyboard on PC**, **touch on phone/tablet**.

### Desktop (PC)

| Input | Action |
|-------|--------|
| **WASD** / **arrows** | Move |
| **Space** | Dash |
| Hold mouse / right-click | Point-to-move / dash |
| **Enter** / **Space** | Confirm / start / retry |
| **Escape** | Back / menu |
| **Up / Down** or **W / S** | Mode select — mode |
| **Left / Right** or **A / D** | Mode select — difficulty |

### Phone / tablet

On-screen **Game Boy** (portrait) / **PSP** (landscape) chrome:

| Input | Action |
|-------|--------|
| **Virtual stick** | Move (analog) |
| **DASH button** | Dash |
| Tap | Confirm / start / retry |
| Top / bottom thirds | Mode select — mode |
| Left / right sides | Mode select — difficulty |
| Two-finger / left edge | Back |

Install the PWA (or use Fullscreen) from the post-boot prompt for true full-screen play without browser chrome.

High scores: `save_data.json` (desktop), **localStorage** (web).

## Modes & difficulty

| Mode | Rules |
|------|--------|
| **Classic** | 3 hearts, levels, rising intensity |
| **Zen** | No hazards — pure collecting |
| **Survival** | 1 heart, aggressive spawns |
| **Timed** | 60-second score attack |

| Difficulty | Speed | Score |
|------------|-------|-------|
| Easy | ×0.85 | ×0.75 |
| Normal | ×1.0 | ×1.0 |
| Hard | ×1.3 | ×1.5 |
| Insane | ×1.65 | ×2.5 |

## Features

- Bevy 0.19 ECS, modular sources
- Resolution-independent mesh graphics
- Combos, magnet / shield / speed power-ups
- Dash trails, shockwaves, screen shake
- Touch + keyboard + mouse
- Screenshot tour: `cargo run -- --screenshots`

## Project layout

```
src/           game code
assets/        sprites + OGG SFX
scripts/       web build / serve / cache clear
screenshots/   visual QA
web/           browser CSS
TODO.md        launch checklist (org + product)
```

## Roadmap / todos

See **[TODO.md](TODO.md)** for the IntRUSTing Games launch checklist (domain, hosting, logos, site, beta, public release).

## System deps (Linux)

**Fedora**

```bash
sudo dnf install gcc-c++ libX11-devel alsa-lib-devel systemd-devel \
  wayland-devel libxkbcommon-devel pkgconf-pkg-config
```

**Debian / Ubuntu**

```bash
sudo apt-get install g++ pkg-config libx11-dev libasound2-dev libudev-dev \
  libwayland-dev libxkbcommon-dev
```

## License

[MIT](LICENSE) © IntRUSTing Games
