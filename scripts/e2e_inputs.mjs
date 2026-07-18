/**
 * Unified exhaustive E2E + optional matrix PNG capture (one journey, no duplicate loads).
 *
 * Per format (default):
 *   - Record VIDEO of full surface (modes, diffs, ≥play, inputs)
 *   - At each screen: QUALITY HOLD (settle) then write matrix PNG
 *     screenshots/viewports/{format}_{01_boot|02_menu|...}.png
 *   - Parallel formats: CONCURRENCY (default 3) separate browser pages
 *
 * Env:
 *   E2E_PLAY_MS          play duration ms (default 20000)
 *   E2E_FORMATS          comma filter of format ids
 *   CAPTURE_MATRIX=0     disable matrix PNGs (video only)
 *   CONCURRENCY=3        parallel formats (1 = serial)
 *   MATRIX_ONLY=1        only matrix holds + short play (skip 20s / extra input paths)
 *
 * Reviews stay separate: video_critique.md vs matrix_critique.md (see skill).
 *
 * CAPTURE vs REVIEW: this script only does CAPTURE. Console CAPTURE_OK / CAPTURE_FAIL
 * (and results.json ok:true) mean automation wrote the artifact or step succeeded —
 * NOT visual acceptance. A4b/A6 + A7 in ui-viewport-qa / qa_success_criteria.json
 * are the review gates. Never treat suite exit 0 as "looks good."
 */
import puppeteer from 'puppeteer-core';
import { chromeExecutable, chromeGpuArgs, logChromeGlMode } from './chrome_launch.mjs';
import { applyDeviceEmulation, isHandheldFormat } from './device_emulation.mjs';
import { startPageRecording, extractReviewStills } from './record.mjs';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, '..');
const OUT = path.join(ROOT, 'screenshots/web/e2e');
const VID = path.join(OUT, 'recordings');
const STILLS = path.join(OUT, 'stills');
const MATRIX_OUT = path.join(ROOT, 'screenshots/viewports');
const MATRIX = JSON.parse(fs.readFileSync(path.join(__dirname, 'qa_matrix.json'), 'utf8'));
const PLAY_MS = Number(process.env.E2E_PLAY_MS || 20000);
const CAPTURE_MATRIX = process.env.CAPTURE_MATRIX !== '0';
const CONCURRENCY = Math.max(1, Number(process.env.CONCURRENCY || 3));
const MATRIX_ONLY = process.env.MATRIX_ONLY === '1';
const HOLD_MS = Number(process.env.MATRIX_HOLD_MS || 450);
// Force-GO delay: full e2e wants continuous ≥20s play then auto GO; matrix-only is short.
// Game reads qa_go_ms; e2e=1 alone defaults to 22.5s if qa_go_ms omitted.
const FORCE_GO_MS = MATRIX_ONLY
  ? Number(process.env.E2E_FORCE_GO_MS || 2500)
  : Number(process.env.E2E_FORCE_GO_MS || Math.max(PLAY_MS + 2500, 22500));
// Default 17880 — shared with web-serve-dist / Trunk (not 8080).
const PORT = process.env.PORT || process.env.RUSTY_PORT || '17880';
const BASE = (process.env.E2E_URL || `http://127.0.0.1:${PORT}/`).replace(/\/?$/, '/');
const URL = `${BASE}?e2e=1&qa_matrix=1&qa_go_ms=${FORCE_GO_MS}`;

const MODES = ['CLASSIC', 'ZEN', 'SURVIVAL', 'TIMED'];
const DIFFS = ['EASY', 'NORMAL', 'HARD', 'INSANE'];

fs.mkdirSync(OUT, { recursive: true });
fs.mkdirSync(VID, { recursive: true });
fs.mkdirSync(STILLS, { recursive: true });
fs.mkdirSync(MATRIX_OUT, { recursive: true });

