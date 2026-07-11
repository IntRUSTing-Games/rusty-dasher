/**
 * Exhaustive E2E for every format in qa_matrix.json.
 *
 * This game is small — every run MUST cover the full product surface:
 *   all screens, all modes, all difficulties, all primary inputs,
 *   swap controls (handheld), and ≥20s of actual play with movement+dash.
 *
 * Artifacts are primarily VIDEO recordings (catch transient bugs), not stills.
 * Review stills are extracted from each video for quick agent visual scan.
 *
 * Desktop formats: keyboard + mouse paths
 * Handheld formats: device-emulation touch path (+ keyboard smoke)
 *
 * Real USB phone: scripts/e2e_phone.mjs (adb screenrecord + CDP)
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
const MATRIX = JSON.parse(fs.readFileSync(path.join(__dirname, 'qa_matrix.json'), 'utf8'));
const URL = 'http://127.0.0.1:8080/?e2e=1';
const PLAY_MS = Number(process.env.E2E_PLAY_MS || 20000);

// Full product surface (must all be exercised)
const MODES = ['CLASSIC', 'ZEN', 'SURVIVAL', 'TIMED']; // 4 mode steps via menu_down
const DIFFS = ['EASY', 'NORMAL', 'HARD', 'INSANE']; // cycle left/right

fs.mkdirSync(OUT, { recursive: true });
fs.mkdirSync(VID, { recursive: true });
fs.mkdirSync(STILLS, { recursive: true });

const results = [];
function pass(name, detail = '') {
  results.push({ name, ok: true, detail });
  console.log('PASS', name, detail);
}
function fail(name, detail = '') {
  results.push({ name, ok: false, detail });
  console.error('FAIL', name, detail);
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

logChromeGlMode();
const browser = await puppeteer.launch({
  executablePath: chromeExecutable(),
  headless: 'new',
  args: chromeGpuArgs(),
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

async function waitReady(page) {
  await page.waitForSelector('canvas', { timeout: 180000 });
  await page.waitForFunction(
    () =>
      document.getElementById('boot')?.classList.contains('hidden') ||
      document.getElementById('boot-cta')?.style?.display === 'inline-block',
    { timeout: 180000 }
  );
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

/** Exhaustive keyboard path: every mode, every difficulty, play 20s, game over/back. */
async function runKeyboardExhaustive(format) {
  const tag = `${format.id}/keyboard`;
  const page = await newPage(format);
  const recPath = path.join(VID, `${format.id}_keyboard.webm`);
  let rec;
  try {
    page.__errors.length = 0;
    await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 180000 });
    await waitReady(page);
    rec = await startPageRecording(page, recPath);

    // BOOT
    await page.keyboard.press('Enter');
    await sleep(400);
    let hidden = await page.evaluate(() =>
      document.getElementById('boot')?.classList.contains('hidden')
    );
    if (!hidden) await page.keyboard.press('Space');
    await sleep(300);
    hidden = await page.evaluate(() =>
      document.getElementById('boot')?.classList.contains('hidden')
    );
    if (hidden) pass(`${tag}: boot`);
    else fail(`${tag}: boot`);
    await dismissInstallIfAny(page);
    await focusCanvas(page);

    // MENU → mode select
    await page.keyboard.press('Enter');
    await sleep(700);

    // Cycle ALL modes (4)
    for (let i = 0; i < MODES.length; i++) {
      await page.keyboard.press('ArrowDown');
      await sleep(180);
    }
    for (let i = 0; i < MODES.length; i++) {
      await page.keyboard.press('ArrowUp');
      await sleep(180);
    }
    pass(`${tag}: cycle all modes`, MODES.join(','));

    // Cycle ALL difficulties (4)
    for (let i = 0; i < DIFFS.length; i++) {
      await page.keyboard.press('ArrowRight');
      await sleep(180);
    }
    for (let i = 0; i < DIFFS.length; i++) {
      await page.keyboard.press('ArrowLeft');
      await sleep(180);
    }
    pass(`${tag}: cycle all difficulties`, DIFFS.join(','));

    // Back to menu, re-enter (back key)
    await page.keyboard.press('Escape');
    await sleep(500);
    await page.keyboard.press('Enter');
    await sleep(600);

    // Start Classic/Normal play
    await page.keyboard.press('Space');
    await sleep(1200);

    // PLAY ≥20s with WASD + arrows + Space dash
    const end = Date.now() + PLAY_MS;
    let step = 0;
    while (Date.now() < end) {
      const keys = ['KeyW', 'KeyA', 'KeyS', 'KeyD', 'ArrowUp', 'ArrowLeft', 'ArrowDown', 'ArrowRight'];
      const k = keys[step % keys.length];
      await page.keyboard.down(k);
      await sleep(280);
      await page.keyboard.up(k);
      if (step % 3 === 0) {
        await page.keyboard.press('Space'); // dash
      }
      step++;
      await sleep(120);
    }
    pass(`${tag}: play ${PLAY_MS}ms move+dash`, `steps=${step}`);

    // Esc → menu
    await page.keyboard.press('Escape');
    await sleep(800);
    if (page.__errors.length === 0) pass(`${tag}: esc menu / no panic`);
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
        const stillDir = path.join(STILLS, `${format.id}_keyboard`);
        await extractReviewStills(recPath, stillDir, 6);
      } catch (e) {
        fail(`${tag}: recording encode`, String(e));
      }
    }
    await page.close();
  }
}

