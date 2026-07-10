/**
 * Capture title + mode-select on all six target surfaces:
 * 4K, 1080p, tablet portrait/landscape, phone portrait/landscape.
 * Click boot CTA (not Enter) so we don't double-advance menus.
 */
import puppeteer from 'puppeteer-core';
import fs from 'fs';
import path from 'path';

const OUT = '/code/1st-rust-game/screenshots/viewports';
const URL = 'http://127.0.0.1:8080/';
fs.mkdirSync(OUT, { recursive: true });

const VIEWPORTS = [
  { name: '4k', width: 3840, height: 2160, dpr: 1, touch: false },
  { name: '1080p', width: 1920, height: 1080, dpr: 1, touch: false },
  { name: 'tablet_portrait', width: 768, height: 1024, dpr: 2, touch: true },
  { name: 'tablet_landscape', width: 1024, height: 768, dpr: 2, touch: true },
  { name: 'phone_portrait', width: 390, height: 844, dpr: 2, touch: true },
  { name: 'phone_landscape', width: 844, height: 390, dpr: 2, touch: true },
];

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

async function boot(page) {
  await page.waitForSelector('canvas', { timeout: 120000 });
  await page.waitForFunction(
    () => document.getElementById('boot-cta')?.style?.display === 'inline-block',
    { timeout: 120000 }
  );
  // Click the HTML boot button so the game doesn't also receive Enter
  await page.click('#boot-cta');
  await sleep(900);
  await page.evaluate(() => {
    const c = document.querySelector('canvas');
    if (c) {
      c.tabIndex = 0;
      c.focus();
    }
  });
  await sleep(500);
}

async function shot(page, name) {
  const file = path.join(OUT, name + '.png');
  await page.screenshot({ path: file, fullPage: false });
  console.log('saved', file, fs.statSync(file).size);
}

const browser = await puppeteer.launch({
  executablePath: '/home/viny/bin/google-chrome',
  headless: 'new',
  args: [
    '--no-sandbox',
    '--disable-gpu-sandbox',
    '--use-gl=angle',
    '--use-angle=gl-egl',
    '--enable-webgl',
    '--ignore-gpu-blocklist',
    '--window-size=1920,1080',
  ],
});

for (const vp of VIEWPORTS) {
  const page = await browser.newPage();
  await page.setViewport({
    width: vp.width,
    height: vp.height,
    deviceScaleFactor: vp.dpr,
    isMobile: vp.touch,
    hasTouch: vp.touch,
  });
  console.log('viewport', vp.name);
  await page.goto(URL, { waitUntil: 'networkidle0', timeout: 120000 });
  await boot(page);
  await sleep(1400);
  await shot(page, `${vp.name}_01_menu`);

  // Title → mode select (one Enter only)
  await page.keyboard.press('Enter');
  await sleep(1200);
  await shot(page, `${vp.name}_02_mode_select`);
  await page.close();
}

await browser.close();
console.log('done');