const results = [];
/** Capture-step success only — not visual review / A7. */
function pass(name, detail = '') {
  results.push({ name, ok: true, detail, layer: 'capture' });
  console.log('CAPTURE_OK', name, detail);
}
/** Capture-step failure only — not visual review. */
function fail(name, detail = '') {
  results.push({ name, ok: false, detail, layer: 'capture' });
  console.error('CAPTURE_FAIL', name, detail);
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

logChromeGlMode();
console.log(
  `[qa] CAPTURE_MATRIX=${CAPTURE_MATRIX} CONCURRENCY=${CONCURRENCY} PLAY_MS=${PLAY_MS} HOLD_MS=${HOLD_MS} FORCE_GO_MS=${FORCE_GO_MS}`
);
console.log(`[qa] URL ${URL}`);

const browser = await puppeteer.launch({
  executablePath: chromeExecutable(),
  headless: 'new',
  args: chromeGpuArgs(),
  // Parallel formats + multi-touch can stall CDP; don't use default 180s protocol cap tightly
  protocolTimeout: 300000,
});

async function shutdownBrowser(code = 0) {
  try {
    if (browser && browser.connected !== false) await browser.close();
  } catch (_) {}
  setTimeout(() => process.exit(code), 500).unref?.();
  process.exit(code);
}
for (const sig of ['SIGINT', 'SIGTERM', 'SIGHUP']) {
  process.on(sig, () => {
    console.error(`[chrome] ${sig}: closing browser`);
    shutdownBrowser(130);
  });
}
process.on('uncaughtException', (err) => {
  console.error('[chrome] uncaughtException', err);
  shutdownBrowser(1);
});

async function newPage(format) {
  const page = await browser.newPage();
  const logs = [];
  const pageErrors = [];
  page.on('console', (msg) => logs.push(`${msg.type()}: ${msg.text()}`));
  page.on('pageerror', (err) => {
    pageErrors.push(String(err));
    logs.push('PAGEERROR ' + err);
  });
  if (isHandheldFormat(format)) {
    const { client } = await applyDeviceEmulation(page, format);
    page.__cdp = client;
  } else {
    await page.setViewport({
      width: format.width,
      height: format.height,
      deviceScaleFactor: format.dpr,
      isMobile: false,
      hasTouch: false,
    });
  }
  page.__logs = logs;
  page.__errors = pageErrors;
  return page;
}

/** Settle UI, then write a clear PNG for the layout matrix (high quality still). */
async function qualityMatrixShot(page, formatId, shotSuffix, extraHoldMs = 0) {
  if (!CAPTURE_MATRIX) return null;
  await sleep(HOLD_MS + extraHoldMs);
  // Prefer idle frames: brief wait after hold
  await sleep(100);
  const name = `${formatId}_${shotSuffix}`;
  const file = path.join(MATRIX_OUT, name + '.png');
  await page.screenshot({ path: file, type: 'png', captureBeyondViewport: false });
  const size = fs.statSync(file).size;
  console.log('[matrix]', name, size);
  if (size < 500) fail(`${formatId}/matrix ${shotSuffix}`, 'tiny file');
  else pass(`${formatId}/matrix ${shotSuffix}`, `${size}b`);
  return file;
}

/**
 * Dismiss HTML boot without keyboard.
 * Enter/Space on the page also hits Bevy Menu confirm → ModeSelect, so the
 * 02_menu matrix cell would wrongly capture SELECT MODE.
 */
async function dismissBootOverlay(page, format) {
  // HTML-only dismiss. Never send Enter/Space here — those also confirm Bevy
  // Menu → ModeSelect and poison 02_menu / later matrix cells.
  try {
    await page.waitForSelector('#boot-cta', { timeout: 5000 });
  } catch (_) {}
  for (let attempt = 0; attempt < 6; attempt++) {
    const hidden = await page.evaluate(() => {
      const boot = document.getElementById('boot');
      if (boot?.classList.contains('hidden')) return true;
      const el = document.getElementById('boot-cta');
      if (el) {
        el.style.display = 'inline-block';
        el.click();
      }
      boot?.classList.add('hidden');
      return !!boot?.classList.contains('hidden');
    });
    if (hidden) return true;
    // Pointer on CTA box only (not canvas center — that can hit Menu confirm).
    try {
      const box = await page.evaluate(() => {
        const el = document.getElementById('boot-cta');
        if (!el) return null;
        const r = el.getBoundingClientRect();
        return { x: r.x + r.width / 2, y: r.y + r.height / 2 };
      });
      if (box) {
        if (format.touch) await page.touchscreen.tap(box.x, box.y);
        else await page.mouse.click(box.x, box.y);
      }
    } catch (_) {}
    await sleep(300);
  }
  return page.evaluate(() =>
    document.getElementById('boot')?.classList.contains('hidden')
  );
}

/**
 * Play session with move + optional dash.
 * @param {{ dash?: boolean }} opts — when dash=false, never press Space (avoids
 *   GameOver "again" restart near force-GO).
 */
async function playSessionKeyboard(page, ms, opts = {}) {
  const allowDash = opts.dash !== false;
  const end = Date.now() + ms;
  let step = 0;
  const keys = ['KeyW', 'KeyA', 'KeyS', 'KeyD', 'ArrowUp', 'ArrowLeft', 'ArrowDown', 'ArrowRight'];
  try {
    while (Date.now() < end) {
      const k = keys[step % keys.length];
      await page.keyboard.down(k);
      await sleep(200);
      await page.keyboard.up(k);
      // Space = dash while Playing; on GameOver it restarts — stop dash near end
      // of the overall force window by calling with { dash: false }.
      if (allowDash && step % 3 === 0 && Date.now() + 500 < end) {
        await page.keyboard.press('Space');
      }
      step++;
      await sleep(90);
    }
  } finally {
    // Ensure no stuck keys (especially Space) so force-GO can't race a restart.
    for (const k of [...keys, 'Space', 'Enter']) {
      try {
        await page.keyboard.up(k);
      } catch (_) {}
    }
  }
  return step;
}

/** Read documentElement[data-rd-state] published by the WASM build. */
async function readQaState(page) {
  try {
    return await page.evaluate(() =>
      document.documentElement?.getAttribute('data-rd-state')
    );
  } catch (_) {
    return null;
  }
}

/** Poll until GameState label matches (or timeout). */
async function waitForQaState(page, want, timeoutMs = 15000) {
  const deadline = Date.now() + timeoutMs;
  let last = null;
  while (Date.now() < deadline) {
    last = await readQaState(page);
    if (last === want) return true;
    await sleep(150);
  }
  console.warn(`[qa] waitForQaState(${want}) timeout last=${last}`);
  return false;
}

/**
 * Boot ready = canvas present AND (boot hidden OR CTA visible for gesture).
 * Retries once with reload — secondary touch/mouse paths can hang when WASM
 * init races under CONCURRENCY≥2 (Target closed / 180s WaitTimeout).
 */
async function waitReady(page, { timeoutMs = 90000, reloads = 1 } = {}) {
  for (let attempt = 0; attempt <= reloads; attempt++) {
    try {
      await page.waitForSelector('canvas', { timeout: timeoutMs });
      await page.waitForFunction(
        () => {
          const boot = document.getElementById('boot');
          const cta = document.getElementById('boot-cta');
          if (boot?.classList.contains('hidden')) return true;
          if (!cta) return false;
          const disp = cta.style?.display || '';
          // CTA is shown as inline-block when WASM ready; also accept computed style.
          if (disp === 'inline-block' || disp === 'block') return true;
          try {
            const cs = window.getComputedStyle(cta);
            return cs.display !== 'none' && cs.visibility !== 'hidden';
          } catch (_) {
            return false;
          }
        },
        { timeout: timeoutMs }
      );
      return;
    } catch (e) {
      if (attempt >= reloads) throw e;
      console.warn(`[qa] waitReady attempt ${attempt + 1} failed; reload…`, String(e).slice(0, 120));
      try {
        await page.reload({ waitUntil: 'domcontentloaded', timeout: 120000 });
      } catch (_) {
        // fall through to next attempt / final throw
      }
    }
  }
}

async function dismissInstallIfAny(page) {
  await page.evaluate(() => {
    const el = document.getElementById('install');
    if (el) el.classList.add('hidden');
  });
}

async function focusCanvas(page) {
  await page.evaluate(() => {
    const c = document.querySelector('canvas');
    if (c) {
      c.tabIndex = 0;
      c.focus({ preventScroll: true });
    }
  });
  await sleep(80);
}

function center(format) {
  return { x: Math.floor(format.width / 2), y: Math.floor(format.height / 2) };
}

function stickDashPoints(format) {
  const w = format.width;
  const h = format.height;
  const portrait = h >= w;
  if (portrait) {
    return {
      stick: { x: Math.floor(w * 0.28), y: Math.floor(h * 0.83) },
      stick2: { x: Math.floor(w * 0.38), y: Math.floor(h * 0.78) },
      dash: { x: Math.floor(w * 0.75), y: Math.floor(h * 0.83) },
      start: { x: Math.floor(w / 2), y: Math.floor(h * 0.68) },
      swap: { x: Math.floor(w / 2), y: Math.floor(h * 0.88) },
      modeUp: { x: Math.floor(w / 2), y: Math.floor(h * 0.26) },
      modeDown: { x: Math.floor(w / 2), y: Math.floor(h * 0.4) },
      diffL: { x: Math.floor(w * 0.3), y: Math.floor(h * 0.52) },
      diffR: { x: Math.floor(w * 0.7), y: Math.floor(h * 0.52) },
      confirm: { x: Math.floor(w / 2), y: Math.floor(h * 0.45) },
    };
  }
  return {
    stick: { x: Math.floor(w * 0.1), y: Math.floor(h * 0.52) },
    stick2: { x: Math.floor(w * 0.14), y: Math.floor(h * 0.4) },
    dash: { x: Math.floor(w * 0.9), y: Math.floor(h * 0.52) },
    start: { x: Math.floor(w / 2), y: Math.floor(h * 0.68) },
    swap: { x: Math.floor(w / 2), y: Math.floor(h * 0.88) },
    modeUp: { x: Math.floor(w / 2), y: Math.floor(h * 0.26) },
    modeDown: { x: Math.floor(w / 2), y: Math.floor(h * 0.4) },
    diffL: { x: Math.floor(w * 0.3), y: Math.floor(h * 0.52) },
    diffR: { x: Math.floor(w * 0.7), y: Math.floor(h * 0.52) },
    confirm: { x: Math.floor(w / 2), y: Math.floor(h * 0.45) },
  };
}

/**
 * Primary path: video + matrix PNGs (one load per format).
 * Captures all 5 matrix screens with quality holds; records full journey + play.
 */
async function runPrimaryWithMatrix(format) {
  const tag = `${format.id}/primary`;
  const page = await newPage(format);
  const recPath = path.join(VID, `${format.id}_keyboard.webm`);
  let rec;
  try {
    page.__errors.length = 0;
    await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 180000 });
    await waitReady(page);
    rec = await startPageRecording(page, recPath);

    // --- 01 BOOT (CTA visible, settled) ---
    await qualityMatrixShot(page, format.id, '01_boot', 200);

    // Click CTA only — keyboard confirm also advances Bevy Menu → ModeSelect.
    const hidden = await dismissBootOverlay(page, format);
    if (hidden) pass(`${tag}: boot`);
    else fail(`${tag}: boot`);
    await dismissInstallIfAny(page);
    await focusCanvas(page);
    // Menu must paint after boot; never press Enter/Space before 02_menu shot.
    await sleep(1200);

    // If we somehow landed on ModeSelect, back out so 02_menu is true Menu.
    await page.keyboard.press('Escape');
    await sleep(400);

    // --- 02 MENU (true main menu: title / play CTA, not SELECT MODE) ---
    await qualityMatrixShot(page, format.id, '02_menu', 400);

    await page.keyboard.press('Enter');
    await sleep(1000);

    if (!MATRIX_ONLY) {
      for (let i = 0; i < MODES.length; i++) {
        await page.keyboard.press('ArrowDown');
        await sleep(160);
      }
      for (let i = 0; i < MODES.length; i++) {
        await page.keyboard.press('ArrowUp');
        await sleep(160);
      }
      pass(`${tag}: cycle all modes`, MODES.join(','));
      for (let i = 0; i < DIFFS.length; i++) {
        await page.keyboard.press('ArrowRight');
        await sleep(160);
      }
      for (let i = 0; i < DIFFS.length; i++) {
        await page.keyboard.press('ArrowLeft');
        await sleep(160);
      }
      pass(`${tag}: cycle all difficulties`, DIFFS.join(','));
    }

    // --- 03 MODE SELECT (settled on classic/normal after cycling home) ---
    await qualityMatrixShot(page, format.id, '03_mode_select', 200);

    await page.keyboard.press('Space'); // START → Playing
    // Wall-clock force-GO (qa_go_ms) starts on Enter Playing in WASM.
    const playStartedAt = Date.now();
    // Settle Playing + handheld stick/DASH chrome before matrix PNG.
    await sleep(1100);
    // Nudge mid-field so 04_playing is not a blank spawn frame.
    let step = await playSessionKeyboard(page, 700, { dash: true });
    await sleep(250);

    // --- 04 PLAYING (chrome mounted; continuous force-GO still far away) ---
    await qualityMatrixShot(page, format.id, '04_playing', 200);

    // One continuous play stretch for video (≥20s). Force-GO is delayed via
    // qa_go_ms so we do NOT restart mid-run (Space after GO was restarting and
    // poisoning 05 + truncating perceived play in reviews).
    const extraPlayMs = MATRIX_ONLY ? 400 : PLAY_MS;
    // Leave a small buffer before force-GO so last Space dashes can't land on GO.
    const safePlayMs = Math.max(0, Math.min(extraPlayMs, FORCE_GO_MS - 3500));
    step += await playSessionKeyboard(page, safePlayMs, { dash: true });
    // Stop dashing; wait for auto Game Over (no Space — would restart).
    // Generous margin: under C=3, rAF can lag; game uses wall-clock now, but
    // first frame after load may delay wall_start_ms slightly.
    const forceDeadline = playStartedAt + FORCE_GO_MS;
    const goWait = Math.max(forceDeadline - Date.now() + 8000, 10000);
    let gotGo = await waitForQaState(page, 'game_over', goWait);
    if (!gotGo) {
      // Extra wall wait — still no Space (would restart → Playing PNG).
      console.warn(`[qa] ${format.id}: primary go miss; extended wait`);
      gotGo = await waitForQaState(page, 'game_over', 12000);
    }
    pass(
      `${tag}: play move+dash`,
      `steps=${step} play_ms=${safePlayMs} go=${gotGo ? 'ok' : 'timeout'} force_ms=${FORCE_GO_MS}`
    );

    // --- 05 GAME OVER (require game_over state; never shoot Playing as GO) ---
    if (!gotGo) {
      // Last chance: poll a bit more before matrix PNG so reviews don't see Playing.
      gotGo = await waitForQaState(page, 'game_over', 6000);
    }
    const st = await readQaState(page);
    if (st !== 'game_over') {
      console.warn(`[qa] ${format.id}: 05 shot with state=${st} (wanted game_over)`);
    }
    await qualityMatrixShot(page, format.id, '05_game_over', 500);

    if (page.__errors.length === 0) pass(`${tag}: no panic`);
    else fail(`${tag}: panic`, page.__errors.join('; '));
  } catch (e) {
    fail(`${tag}: run`, e.stack || String(e));
  } finally {
    if (rec) {
      try {
        const info = await rec.stop();
        console.log('[rec]', recPath, info.frames, 'frames', info.bytes, 'bytes');
        if (info.bytes > 1000) pass(`${tag}: recording`, `${info.frames}f ${info.bytes}b`);
        else fail(`${tag}: recording`, info.error || 'empty video');
        await extractReviewStills(recPath, path.join(STILLS, `${format.id}_keyboard`), 6);
      } catch (e) {
        fail(`${tag}: recording encode`, String(e));
      }
    }
    await page.close();
  }
}

