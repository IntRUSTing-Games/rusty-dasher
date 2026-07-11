/**
 * Full visual matrix: every SCREEN × every FORMAT (see scripts/qa_matrix.json).
 * Produces screenshots/viewports/{format}_{shot_suffix}.png for every cell in qa_matrix.json.
 *
 * Uses ?qa_matrix=1 so Game Over can be forced after a brief play (reliable).
 * Requires dist served at http://127.0.0.1:8080/
 */
import puppeteer from 'puppeteer-core';
import { chromeExecutable, chromeGpuArgs, logChromeGlMode } from './chrome_launch.mjs';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, '..');
const OUT = path.join(ROOT, 'screenshots/viewports');
const MATRIX = JSON.parse(fs.readFileSync(path.join(__dirname, 'qa_matrix.json'), 'utf8'));
const URL = 'http://127.0.0.1:8080/?qa_matrix=1';

fs.mkdirSync(OUT, { recursive: true });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function shot(page, name) {
  const file = path.join(OUT, name + '.png');
  await page.screenshot({ path: file, fullPage: false });
  console.log('saved', name, fs.statSync(file).size);
  return file;
}

async function waitBootReady(page) {
  await page.waitForSelector('canvas', { timeout: 180000 });
  await page.waitForFunction(
    () => document.getElementById('boot-cta')?.style?.display === 'inline-block',
    { timeout: 180000 }
  );
}

async function dismissBoot(page, vp) {
  const cx = Math.floor(vp.width / 2);
  const cy = Math.floor(vp.height / 2);
  // Prefer CTA click so we don't double-advance Bevy menus with Enter.
  try {
    await page.click('#boot-cta');
  } catch {
    if (vp.touch) await page.touchscreen.tap(cx, cy);
    else await page.mouse.click(cx, cy);
  }
  await sleep(700);
  await page.evaluate(() => {
    const c = document.querySelector('canvas');
    if (c) {
      c.tabIndex = 0;
      c.focus({ preventScroll: true });
    }
  });
  await sleep(500);
}

async function captureFormat(browser, format) {
  const page = await browser.newPage();
  await page.setViewport({
    width: format.width,
    height: format.height,
    deviceScaleFactor: format.dpr,
    isMobile: format.touch,
    hasTouch: format.touch,
  });
  if (format.touch) {
    const client = await page.createCDPSession();
    await client.send('Emulation.setTouchEmulationEnabled', {
      enabled: true,
      maxTouchPoints: 2,
    });
  }

  console.log('=== format', format.id, `${format.width}x${format.height}`);
  await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 180000 });

  // 01 boot (ready state — CTA visible)
  await waitBootReady(page);
  await sleep(400);
  await shot(page, `${format.id}_01_boot`);

  await dismissBoot(page, format);
  await sleep(1000);
  // 02 menu
  await shot(page, `${format.id}_02_menu`);

  // 03 mode select
  const cx = Math.floor(format.width / 2);
  const cy = Math.floor(format.height / 2);
  if (format.touch) {
    await page.touchscreen.tap(cx, cy);
  } else {
    await page.keyboard.press('Enter');
  }
  await sleep(1200);
  await shot(page, `${format.id}_03_mode_select`);

  // 04 playing
  if (format.touch) {
    await page.touchscreen.tap(cx, cy);
  } else {
    await page.keyboard.press('Space');
  }
  await sleep(1600);
  await shot(page, `${format.id}_04_playing`);

  // 05 game over (qa_matrix=1 forces after ~2.2s of play)
  await sleep(2800);
  await shot(page, `${format.id}_05_game_over`);

  await page.close();
}

logChromeGlMode();
const browser = await puppeteer.launch({
  executablePath: chromeExecutable(),
  headless: 'new',
  args: chromeGpuArgs(),
});

// Always tear down Chromium on exit/signals so a killed agent run does not leave
// headless chrome eating CPU (seen with orphaned puppeteer_dev_chrome_profile-*).
async function shutdownBrowser(code = 0) {
  try {
    if (browser && browser.connected !== false) {
      await browser.close();
    }
  } catch (_) {}
  // hard-exit if close hangs
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


const missing = [];
try {
  for (const format of MATRIX.formats) {
    await captureFormat(browser, format);
  }

  // Verify all expected matrix cells exist
  for (const format of MATRIX.formats) {
    for (const screen of MATRIX.screens) {
      const name = `${format.id}_${screen.shot_suffix}.png`;
      const file = path.join(OUT, name);
      if (!fs.existsSync(file) || fs.statSync(file).size < 500) {
        missing.push(name);
      }
    }
  }
} finally {
  await browser.close();
}

const manifest = {
  matrix: MATRIX,
  cells: MATRIX.formats.length * MATRIX.screens.length,
  missing,
  out: OUT,
  at: new Date().toISOString(),
};
fs.writeFileSync(path.join(OUT, 'matrix_results.json'), JSON.stringify(manifest, null, 2));

console.log('\n=== VIEWPORT MATRIX ===');
console.log('expected', MATRIX.expected_cells, 'missing', missing.length);
if (missing.length) {
  console.error('MISSING:', missing);
  process.exit(1);
}
console.log('all', MATRIX.expected_cells, 'cells present');
process.exit(0);
