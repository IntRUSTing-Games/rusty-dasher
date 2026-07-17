/**
 * Layout matrix PNGs — preferred production is UNIFIED with e2e:
 *   CAPTURE_MATRIX=1 node scripts/e2e_inputs.mjs
 * which quality-holds at each screen while recording video (no second full walk).
 *
 * This script is a FALLBACK / verify-only path when matrix cells are missing
 * after e2e, or when CAPTURE_MATRIX=0 was used.
 *
 * Env:
 *   VERIFY_ONLY=1   only check expected files exist (no browser)
 *   CONCURRENCY=3   parallel formats when capturing
 */
import puppeteer from 'puppeteer-core';
import { chromeExecutable, chromeGpuArgs, logChromeGlMode } from './chrome_launch.mjs';
import { applyDeviceEmulation, isHandheldFormat } from './device_emulation.mjs';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, '..');
const OUT = path.join(ROOT, 'screenshots/viewports');
const MATRIX = JSON.parse(fs.readFileSync(path.join(__dirname, 'qa_matrix.json'), 'utf8'));
const PORT = process.env.PORT || process.env.RUSTY_PORT || '17880';
const BASE = (process.env.E2E_URL || `http://127.0.0.1:${PORT}/`).replace(/\/?$/, '/');
const URL = `${BASE}?qa_matrix=1`;
const VERIFY_ONLY = process.env.VERIFY_ONLY === '1';
const CONCURRENCY = Math.max(1, Number(process.env.CONCURRENCY || 3));
const HOLD_MS = Number(process.env.MATRIX_HOLD_MS || 450);

fs.mkdirSync(OUT, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function missingCells(formats = MATRIX.formats) {
  const missing = [];
  for (const format of formats) {
    for (const screen of MATRIX.screens) {
      const name = `${format.id}_${screen.shot_suffix}.png`;
      const file = path.join(OUT, name);
      if (!fs.existsSync(file) || fs.statSync(file).size < 500) missing.push(name);
    }
  }
  return missing;
}

function writeManifest(missing) {
  const manifest = {
    matrix: MATRIX,
    cells: MATRIX.formats.length * MATRIX.screens.length,
    expected_cells: MATRIX.expected_cells,
    missing,
    out: OUT,
    at: new Date().toISOString(),
  };
  fs.writeFileSync(path.join(OUT, 'matrix_results.json'), JSON.stringify(manifest, null, 2));
  return manifest;
}

if (VERIFY_ONLY) {
  const missing = missingCells();
  writeManifest(missing);
  console.log('=== VIEWPORT VERIFY ONLY ===');
  console.log('expected', MATRIX.expected_cells, 'missing', missing.length);
  if (missing.length) {
    console.error('MISSING:', missing);
    process.exit(1);
  }
  process.exit(0);
}

// If already complete from unified e2e, skip re-walk
const preMissing = missingCells();
if (preMissing.length === 0) {
  writeManifest([]);
  console.log('=== VIEWPORT MATRIX ===');
  console.log('all', MATRIX.expected_cells, 'cells already present (from unified e2e) — skip capture');
  process.exit(0);
}
console.log('[viewport] missing', preMissing.length, '— capturing only incomplete formats');

const needFormats = MATRIX.formats.filter((f) =>
  MATRIX.screens.some((s) => {
    const file = path.join(OUT, `${f.id}_${s.shot_suffix}.png`);
    return !fs.existsSync(file) || fs.statSync(file).size < 500;
  })
);

async function shot(page, name) {
  await sleep(HOLD_MS);
  await sleep(80);
  const file = path.join(OUT, name + '.png');
  await page.screenshot({ path: file, type: 'png' });
  console.log('saved', name, fs.statSync(file).size);
}

async function waitBootReady(page) {
  await page.waitForSelector('canvas', { timeout: 180000 });
  await page.waitForFunction(
    () => document.getElementById('boot-cta')?.style?.display === 'inline-block',
    { timeout: 180000 }
  );
}

async function captureFormat(browser, format) {
  const page = await browser.newPage();
  try {
    if (isHandheldFormat(format)) await applyDeviceEmulation(page, format);
    else {
      await page.setViewport({
        width: format.width,
        height: format.height,
        deviceScaleFactor: format.dpr,
        isMobile: false,
        hasTouch: false,
      });
    }
    console.log('=== format', format.id, `${format.width}x${format.height}`);
    await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 180000 });
    await waitBootReady(page);
    await shot(page, `${format.id}_01_boot`);
    try {
      await page.click('#boot-cta');
    } catch {
      const cx = Math.floor(format.width / 2);
      const cy = Math.floor(format.height / 2);
      if (format.touch) await page.touchscreen.tap(cx, cy);
      else await page.mouse.click(cx, cy);
    }
    await sleep(900);
    await page.evaluate(() => {
      document.getElementById('install')?.classList.add('hidden');
      const c = document.querySelector('canvas');
      if (c) {
        c.tabIndex = 0;
        c.focus({ preventScroll: true });
      }
    });
    await shot(page, `${format.id}_02_menu`);
    const cx = Math.floor(format.width / 2);
    const cy = Math.floor(format.height / 2);
    if (format.touch) await page.touchscreen.tap(cx, cy);
    else await page.keyboard.press('Enter');
    await sleep(1100);
    await shot(page, `${format.id}_03_mode_select`);
    if (format.touch) await page.touchscreen.tap(cx, Math.floor(format.height * 0.68));
    else await page.keyboard.press('Space');
    await sleep(1500);
    await shot(page, `${format.id}_04_playing`);
    await sleep(2800);
    await shot(page, `${format.id}_05_game_over`);
  } finally {
    await page.close();
  }
}

logChromeGlMode();
const browser = await puppeteer.launch({
  executablePath: chromeExecutable(),
  headless: 'new',
  args: chromeGpuArgs(),
});

async function mapPool(items, limit, fn) {
  let i = 0;
  async function worker() {
    while (i < items.length) {
      const idx = i++;
      await fn(items[idx]);
    }
  }
  await Promise.all(Array.from({ length: Math.min(limit, items.length) }, () => worker()));
}

try {
  await mapPool(needFormats, CONCURRENCY, (f) => captureFormat(browser, f));
} finally {
  await browser.close();
}

const missing = missingCells();
writeManifest(missing);
console.log('\n=== VIEWPORT MATRIX ===');
console.log('expected', MATRIX.expected_cells, 'missing', missing.length);
if (missing.length) {
  console.error('MISSING:', missing);
  process.exit(1);
}
console.log('all', MATRIX.expected_cells, 'cells present');
process.exit(0);