/**
 * Mouse secondary path. Retries once if play is truncated (CDP hang under
 * CONCURRENCY≥2 has produced steps=1 / ~3s encodes that still "passed").
 */
async function runMouseExhaustive(format) {
  if (MATRIX_ONLY) return;
  const tag = `${format.id}/mouse`;
  const recPath = path.join(VID, `${format.id}_mouse.webm`);
  // ~300ms/step nominal; require enough steps for ≥20s continuous play gate.
  const minSteps = Math.max(20, Math.floor(PLAY_MS / 550)); // high-res mouse loops ~37–40 steps/20s
  // ~12fps screencast: menu ~4s + play; require enough frames for real play length.
  const minFrames = Math.max(80, Math.floor((PLAY_MS / 1000) * 6));
  let modesPassed = false;

  for (let attempt = 1; attempt <= 2; attempt++) {
    const page = await newPage(format);
    const { x: cx, y: cy } = center(format);
    let rec;
    let step = 0;
    let playElapsed = 0;
    let runErr = null;
    let recInfo = null;
    let pagePanics = [];
    try {
      page.__errors.length = 0;
      await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 180000 });
      await waitReady(page, { timeoutMs: 90000, reloads: 1 });
      rec = await startPageRecording(page, recPath);
      await page.mouse.click(cx, cy);
      await sleep(500);
      await dismissInstallIfAny(page);
      await page.mouse.click(cx, cy);
      await sleep(800);
      const w = format.width;
      const h = format.height;
      await page.mouse.click(Math.floor(w / 2), Math.floor(h * 0.38));
      await sleep(200);
      await page.mouse.click(Math.floor(w * 0.88), Math.floor(h * 0.5));
      await sleep(200);
      if (!modesPassed) {
        pass(`${tag}: mode+difficulty clicks`);
        modesPassed = true;
      }
      await page.mouse.click(cx, cy);
      await sleep(1500);
      // Full PLAY_MS for secondary path (skill: ≥20s play on every path).
      const playStarted = Date.now();
      const end = playStarted + PLAY_MS;
      while (Date.now() < end) {
        const x0 = Math.floor(w * (0.3 + 0.4 * ((step % 5) / 5)));
        const y0 = Math.floor(h * (0.35 + 0.3 * (((step + 2) % 5) / 5)));
        // Drag with left only; right-click dash after release (avoids CDP stall when
        // right-click is nested while left button is still held under C≥2).
        await page.mouse.move(x0, y0);
        await page.mouse.down({ button: 'left' });
        await page.mouse.move(x0 + 40, y0 - 30, { steps: 4 });
        await sleep(180);
        await page.mouse.up({ button: 'left' });
        if (step % 2 === 0) {
          await page.mouse.click(x0 + 40, y0 - 30, { button: 'right' });
        }
        step++;
        await sleep(120);
      }
      playElapsed = Date.now() - playStarted;
      pagePanics = [...(page.__errors || [])];
    } catch (e) {
      runErr = e.stack || String(e);
      try {
        pagePanics = [...(page.__errors || [])];
      } catch (_) {}
    } finally {
      if (rec) {
        try {
          recInfo = await rec.stop();
          await extractReviewStills(recPath, path.join(STILLS, `${format.id}_mouse`), 6);
        } catch (e) {
          runErr = runErr || String(e);
        }
      }
      try {
        await page.close();
      } catch (_) {}
    }

    const shortPlay = step < minSteps || playElapsed < PLAY_MS * 0.85;
    const frames = recInfo?.frames ?? 0;
    const bytes = recInfo?.bytes ?? 0;
    const shortRec = bytes < 1000 || frames < minFrames;
    const canRetry = attempt < 2 && (shortPlay || shortRec || runErr);

    if (canRetry) {
      console.warn(
        `[qa] ${tag}: attempt ${attempt} weak (steps=${step} elapsed_ms=${playElapsed} frames=${frames} err=${runErr ? String(runErr).slice(0, 80) : 'none'}); retry`
      );
      continue;
    }

    // Final attempt results (or good first attempt).
    if (runErr && (shortPlay || shortRec)) {
      fail(`${tag}: run`, runErr);
    } else if (shortPlay) {
      fail(
        `${tag}: play drag+right-dash`,
        `steps=${step} elapsed_ms=${playElapsed} (need ≥${minSteps} steps / ~${PLAY_MS}ms)`
      );
    } else {
      pass(
        `${tag}: play drag+right-dash`,
        `steps=${step} elapsed_ms=${playElapsed}${attempt > 1 ? ' retry' : ''}`
      );
    }
    if (!shortPlay) {
      if (pagePanics.length === 0) pass(`${tag}: no panic`);
      else fail(`${tag}: panic`, pagePanics.join('; '));
    }
    if (bytes > 1000 && frames >= minFrames) {
      pass(`${tag}: recording`, `${frames}f`);
    } else if (bytes > 1000) {
      fail(`${tag}: recording`, `${frames}f too short (min ${minFrames})`);
    } else {
      fail(`${tag}: recording`, recInfo?.error || 'empty');
    }
    break;
  }
}

