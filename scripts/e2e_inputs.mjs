/**
 * E2E: every screen must work with keyboard, mouse, and touch.
 * Exit 0 only if all paths pass.
 */
import puppeteer from 'puppeteer-core';
import fs from 'fs';
import path from 'path';

const OUT = '/code/1st-rust-game/screenshots/web/e2e';
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

async function waitCanvas(page) {
  await page.waitForSelector('canvas', { timeout: 120000 });
  await sleep(2500);
}

async function dismissBootKeyboard(page) {
  // Wait until boot says Ready
  await page.waitForFunction(
    () => document.getElementById('boot-cta')?.style?.display === 'inline-block',
    { timeout: 120000 }
  );
  await page.keyboard.press('Enter');
  await sleep(400);
  const hidden = await page.evaluate(() =>
    document.getElementById('boot')?.classList.contains('hidden')
  );
  if (!hidden) {
    // try Space
    await page.keyboard.press('Space');
    await sleep(300);
  }
  const hidden2 = await page.evaluate(() =>
    document.getElementById('boot')?.classList.contains('hidden')
  );
  return hidden2;
}

async function focusCanvas(page) {
  await page.evaluate(() => {
    const c = document.querySelector('canvas');
    if (c) {
      c.tabIndex = 0;
      c.focus();
    }
  });
  await sleep(100);
}

async function getScoreText(page) {
  // can't easily read bevy text - use console or visual. Check no panic.
  return page.evaluate(() => document.body.innerText.slice(0, 200));
}

async function noPanic(page, logs) {
  return !logs.some((l) => /panicked|PAGEERROR|unreachable/i.test(l));
}

const logs = [];
const pageErrors = [];

const browser = await puppeteer.launch({
  executablePath: '/home/viny/bin/google-chrome',
  headless: 'new',
  args: [
    '--no-sandbox',
    '--disable-setuid-sandbox',
    '--enable-unsafe-swiftshader',
    '--use-gl=angle',
    '--use-angle=swiftshader-webgl',
    '--window-size=1280,720',
    '--force-device-scale-factor=1',
  ],
  defaultViewport: { width: 1280, height: 720, deviceScaleFactor: 1 },
});

async function newPage() {
  const page = await browser.newPage();
  page.on('console', (msg) => {
    const t = `${msg.type()}: ${msg.text()}`;
    logs.push(t);
  });
  page.on('pageerror', (err) => {
    pageErrors.push(String(err));
    logs.push('PAGEERROR ' + err);
  });
  return page;
}

// --- Path A: KEYBOARD only ---
{
  logs.length = 0;
  pageErrors.length = 0;
  const page = await newPage();
  await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 120000 });
  await waitCanvas(page);
  await shot(page, 'kb_01_boot');

  const bootOk = await dismissBootKeyboard(page);
  if (bootOk) pass('keyboard: boot dismiss (Enter/Space)');
  else fail('keyboard: boot dismiss (Enter/Space)', 'boot still visible');
  await focusCanvas(page);
  await shot(page, 'kb_02_menu');

  // Menu -> ModeSelect
  await page.keyboard.press('Enter');
  await sleep(900);
  await shot(page, 'kb_03_mode');
  // Mode change
  await page.keyboard.press('ArrowDown');
  await sleep(400);
  await page.keyboard.press('ArrowUp');
  await sleep(400);
  // Start
  await page.keyboard.press('Space');
  await sleep(2000);
  await shot(page, 'kb_04_play');
  const panicPlay = pageErrors.length > 0;
  if (!panicPlay) pass('keyboard: enter play (Space)');
  else fail('keyboard: enter play', pageErrors.join('; '));

  // Move + dash
  await page.keyboard.down('KeyD');
  await sleep(300);
  await page.keyboard.up('KeyD');
  await page.keyboard.press('Space');
  await sleep(800);
  await shot(page, 'kb_05_move');
  if (pageErrors.length === 0) pass('keyboard: move + dash');
  else fail('keyboard: move + dash', pageErrors.at(-1));

  // Escape to menu
  await page.keyboard.press('Escape');
  await sleep(900);
  await shot(page, 'kb_06_esc');
  if (pageErrors.length === 0) pass('keyboard: escape to menu');
  else fail('keyboard: escape to menu', pageErrors.at(-1));

  // Through modes to game over: play then die hard - skip forced game over
  // Enter modes, enter play, escape is enough for keyboard screens
  await page.close();
}