/** Exhaustive mouse path (desktop only). */
async function runMouseExhaustive(format) {
  const tag = `${format.id}/mouse`;
  const page = await newPage(format);
  const { x: cx, y: cy } = center(format);
  const recPath = path.join(VID, `${format.id}_mouse.webm`);
  let rec;
  try {
    page.__errors.length = 0;
    await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 180000 });
    await waitReady(page);
    rec = await startPageRecording(page, recPath);

    await page.mouse.click(cx, cy);
    await sleep(500);
    await dismissInstallIfAny(page);

    // Menu confirm
    await page.mouse.click(cx, cy);
    await sleep(800);

    // Mode thirds / sides for difficulty
    const w = format.width;
    const h = format.height;
    // mode down (lower third of mode list band)
    await page.mouse.click(Math.floor(w / 2), Math.floor(h * 0.38));
    await sleep(200);
    await page.mouse.click(Math.floor(w / 2), Math.floor(h * 0.28));
    await sleep(200);
    // difficulty sides
    await page.mouse.click(Math.floor(w * 0.12), Math.floor(h * 0.5));
    await sleep(200);
    await page.mouse.click(Math.floor(w * 0.88), Math.floor(h * 0.5));
    await sleep(200);
    pass(`${tag}: mode+difficulty clicks`);

    // Start
    await page.mouse.click(cx, cy);
    await sleep(1500);

    // Play 20s point-to-move + right-click dash
    const end = Date.now() + PLAY_MS;
    let step = 0;
    while (Date.now() < end) {
      const x0 = Math.floor(w * (0.3 + 0.4 * ((step % 5) / 5)));
      const y0 = Math.floor(h * (0.35 + 0.3 * (((step + 2) % 5) / 5)));
      await page.mouse.move(x0, y0);
      await page.mouse.down();
      await page.mouse.move(x0 + 40, y0 - 30, { steps: 4 });
      await sleep(200);
      if (step % 2 === 0) {
        await page.mouse.click(x0 + 40, y0 - 30, { button: 'right' });
      }
      await page.mouse.up();
      step++;
      await sleep(150);
    }
    pass(`${tag}: play ${PLAY_MS}ms drag+right-dash`, `steps=${step}`);
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
        await extractReviewStills(recPath, path.join(STILLS, `${format.id}_mouse`), 6);
      } catch (e) {
        fail(`${tag}: recording encode`, String(e));
      }
    }
    await page.close();
  }
}