async function runTouchExhaustive(format) {
  if (MATRIX_ONLY) return;
  const tag = `${format.id}/touch`;
  const page = await newPage(format);
  const client = page.__cdp || (await page.createCDPSession());
  if (!page.__cdp) {
    await client.send('Emulation.setTouchEmulationEnabled', {
      enabled: true,
      maxTouchPoints: 5,
    });
  }
  const pts = stickDashPoints(format);
  const recPath = path.join(VID, `${format.id}_touch.webm`);
  let rec;
  try {
    page.__errors.length = 0;
    // Secondary path under concurrency: allow reload if WASM boot stalls.
    await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 180000 });
    await waitReady(page, { timeoutMs: 90000, reloads: 1 });
    rec = await startPageRecording(page, recPath);
    await page.touchscreen.tap(pts.confirm.x, pts.confirm.y);
    await sleep(500);
    await dismissInstallIfAny(page);
    const bootHidden = await page.evaluate(() =>
      document.getElementById('boot')?.classList.contains('hidden')
    );
    if (bootHidden) pass(`${tag}: boot`);
    else fail(`${tag}: boot`);
    await page.touchscreen.tap(pts.swap.x, pts.swap.y);
    await sleep(300);
    await page.touchscreen.tap(pts.swap.x, pts.swap.y);
    await sleep(250);
    pass(`${tag}: swap controls toggle`);
    await page.touchscreen.tap(pts.confirm.x, pts.confirm.y);
    await sleep(800);
    for (let i = 0; i < MODES.length; i++) {
      await page.touchscreen.tap(pts.modeDown.x, pts.modeDown.y);
      await sleep(180);
    }
    pass(`${tag}: cycle modes`);
    for (let i = 0; i < DIFFS.length; i++) {
      await page.touchscreen.tap(pts.diffR.x, pts.diffR.y);
      await sleep(180);
    }
    pass(`${tag}: cycle difficulties`);

    // I-NO-TWO-FINGER-GESTURE / SIM-NO-TWO-FINGER-BACK: free two-finger mid-panel
    // must NOT leave mode_select (no timed multi-touch navigation).
    await waitForQaState(page, 'mode_select', 5000);
    {
      const w = format.width || 390;
      const h = format.height || 844;
      const cx = w * 0.5;
      const cy = h * 0.35; // mode-list band, not bottom strip / left edge
      await client.send('Input.dispatchTouchEvent', {
        type: 'touchStart',
        touchPoints: [
          { x: cx - 24, y: cy, id: 1 },
          { x: cx + 24, y: cy, id: 2 },
        ],
      });
      await sleep(120);
      await client.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
      await sleep(450);
      const st = await readQaState(page);
      if (st === 'mode_select') {
        pass(`${tag}: free two-finger does not back`, `state=${st}`);
      } else {
        fail(
          `${tag}: free two-finger does not back`,
          `expected mode_select after free multi-touch, got ${st}`
        );
      }
    }

    await page.touchscreen.tap(pts.start.x, pts.start.y);
    await sleep(1500);
    // Full PLAY_MS for secondary path (skill: ≥20s play on every path).
    // Stick + DASH multi-touch is OK only on dedicated chrome hit targets.
    const end = Date.now() + PLAY_MS;
    let step = 0;
    while (Date.now() < end) {
      const dx = (step % 2 === 0 ? 1 : -1) * 20;
      const dy = (step % 3 === 0 ? -1 : 1) * 15;
      await client.send('Input.dispatchTouchEvent', {
        type: 'touchStart',
        touchPoints: [{ x: pts.stick.x, y: pts.stick.y, id: 1 }],
      });
      await sleep(40);
      await client.send('Input.dispatchTouchEvent', {
        type: 'touchMove',
        touchPoints: [{ x: pts.stick.x + dx, y: pts.stick.y + dy, id: 1 }],
      });
      await sleep(180);
      if (step % 2 === 0) {
        await client.send('Input.dispatchTouchEvent', {
          type: 'touchStart',
          touchPoints: [
            { x: pts.stick.x + dx, y: pts.stick.y + dy, id: 1 },
            { x: pts.dash.x, y: pts.dash.y, id: 2 },
          ],
        });
        await sleep(80);
      }
      await client.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
      step++;
      await sleep(120);
    }
    pass(`${tag}: play stick+dash`, `steps=${step}`);
    if (page.__errors.length === 0) pass(`${tag}: no panic`);
    else fail(`${tag}: panic`, page.__errors.join('; '));
  } catch (e) {
    fail(`${tag}: run`, e.stack || String(e));
  } finally {
    if (rec) {
      try {
        const info = await rec.stop();
        if (info.bytes > 1000) pass(`${tag}: recording`, `${info.frames}f`);
        else fail(`${tag}: recording`, info.error || 'empty');
        await extractReviewStills(recPath, path.join(STILLS, `${format.id}_touch`), 6);
      } catch (e) {
        fail(`${tag}: recording encode`, String(e));
      }
    }
    await page.close();
  }
}

