/**
 * E2E input matrix: keyboard / mouse / touch on EVERY format in qa_matrix.json.
 * Exit 0 only if all format×input paths that apply pass.
 *
 * Desktop formats (touch:false): keyboard + mouse
 * Handheld formats (touch:true): touch (+ keyboard smoke still runs where useful)
 */
import puppeteer from 'puppeteer-core';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, '..');
const OUT = path.join(ROOT, 'screenshots/web/e2e');
const MATRIX = JSON.parse(fs.readFileSync(path.join(__dirname, 'qa_matrix.json'), 'utf8'));
const URL = 'http://127.0.0.1:8080/';

fs.mkdirSync(OUT, { recursive: true });

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

async function shot(page, name) {
  await page.screenshot({ path: path.join(OUT, name + '.png') });
}

const browser = await puppeteer.launch({
  executablePath: '/home/viny/bin/google-chrome',
  headless: 'new',
  args: [
    '--no-sandbox',
    '--disable-setuid-sandbox',
    '--enable-unsafe-swiftshader',
    '--use-gl=angle',
    '--use-angle=swiftshader-webgl',
    '--window-size=1920,1080',
    '--force-device-scale-factor=1',
  ],
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
  await page.setViewport({
    width: format.width,
    height: format.height,
    deviceScaleFactor: format.dpr,
    isMobile: !!format.touch,
    hasTouch: !!format.touch,
  });
  page.__logs = logs;
  page.__errors = pageErrors;
  return page;
}

async function waitReady(page) {
  await page.waitForSelector('canvas', { timeout: 180000 });
  await page.waitForFunction(
    () => document.getElementById('boot-cta')?.style?.display === 'inline-block',
    { timeout: 180000 }
  );
}

function center(format) {
  return { x: Math.floor(format.width / 2), y: Math.floor(format.height / 2) };
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

async function runKeyboard(format) {
  const tag = `${format.id}/keyboard`;
  const page = await newPage(format);
  try {
    page.__errors.length = 0;
    await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 180000 });
    await waitReady(page);
    await shot(page, `${format.id}_kb_01_boot`);

    await page.keyboard.press('Enter');
    await sleep(500);
    const hidden = await page.evaluate(() =>
      document.getElementById('boot')?.classList.contains('hidden')
    );
    if (!hidden) await page.keyboard.press('Space');
    await sleep(400);
    const hidden2 = await page.evaluate(() =>
      document.getElementById('boot')?.classList.contains('hidden')
    );
    if (hidden2) pass(`${tag}: boot dismiss`);
    else fail(`${tag}: boot dismiss`);

    await focusCanvas(page);
    await shot(page, `${format.id}_kb_02_menu`);
    await page.keyboard.press('Enter');
    await sleep(900);
    await shot(page, `${format.id}_kb_03_mode`);
    await page.keyboard.press('ArrowDown');
    await sleep(200);
    await page.keyboard.press('Space');
    await sleep(1800);
    await shot(page, `${format.id}_kb_04_play`);
    if (page.__errors.length === 0) pass(`${tag}: play`);
    else fail(`${tag}: play`, page.__errors.join('; '));

    await page.keyboard.down('KeyD');
    await sleep(250);
    await page.keyboard.up('KeyD');
    await page.keyboard.press('Space');
    await sleep(500);
    await shot(page, `${format.id}_kb_05_input`);
    if (page.__errors.length === 0) pass(`${tag}: move+dash`);
    else fail(`${tag}: move+dash`, page.__errors.at(-1));

    await page.keyboard.press('Escape');
    await sleep(800);
    await shot(page, `${format.id}_kb_06_esc`);
    if (page.__errors.length === 0) pass(`${tag}: esc menu`);
    else fail(`${tag}: esc menu`, page.__errors.at(-1));
  } finally {
    await page.close();
  }
}