/** Exhaustive touch path (handheld): all bands, swap, all modes/diffs, stick+dash, 20s play. */
async function runTouchExhaustive(format) {
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
    await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 180000 });
    await waitReady(page);
    rec = await startPageRecording(page, recPath);

    // BOOT
    await page.touchscreen.tap(pts.confirm.x, pts.confirm.y);
    await sleep(500);
    await dismissInstallIfAny(page);
    const bootHidden = await page.evaluate(() =>
      document.getElementById('boot')?.classList.contains('hidden')
    );
    if (bootHidden) pass(`${tag}: boot`);
    else fail(`${tag}: boot`);

    // MENU swap band then confirm
    await page.touchscreen.tap(pts.swap.x, pts.swap.y);
    await sleep(400);
    await page.touchscreen.tap(pts.swap.x, pts.swap.y); // toggle back
    await sleep(300);
    pass(`${tag}: swap controls toggle`);
    await page.touchscreen.tap(pts.confirm.x, pts.confirm.y);
    await sleep(800);

    // MODE: cycle all modes via mode down
    for (let i = 0; i < MODES.length; i++) {
      await page.touchscreen.tap(pts.modeDown.x, pts.modeDown.y);
      await sleep(200);
    }
    for (let i = 0; i < 2; i++) {
      await page.touchscreen.tap(pts.modeUp.x, pts.modeUp.y);
      await sleep(200);
    }
    pass(`${tag}: cycle modes`);

    // DIFF all
    for (let i = 0; i < DIFFS.length; i++) {
      await page.touchscreen.tap(pts.diffR.x, pts.diffR.y);
      await sleep(200);
    }
    for (let i = 0; i < DIFFS.length; i++) {
      await page.touchscreen.tap(pts.diffL.x, pts.diffL.y);
      await sleep(200);
    }
    pass(`${tag}: cycle difficulties`);

    // START
    await page.touchscreen.tap(pts.start.x, pts.start.y);
    await sleep(1500);

    // PLAY ≥20s stick + dash multi-touch
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
      await sleep(200);
      if (step % 2 === 0) {
        await client.send('Input.dispatchTouchEvent', {
          type: 'touchStart',
          touchPoints: [
            { x: pts.stick.x + dx, y: pts.stick.y + dy, id: 1 },
            { x: pts.dash.x, y: pts.dash.y, id: 2 },
          ],
        });
        await sleep(100);
      }
      await client.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
      step++;
      await sleep(150);
    }
    pass(`${tag}: play ${PLAY_MS}ms stick+dash`, `steps=${step}`);
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

// Optional filter: E2E_FORMATS=phone_rodin_chrome,laptop_hd
const only = (process.env.E2E_FORMATS || '')
  .split(',')
  .map((s) => s.trim())
  .filter(Boolean);
const formats = only.length
  ? MATRIX.formats.filter((f) => only.includes(f.id))
  : MATRIX.formats;

for (const format of formats) {
  console.log('\n==== format', format.id, isHandheldFormat(format) ? '[device-emulation]' : '[desktop]', '====');
  // Always keyboard exhaustive (keys must work on all sizes)
  await runKeyboardExhaustive(format);
  if (!format.touch) {
    await runMouseExhaustive(format);
  } else {
    await runTouchExhaustive(format);
  }
}

await browser.close();

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
      modes: MODES,
      difficulties: DIFFS,
      results,
      failed: failed.length,
      recordings,
      recordings_dir: VID,
      stills_dir: STILLS,
      at: new Date().toISOString(),
      note: 'Primary e2e artifact is VIDEO under recordings/. Stills are review extracts only.',
    },
    null,
    2
  )
);
console.log('\n=== E2E SUMMARY ===');
console.log('passed', results.filter((r) => r.ok).length, '/', results.length);
console.log('recordings', recordings.length, 'in', VID);
if (failed.length) {
  console.error('FAILED:', failed);
  process.exit(1);
}
process.exit(0);