async function runFormat(format) {
  console.log(
    '\n==== format',
    format.id,
    isHandheldFormat(format) ? '[device-emulation]' : '[desktop]',
    '===='
  );
  await runPrimaryWithMatrix(format);
  if (!format.touch) await runMouseExhaustive(format);
  else await runTouchExhaustive(format);
}

/** Run up to `limit` async workers over items. */
async function mapPool(items, limit, fn) {
  const ret = [];
  let i = 0;
  async function worker() {
    while (i < items.length) {
      const idx = i++;
      ret[idx] = await fn(items[idx], idx);
    }
  }
  await Promise.all(Array.from({ length: Math.min(limit, items.length) }, () => worker()));
  return ret;
}

const only = (process.env.E2E_FORMATS || '')
  .split(',')
  .map((s) => s.trim())
  .filter(Boolean);
const formats = only.length
  ? MATRIX.formats.filter((f) => only.includes(f.id))
  : MATRIX.formats;

await mapPool(formats, CONCURRENCY, (format) => runFormat(format));

await browser.close();

// Verify matrix completeness when capturing
const missing = [];
if (CAPTURE_MATRIX) {
  for (const format of formats) {
    for (const screen of MATRIX.screens) {
      const name = `${format.id}_${screen.shot_suffix}.png`;
      const file = path.join(MATRIX_OUT, name);
      if (!fs.existsSync(file) || fs.statSync(file).size < 500) missing.push(name);
    }
  }
  fs.writeFileSync(
    path.join(MATRIX_OUT, 'matrix_results.json'),
    JSON.stringify(
      {
        matrix: MATRIX,
        cells: formats.length * MATRIX.screens.length,
        expected_cells: MATRIX.expected_cells,
        missing,
        source: 'e2e_inputs.mjs unified capture',
        at: new Date().toISOString(),
      },
      null,
      2
    )
  );
  if (missing.length) {
    console.error('MATRIX MISSING:', missing);
    fail('matrix complete', missing.join(', '));
  } else {
    pass('matrix complete', `${formats.length * MATRIX.screens.length} cells`);
  }
}