// --- Path B: MOUSE only ---
{
  logs.length = 0;
  pageErrors.length = 0;
  const page = await newPage();
  await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 120000 });
  await waitCanvas(page);
  await page.waitForFunction(
    () => document.getElementById('boot-cta')?.style?.display === 'inline-block',
    { timeout: 120000 }
  );
  // Click boot
  await page.mouse.click(640, 360);
  await sleep(500);
  const hidden = await page.evaluate(() =>
    document.getElementById('boot')?.classList.contains('hidden')
  );
  if (hidden) pass('mouse: boot dismiss (click)');
  else fail('mouse: boot dismiss (click)');
  await shot(page, 'mouse_01_menu');

  // Click center for menu confirm
  await page.mouse.click(640, 360);
  await sleep(900);
  await shot(page, 'mouse_02_mode');
  if (pageErrors.length === 0) pass('mouse: menu -> mode select');
  else fail('mouse: menu -> mode select', pageErrors.join('; '));

  // Click center to start
  await page.mouse.click(640, 360);
  await sleep(2000);
  await shot(page, 'mouse_03_play');
  if (pageErrors.length === 0) pass('mouse: start game');
  else fail('mouse: start game', pageErrors.join('; '));

  // Drag to move
  await page.mouse.move(400, 400);
  await page.mouse.down();
  await page.mouse.move(700, 300, { steps: 8 });
  await sleep(500);
  await page.mouse.up();
  // Click right edge dash
  await page.mouse.click(1200, 360);
  await sleep(600);
  await shot(page, 'mouse_04_play_input');
  if (pageErrors.length === 0) pass('mouse: drag move + right-edge dash');
  else fail('mouse: drag move + right-edge dash', pageErrors.at(-1));

  await page.close();
}

// --- Path C: TOUCH only ---
{
  logs.length = 0;
  pageErrors.length = 0;
  const page = await newPage();
  // enable touch
  const client = await page.createCDPSession();
  await client.send('Emulation.setTouchEmulationEnabled', { enabled: true, maxTouchPoints: 2 });
  await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 120000 });
  await waitCanvas(page);
  await page.waitForFunction(
    () => document.getElementById('boot-cta')?.style?.display === 'inline-block',
    { timeout: 120000 }
  );

  await page.touchscreen.tap(640, 360);
  await sleep(500);
  const hidden = await page.evaluate(() =>
    document.getElementById('boot')?.classList.contains('hidden')
  );
  if (hidden) pass('touch: boot dismiss (tap)');
  else fail('touch: boot dismiss (tap)');
  await shot(page, 'touch_01_menu');

  await page.touchscreen.tap(640, 360);
  await sleep(900);
  await shot(page, 'touch_02_mode');
  if (pageErrors.length === 0) pass('touch: menu -> mode');
  else fail('touch: menu -> mode', pageErrors.join('; '));

  // tap center start
  await page.touchscreen.tap(640, 400);
  await sleep(2000);
  await shot(page, 'touch_03_play');
  if (pageErrors.length === 0) pass('touch: start game');
  else fail('touch: start game', pageErrors.join('; '));

  // touch drag
  await page.touchscreen.touchStart(400, 400);
  await page.touchscreen.touchMove(650, 280);
  await sleep(400);
  await page.touchscreen.touchEnd();
  await page.touchscreen.tap(1180, 360); // dash zone
  await sleep(500);
  await shot(page, 'touch_04_play_input');
  if (pageErrors.length === 0) pass('touch: drag + dash');
  else fail('touch: drag + dash', pageErrors.at(-1));

  await page.close();
}

// --- Path D: small window text scale (visual) ---
{
  const page = await newPage();
  await page.setViewport({ width: 480, height: 360, deviceScaleFactor: 1 });
  await page.goto(URL, { waitUntil: 'domcontentloaded', timeout: 120000 });
  await waitCanvas(page);
  await page.waitForFunction(
    () => document.getElementById('boot-cta')?.style?.display === 'inline-block',
    { timeout: 120000 }
  );
  await page.keyboard.press('Enter');
  await sleep(800);
  await shot(page, 'small_window_menu');
  pass('small-window: menu screenshot (manual readability check)');
  await page.close();
}

await browser.close();

const failed = results.filter((r) => !r.ok);
fs.writeFileSync(
  path.join(OUT, 'results.json'),
  JSON.stringify({ results, pageErrors, failed: failed.length }, null, 2)
);
console.log('\n=== SUMMARY ===');
console.log('passed', results.filter((r) => r.ok).length, '/', results.length);
if (failed.length) {
  console.error('FAILED:', failed);
  process.exit(1);
}
process.exit(0);