async function runMouse(format) {
  const tag = `${format.id}/mouse`;
  const page = await newPage(format);
  const { x: cx, y: cy } = center(format);
  try {
    page.__errors.length = 0;
    await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 180000 });
    await waitReady(page);
    await page.mouse.click(cx, cy);
    await sleep(500);
    const hidden = await page.evaluate(() =>
      document.getElementById('boot')?.classList.contains('hidden')
    );
    if (hidden) pass(`${tag}: boot click`);
    else fail(`${tag}: boot click`);
    await shot(page, `${format.id}_mouse_01_menu`);

    await page.mouse.click(cx, cy);
    await sleep(900);
    await shot(page, `${format.id}_mouse_02_mode`);
    if (page.__errors.length === 0) pass(`${tag}: mode`);
    else fail(`${tag}: mode`, page.__errors.join('; '));

    await page.mouse.click(cx, cy);
    await sleep(1800);
    await shot(page, `${format.id}_mouse_03_play`);
    if (page.__errors.length === 0) pass(`${tag}: play`);
    else fail(`${tag}: play`, page.__errors.join('; '));

    const x0 = Math.floor(format.width * 0.35);
    const y0 = Math.floor(format.height * 0.55);
    const x1 = Math.floor(format.width * 0.6);
    const y1 = Math.floor(format.height * 0.4);
    await page.mouse.move(x0, y0);
    await page.mouse.down();
    await page.mouse.move(x1, y1, { steps: 6 });
    await sleep(300);
    await page.mouse.click(x1, y1, { button: 'right' });
    await sleep(300);
    await page.mouse.up();
    await shot(page, `${format.id}_mouse_04_input`);
    if (page.__errors.length === 0) pass(`${tag}: drag+right-click dash`);
    else fail(`${tag}: drag+right-click dash`, page.__errors.at(-1));
  } finally {
    await page.close();
  }
}

async function runTouch(format) {
  const tag = `${format.id}/touch`;
  const page = await newPage(format);
  const client = await page.createCDPSession();
  await client.send('Emulation.setTouchEmulationEnabled', {
    enabled: true,
    maxTouchPoints: 2,
  });
  const { x: cx, y: cy } = center(format);
  try {
    page.__errors.length = 0;
    await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 180000 });
    await waitReady(page);
    await page.touchscreen.tap(cx, cy);
    await sleep(500);
    const hidden = await page.evaluate(() =>
      document.getElementById('boot')?.classList.contains('hidden')
    );
    if (hidden) pass(`${tag}: boot tap`);
    else fail(`${tag}: boot tap`);
    await shot(page, `${format.id}_touch_01_menu`);

    await page.touchscreen.tap(cx, cy);
    await sleep(900);
    await shot(page, `${format.id}_touch_02_mode`);
    if (page.__errors.length === 0) pass(`${tag}: mode`);
    else fail(`${tag}: mode`, page.__errors.join('; '));

    await page.touchscreen.tap(cx, Math.floor(format.height * 0.55));
    await sleep(1800);
    await shot(page, `${format.id}_touch_03_play`);
    if (page.__errors.length === 0) pass(`${tag}: play`);
    else fail(`${tag}: play`, page.__errors.join('; '));

    const x0 = Math.floor(format.width * 0.35);
    const y0 = Math.floor(format.height * 0.55);
    const x1 = Math.floor(format.width * 0.55);
    const y1 = Math.floor(format.height * 0.4);
    const x2 = Math.floor(format.width * 0.75);
    const y2 = Math.floor(format.height * 0.65);
    await client.send('Input.dispatchTouchEvent', {
      type: 'touchStart',
      touchPoints: [{ x: x0, y: y0, id: 1 }],
    });
    await sleep(120);
    await client.send('Input.dispatchTouchEvent', {
      type: 'touchMove',
      touchPoints: [{ x: x1, y: y1, id: 1 }],
    });
    await sleep(150);
    await client.send('Input.dispatchTouchEvent', {
      type: 'touchStart',
      touchPoints: [
        { x: x1, y: y1, id: 1 },
        { x: x2, y: y2, id: 2 },
      ],
    });
    await sleep(200);
    await client.send('Input.dispatchTouchEvent', {
      type: 'touchEnd',
      touchPoints: [{ x: x1, y: y1, id: 1 }],
    });
    await sleep(400);
    await shot(page, `${format.id}_touch_04_input`);
    if (page.__errors.length === 0) pass(`${tag}: move+2nd-finger dash`);
    else fail(`${tag}: move+2nd-finger dash`, page.__errors.at(-1));
  } finally {
    await page.close();
  }
}

for (const format of MATRIX.formats) {
  console.log('\n==== format', format.id, '====');
  // Always exercise keyboard (WASD/arrows/Space) so desktop keys work on all sizes
  await runKeyboard(format);
  if (!format.touch) {
    await runMouse(format);
  } else {
    await runTouch(format);
  }
}

await browser.close();

const failed = results.filter((r) => !r.ok);
fs.writeFileSync(
  path.join(OUT, 'results.json'),
  JSON.stringify(
    {
      matrix_formats: MATRIX.formats.map((f) => f.id),
      results,
      failed: failed.length,
      at: new Date().toISOString(),
    },
    null,
    2
  )
);
console.log('\n=== E2E SUMMARY ===');
console.log('passed', results.filter((r) => r.ok).length, '/', results.length);
if (failed.length) {
  console.error('FAILED:', failed);
  process.exit(1);
}
process.exit(0);