const failed = results.filter((r) => !r.ok);
const recordings = fs.existsSync(VID)
  ? fs.readdirSync(VID).filter((f) => f.endsWith('.webm') || f.endsWith('.mp4'))
  : [];
fs.writeFileSync(
  path.join(OUT, 'results.json'),
  JSON.stringify(
    {
      matrix_formats: formats.map((f) => f.id),
      play_ms: PLAY_MS,
      capture_matrix: CAPTURE_MATRIX,
      concurrency: CONCURRENCY,
      modes: MODES,
      difficulties: DIFFS,
      results,
      failed: failed.length,
      recordings,
      recordings_dir: VID,
      stills_dir: STILLS,
      matrix_out: CAPTURE_MATRIX ? MATRIX_OUT : null,
      matrix_missing: missing,
      at: new Date().toISOString(),
      layer: 'capture_only',
      note:
        'CAPTURE only (ok/CAPTURE_OK = automation step succeeded). NOT visual review. A4b/A6 critiques + A7 pre-prod are separate (video_critique vs matrix_critique; qa_success_criteria.json).',
    },
    null,
    2
  )
);
console.log('\n=== E2E CAPTURE SUMMARY (not visual review) ===');
console.log('capture_ok', results.filter((r) => r.ok).length, '/', results.length);
console.log('recordings', recordings.length, 'in', VID);
if (CAPTURE_MATRIX) console.log('matrix missing', missing.length);
console.log(
  'NOTE: CAPTURE_OK ≠ looks good. Run A4b/A6 reviews before A7; suite exit 0 is not PRE-PROD PASS.'
);
if (failed.length) {
  console.error('CAPTURE_FAILED:', failed);
  process.exit(1);
}
process.exit(0);
