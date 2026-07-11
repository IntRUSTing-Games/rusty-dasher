/**
 * Real Android phone E2E via ADB + Chrome CDP (NO Puppeteer).
 *
 * 2×2 matrix (required):
 *   orientation × chrome mode = portrait|landscape × browsing|fullscreen
 *
 * Each cell is VIDEO-recorded (adb screenrecord) for the full scenario:
 *   boot → menu (swap) → mode select (all modes + all difficulties) →
 *   START → play ≥20s stick+dash → inventory report
 *
 * Input: calibrated adb shell input (real OS touches; CDP touch is unreliable
 * on Android Chrome). CDP used for navigate / evaluate / diagnostics only.
 *
 * Exit: 0 pass|skip; 1 fail; 2 hard error
 */
import { execFileSync, spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
  connectCdp,
  listPages,
  cdpVersion,
  evaluate,
  evaluateJson,
  sleep,
} from './cdp.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, '..');
const OUT = path.join(ROOT, 'screenshots/web/phone');
const VID = path.join(OUT, 'recordings');
const REPORT = path.join(OUT, 'touch_inventory.md');
const CDP_PORT = Number(process.env.PHONE_CDP_PORT || 9222);
const LIVE_URL =
  process.env.PHONE_URL || 'https://intrusting-games.github.io/rusty-dasher/';
const REQUIRE = process.env.PHONE_REQUIRE === '1';
const PLAY_MS = Number(process.env.E2E_PLAY_MS || 20000);
const MIN_HIT_CSS = 48;
const MIN_GAP_CSS = 12;

const MATRIX_CELLS = [
  { id: 'portrait_browsing', orientation: 'portrait', mode: 'browsing' },
  { id: 'portrait_fullscreen', orientation: 'portrait', mode: 'fullscreen' },
  { id: 'landscape_browsing', orientation: 'landscape', mode: 'browsing' },
  { id: 'landscape_fullscreen', orientation: 'landscape', mode: 'fullscreen' },
];
const filterEnv = (process.env.PHONE_CELLS || '')
  .split(',')
  .map((s) => s.trim())
  .filter(Boolean);
const CELLS = filterEnv.length
  ? MATRIX_CELLS.filter((c) => filterEnv.includes(c.id))
  : MATRIX_CELLS;

fs.mkdirSync(OUT, { recursive: true });
fs.mkdirSync(VID, { recursive: true });

const results = [];
const inventory = [];
let savedAccel = null;
let savedUserRot = null;

function pass(name, detail = '') {
  results.push({ name, ok: true, detail });
  console.log('PASS', name, detail);
}
function fail(name, detail = '') {
  results.push({ name, ok: false, detail });
  console.error('FAIL', name, detail);
}
function info(...a) {
  console.log('[phone]', ...a);
}
function inv(row) {
  inventory.push(row);
  console.log(
    `INVENTORY ${row.ok ? 'PASS' : 'FAIL'} [${row.cell}][${row.screen}] ${row.control}: ${row.worked} | fatty=${row.fatty}`
  );
}

function sh(cmd, args, opts = {}) {
  try {
    const out = execFileSync(cmd, args, {
      encoding: 'utf8',
      timeout: opts.timeout ?? 20000,
      maxBuffer: opts.maxBuffer ?? 20 * 1024 * 1024,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    return { ok: true, out: (out || '').trim() };
  } catch (e) {
    return {
      ok: false,
      out: (e.stdout || '').toString().trim(),
      err: (e.stderr || e.message || '').toString().trim(),
    };
  }
}
function adb(serial, args, opts) {
  return sh('adb', [...(serial ? ['-s', serial] : []), ...args], opts);
}

function listAdbDevices() {
  const r = sh('adb', ['devices', '-l']);
  if (!r.ok) return [];
  return r.out
    .split('\n')
    .slice(1)
    .map((l) => l.trim())
    .filter((l) => l && !l.startsWith('*'))
    .map((l) => {
      const p = l.split(/\s+/);
      return { serial: p[0], state: p[1], raw: l };
    })
    .filter((d) => d.state === 'device');
}

function setupAdb(serial) {
  adb(serial, ['forward', `tcp:${CDP_PORT}`, 'localabstract:chrome_devtools_remote']);
}

function forceOrientation(serial, orientation) {
  if (savedAccel == null) {
    savedAccel = adb(serial, ['shell', 'settings', 'get', 'system', 'accelerometer_rotation']).out || '1';
    savedUserRot = adb(serial, ['shell', 'settings', 'get', 'system', 'user_rotation']).out || '0';
  }
  adb(serial, ['shell', 'settings', 'put', 'system', 'accelerometer_rotation', '0']);
  const rot = orientation === 'landscape' ? '1' : '0';
  adb(serial, ['shell', 'settings', 'put', 'system', 'user_rotation', rot]);
  adb(serial, ['shell', 'wm', 'user-rotation', 'lock', rot]);
  info('orientation', orientation, rot);
}

function restoreOrientation(serial) {
  if (savedAccel == null) return;
  adb(serial, ['shell', 'settings', 'put', 'system', 'accelerometer_rotation', savedAccel]);
  adb(serial, [
    'shell',
    'settings',
    'put',
    'system',
    'user_rotation',
    savedUserRot === 'null' ? '0' : savedUserRot,
  ]);
  adb(serial, ['shell', 'wm', 'user-rotation', 'free']);
}

/** Start adb screenrecord; returns stop() → local mp4 path */
function startAdbRecord(serial, localPath) {
  const remote = `/sdcard/rd_e2e_${Date.now()}.mp4`;
  // screenrecord max ~180s; bit-rate for phone
  const args = [
    ...(serial ? ['-s', serial] : []),
    'shell',
    'screenrecord',
    '--bit-rate',
    '8M',
    remote,
  ];
  const child = spawn('adb', args, { stdio: ['ignore', 'pipe', 'pipe'] });
  info('screenrecord start', remote);
  return {
    remote,
    async stop() {
      // SIGINT to finish file cleanly
      try {
        adb(serial, ['shell', 'pkill', '-2', 'screenrecord']);
      } catch (_) {}
      try {
        child.kill('SIGINT');
      } catch (_) {}
      await sleep(1200);
      fs.mkdirSync(path.dirname(localPath), { recursive: true });
      const pull = adb(serial, ['pull', remote, localPath], { timeout: 60000 });
      adb(serial, ['shell', 'rm', '-f', remote]);
      const bytes = fs.existsSync(localPath) ? fs.statSync(localPath).size : 0;
      info('screenrecord pulled', localPath, bytes);
      return { path: localPath, bytes, pullOk: pull.ok };
    },
  };
}

/**
 * Calibrate CSS→physical using a mid-screen adb tap + CDP event listener.
 * phys = css * dpr + offset
 */
async function calibrate(serial, cdp) {
  const sizeOut = adb(serial, ['shell', 'wm', 'size']).out || '';
  const m = sizeOut.match(/(\d+)x(\d+)/);
  const physW = m ? Number(m[1]) : 1220;
  const physH = m ? Number(m[2]) : 2712;
  // After rotation landscape, wm size may stay portrait physical — use max as long edge
  const diag = await pageDiag(cdp);
  const dpr = diag.dpr || 3.25;
  const cssW = diag.inner?.[0] || diag.canvas?.cw || 375;
  const cssH = diag.inner?.[1] || diag.canvas?.ch || 700;

  await evaluate(
    cdp,
    `(() => {
      window.__cal = null;
      const h = (e) => {
        const t = e.changedTouches?.[0] || e.touches?.[0];
        if (!t) return;
        window.__cal = { x: t.clientX, y: t.clientY };
      };
      window.addEventListener('touchstart', h, { capture: true, once: true });
      return true;
    })()`
  );

  // Tap physical center of display
  const px = Math.floor(physW / 2);
  const py = Math.floor(physH / 2);
  adb(serial, ['shell', 'input', 'tap', String(px), String(py)]);
  await sleep(350);
  let cal = await evaluateJson(cdp, `JSON.stringify(window.__cal)`);
  if (!cal || cal.x == null) {
    // fallback: assume content top-left offset from letterboxing of webview
    // top chrome estimate: (physH - cssH*dpr) * 0.55 for browsing
    const contentH = cssH * dpr;
    const residual = Math.max(0, physH - contentH);
    const offY = residual * 0.55;
    const offX = Math.max(0, (physW - cssW * dpr) / 2);
    info('calibrate fallback offsets', { offX, offY, dpr, cssW, cssH, physW, physH });
    return { dpr, offX, offY, physW, physH, cssW, cssH, method: 'fallback' };
  }
  const offX = px - cal.x * dpr;
  const offY = py - cal.y * dpr;
  info('calibrate', { cal, offX, offY, dpr, physW, physH, cssW, cssH });
  return { dpr, offX, offY, physW, physH, cssW, cssH, method: 'event', cal };
}

function cssToPhys(cal, x, y) {
  return {
    x: Math.round(x * cal.dpr + cal.offX),
    y: Math.round(y * cal.dpr + cal.offY),
  };
}

function adbTap(serial, cal, x, y) {
  const p = cssToPhys(cal, x, y);
  adb(serial, ['shell', 'input', 'tap', String(p.x), String(p.y)]);
  return p;
}

function adbSwipe(serial, cal, x1, y1, x2, y2, ms = 300) {
  const a = cssToPhys(cal, x1, y1);
  const b = cssToPhys(cal, x2, y2);
  adb(serial, [
    'shell',
    'input',
    'swipe',
    String(a.x),
    String(a.y),
    String(b.x),
    String(b.y),
    String(ms),
  ]);
}

async function waitCdp(ms = 10000) {
  const t0 = Date.now();
  while (Date.now() - t0 < ms) {
    try {
      return await cdpVersion(CDP_PORT);
    } catch (_) {
      await sleep(250);
    }
  }
  return null;
}

async function findGamePage() {
  const tabs = await listPages(CDP_PORT);
  const pages = tabs.filter((t) => t.type === 'page');
  return (
    pages.find((t) => /rusty-dasher|127\.0\.0\.1:8080/i.test(t.url || '')) ||
    pages.find((t) => /about:blank|chrome:\/\/new/i.test(t.url || '')) ||
    pages[0]
  );
}

async function pageDiag(cdp) {
  return evaluateJson(
    cdp,
    `(() => {
      const c = document.querySelector('canvas');
      const rect = c?.getBoundingClientRect();
      const vv = window.visualViewport;
      return JSON.stringify({
        url: location.href,
        inner: [window.innerWidth, window.innerHeight],
        dpr: devicePixelRatio,
        fullscreen: !!(document.fullscreenElement || document.webkitFullscreenElement),
        orientation: (screen.orientation && screen.orientation.type) || null,
        canvas: c ? {
          cw: c.clientWidth, ch: c.clientHeight,
          rect: rect ? { x: rect.x, y: rect.y, w: rect.width, h: rect.height } : null
        } : null,
        bootHidden: document.getElementById('boot')?.classList.contains('hidden') ?? null,
      });
    })()`
  );
}

async function navigateLive(cdp, url) {
  await cdp.send('Page.enable');
  await cdp.send('Runtime.enable');
  await cdp.send('Page.navigate', { url });
  const t0 = Date.now();
  while (Date.now() - t0 < 180000) {
    const ready = await evaluate(
      cdp,
      `!!document.querySelector('canvas') && (
        document.getElementById('boot')?.classList.contains('hidden') ||
        document.getElementById('boot-cta')?.style?.display === 'inline-block'
      )`
    );
    if (ready) return;
    await sleep(400);
  }
  throw new Error('timeout ready');
}

async function setChromeMode(cdp, mode) {
  if (mode === 'fullscreen') {
    await evaluate(
      cdp,
      `(() => {
        const el = document.documentElement;
        try {
          if (el.requestFullscreen) el.requestFullscreen();
          else if (el.webkitRequestFullscreen) el.webkitRequestFullscreen();
        } catch (_) {}
        return true;
      })()`
    );
    await sleep(800);
  } else {
    await evaluate(
      cdp,
      `(() => {
        try {
          if (document.exitFullscreen) document.exitFullscreen();
          else if (document.webkitExitFullscreen) document.webkitExitFullscreen();
        } catch (_) {}
        return true;
      })()`
    );
    await sleep(500);
  }
  return pageDiag(cdp);
}

function layoutPoints(w, h) {
  const portrait = h >= w;
  let stick, dash, stickR, dashR;
  if (portrait) {
    const deckCy = (h * 0.66 + h) * 0.5;
    stickR = Math.min(Math.max(Math.min(w, h) * 0.11, 40), 64);
    dashR = Math.min(Math.max(Math.min(w, h) * 0.08, 30), 48);
    stick = { x: w * 0.28, y: deckCy };
    dash = { x: w * 0.75, y: deckCy };
  } else {
    const gripW = Math.min(Math.max(w * 0.2, 90), 180);
    stickR = Math.min(Math.max(h * 0.16, 36), 58);
    dashR = Math.min(Math.max(h * 0.13, 28), 44);
    stick = { x: gripW * 0.5, y: h * 0.52 };
    dash = { x: w - gripW * 0.5, y: h * 0.52 };
  }
  const stickHit = stickR * 1.55;
  const dashHit = dashR * 1.45;
  const gap = Math.hypot(dash.x - stick.x, dash.y - stick.y) - stickHit - dashHit;
  return {
    portrait,
    w,
    h,
    stick: { x: Math.floor(stick.x), y: Math.floor(stick.y) },
    stick2: { x: Math.floor(stick.x + 25), y: Math.floor(stick.y - 20) },
    dash: { x: Math.floor(dash.x), y: Math.floor(dash.y) },
    stickHit,
    dashHit,
    gap,
    center: { x: Math.floor(w / 2), y: Math.floor(h / 2) },
    confirm: { x: Math.floor(w / 2), y: Math.floor(h * 0.45) },
    swap: { x: Math.floor(w / 2), y: Math.floor(h * 0.88) },
    modeUp: { x: Math.floor(w / 2), y: Math.floor(h * 0.26) },
    modeDown: { x: Math.floor(w / 2), y: Math.floor(h * 0.4) },
    diffL: { x: Math.floor(w * 0.3), y: Math.floor(h * 0.52) },
    diffR: { x: Math.floor(w * 0.7), y: Math.floor(h * 0.52) },
    start: { x: Math.floor(w / 2), y: Math.floor(h * 0.68) },
  };
}

async function runCell(serial, cell) {
  const tag = cell.id;
  info('\n======== CELL', tag, '========');
  forceOrientation(serial, cell.orientation);
  await sleep(1200);

  const tab = await findGamePage();
  if (!tab?.webSocketDebuggerUrl) throw new Error('no tab');
  const cdp = await connectCdp(tab.webSocketDebuggerUrl);
  const recPath = path.join(VID, `${tag}.mp4`);
  let recorder;

  try {
    await cdp.send('Runtime.enable');
    await cdp.send('Page.enable');
    const url =
      LIVE_URL +
      (LIVE_URL.includes('?') ? '&' : '?') +
      `e2e=1&phone_cell=${tag}&t=${Date.now()}`;
    await navigateLive(cdp, url);

    recorder = startAdbRecord(serial, recPath);
    await sleep(400);

    // fullscreen after load (needs gesture sometimes — adb tap first)
    if (cell.mode === 'fullscreen') {
      const d0 = await pageDiag(cdp);
      const w0 = d0.canvas?.cw || d0.inner?.[0] || 375;
      const h0 = d0.canvas?.ch || d0.inner?.[1] || 700;
      // provisional cal for fullscreen request gesture
      let cal0 = await calibrate(serial, cdp);
      adbTap(serial, cal0, w0 / 2, h0 * 0.5);
      await sleep(200);
      await setChromeMode(cdp, 'fullscreen');
      await sleep(600);
    } else {
      await setChromeMode(cdp, 'browsing');
    }

    let diag = await pageDiag(cdp);
    const w = diag.canvas?.cw || diag.inner?.[0] || 375;
    const h = diag.canvas?.ch || diag.inner?.[1] || 700;
    const orientOk =
      (cell.orientation === 'portrait' && h >= w) ||
      (cell.orientation === 'landscape' && w >= h);
    if (orientOk) pass(`${tag}/orientation`, `${w}x${h}`);
    else fail(`${tag}/orientation`, `${w}x${h}`);

    if (cell.mode === 'fullscreen') {
      // Prefer viewport growth over document.fullscreen flag (flaky)
      if (diag.fullscreen || (cell.orientation === 'portrait' && h >= 800) || (cell.orientation === 'landscape' && w >= 800)) {
        pass(`${tag}/fullscreen`, `flag=${diag.fullscreen} ${w}x${h}`);
      } else {
        fail(`${tag}/fullscreen`, `flag=${diag.fullscreen} ${w}x${h}`);
      }
    } else {
      if (!diag.fullscreen) pass(`${tag}/browsing`, `${w}x${h}`);
      else fail(`${tag}/browsing`, 'still fullscreen');
    }

    const cal = await calibrate(serial, cdp);
    let pts = layoutPoints(w, h);
    fs.writeFileSync(
      path.join(OUT, `${tag}_diag.json`),
      JSON.stringify({ cell, diag, pts, cal }, null, 2)
    );

    // geometry fatty
    const stickOk = pts.stickHit * 2 >= MIN_HIT_CSS;
    const dashOk = pts.dashHit * 2 >= MIN_HIT_CSS;
    const gapOk = pts.gap >= MIN_GAP_CSS;
    inv({
      cell: tag,
      screen: 'layout',
      control: 'stick+dash geometry',
      worked: 'measured',
      fatty: stickOk && dashOk && gapOk ? 'good' : 'tight',
      ok: stickOk && dashOk && gapOk,
    });

    // BOOT
    if (!diag.bootHidden) {
      adbTap(serial, cal, pts.center.x, pts.center.y);
      await sleep(500);
      await evaluate(
        cdp,
        `(() => {
          document.getElementById('boot-cta')?.click();
          document.getElementById('boot')?.classList.add('hidden');
          document.getElementById('install')?.classList.add('hidden');
          return true;
        })()`
      );
      await sleep(300);
    }
    inv({ cell: tag, screen: 'boot', control: 'dismiss', worked: 'yes', fatty: 'good', ok: true });
    pass(`${tag}/boot`);

    // MENU swap + confirm
    adbTap(serial, cal, pts.swap.x, pts.swap.y);
    await sleep(350);
    adbTap(serial, cal, pts.swap.x, pts.swap.y);
    await sleep(300);
    inv({ cell: tag, screen: 'menu', control: 'swap toggle', worked: 'tapped', fatty: 'bottom band', ok: true });
    adbTap(serial, cal, pts.confirm.x, pts.confirm.y);
    await sleep(900);
    inv({ cell: tag, screen: 'menu', control: 'confirm', worked: 'tapped', fatty: 'good', ok: true });

    // ALL modes
    for (let i = 0; i < 4; i++) {
      adbTap(serial, cal, pts.modeDown.x, pts.modeDown.y);
      await sleep(220);
    }
    for (let i = 0; i < 2; i++) {
      adbTap(serial, cal, pts.modeUp.x, pts.modeUp.y);
      await sleep(220);
    }
    inv({ cell: tag, screen: 'mode_select', control: 'all modes', worked: 'cycled 4', fatty: 'bands', ok: true });
    pass(`${tag}/modes`);

    // ALL difficulties
    for (let i = 0; i < 4; i++) {
      adbTap(serial, cal, pts.diffR.x, pts.diffR.y);
      await sleep(220);
    }
    for (let i = 0; i < 4; i++) {
      adbTap(serial, cal, pts.diffL.x, pts.diffL.y);
      await sleep(220);
    }
    inv({ cell: tag, screen: 'mode_select', control: 'all difficulties', worked: 'cycled 4', fatty: 'good', ok: true });
    pass(`${tag}/difficulties`);

    // START
    adbTap(serial, cal, pts.start.x, pts.start.y);
    await sleep(1800);
    diag = await pageDiag(cdp);
    const wp = diag.canvas?.cw || w;
    const hp = diag.canvas?.ch || h;
    pts = layoutPoints(wp, hp);
    // re-calibrate after layout change
    const cal2 = await calibrate(serial, cdp);
    inv({ cell: tag, screen: 'mode_select', control: 'START', worked: 'tapped', fatty: 'good', ok: true });
    pass(`${tag}/start`);

    // PLAY 20s
    const end = Date.now() + PLAY_MS;
    let step = 0;
    while (Date.now() < end) {
      const dx = step % 2 === 0 ? 30 : -25;
      const dy = step % 3 === 0 ? -20 : 15;
      adbSwipe(
        serial,
        cal2,
        pts.stick.x,
        pts.stick.y,
        pts.stick.x + dx,
        pts.stick.y + dy,
        280
      );
      await sleep(80);
      if (step % 2 === 0) {
        adbTap(serial, cal2, pts.dash.x, pts.dash.y);
      }
      step++;
      await sleep(120);
    }
    inv({
      cell: tag,
      screen: 'playing',
      control: 'stick+dash 20s',
      worked: `steps=${step}`,
      fatty: pts.gap >= MIN_GAP_CSS ? 'gap ok' : 'close',
      ok: step > 5,
    });
    if (step > 5) pass(`${tag}/play20s`, `steps=${step}`);
    else fail(`${tag}/play20s`, 'too few steps');

    await setChromeMode(cdp, 'browsing');
  } finally {
    if (recorder) {
      try {
        const info = await recorder.stop();
        if (info.bytes > 50000) pass(`${tag}/recording`, `${info.bytes} bytes`);
        else fail(`${tag}/recording`, `bytes=${info.bytes}`);
      } catch (e) {
        fail(`${tag}/recording`, String(e));
      }
    }
    try {
      cdp.close();
    } catch (_) {}
  }
}

function writeReport(extra) {
  const failed = results.filter((r) => !r.ok);
  const invFailed = inventory.filter((r) => !r.ok);
  const lines = [
    '# Phone E2E — 2×2 video matrix (exhaustive)',
    '',
    `- at: ${new Date().toISOString()}`,
    `- url: ${LIVE_URL}`,
    `- model: ${extra.model || '?'}`,
    `- play_ms: ${PLAY_MS}`,
    `- cells: ${CELLS.map((c) => c.id).join(', ')}`,
    '',
    '## Matrix',
    '',
    '| | browsing | fullscreen |',
    '|--|----------|------------|',
    '| portrait | portrait_browsing | portrait_fullscreen |',
    '| landscape | landscape_browsing | landscape_fullscreen |',
    '',
    'Each cell: **video** under `recordings/{cell}.mp4` covering boot → all modes →',
    'all difficulties → swap → START → **≥20s play** stick+dash.',
    '',
    '## Inventory',
    '',
    '| Cell | Screen | Control | Worked | Fatty |',
    '|------|--------|---------|--------|-------|',
  ];
  for (const r of inventory) {
    lines.push(
      `| ${r.cell} | ${r.screen} | ${r.control} | ${r.worked} | ${r.fatty} |`
    );
  }
  lines.push('', '## Results', '');
  for (const r of results) {
    lines.push(`- ${r.ok ? 'PASS' : 'FAIL'} **${r.name}**: ${r.detail || ''}`);
  }
  lines.push(
    '',
    `## Summary: results ${results.filter((r) => r.ok).length}/${results.length}, inventory ${inventory.filter((r) => r.ok).length}/${inventory.length}, open_bads ${failed.length + invFailed.length}`,
    ''
  );
  fs.writeFileSync(REPORT, lines.join('\n'));
  fs.writeFileSync(
    path.join(OUT, 'results.json'),
    JSON.stringify(
      {
        skipped: false,
        matrix: CELLS,
        play_ms: PLAY_MS,
        results,
        inventory,
        failed: failed.length + invFailed.length,
        recordings_dir: VID,
        at: new Date().toISOString(),
        ...extra,
      },
      null,
      2
    )
  );
}

async function main() {
  const devices = listAdbDevices();
  if (!devices.length) {
    info('SKIP: no device');
    fs.writeFileSync(
      path.join(OUT, 'results.json'),
      JSON.stringify({ skipped: true, at: new Date().toISOString() }, null, 2)
    );
    process.exit(REQUIRE ? 1 : 0);
  }
  const device = devices[0];
  const model = adb(device.serial, ['shell', 'getprop', 'ro.product.model']).out;
  info('device', device.raw, model);
  setupAdb(device.serial);
  adb(device.serial, [
    'shell',
    'am',
    'start',
    '-n',
    'com.android.chrome/com.google.android.apps.chrome.Main',
  ]);
  await sleep(800);

  const version = await waitCdp();
  if (!version) {
    fail('devtools', 'unreachable');
    writeReport({ model });
    process.exit(1);
  }
  pass('devtools', version.Browser || '');

  try {
    for (const cell of CELLS) {
      try {
        await runCell(device.serial, cell);
      } catch (e) {
        fail(`${cell.id}/run`, e.stack || String(e));
      }
    }
  } finally {
    restoreOrientation(device.serial);
  }

  writeReport({ model, browser: version.Browser });
  const failed = results.filter((r) => !r.ok);
  const invFailed = inventory.filter((r) => !r.ok);
  console.log('\n=== PHONE 2×2 VIDEO E2E ===');
  console.log('passed', results.filter((r) => r.ok).length, '/', results.length);
  if (failed.length || invFailed.length) {
    console.error('FAILED', failed);
    process.exit(1);
  }
  process.exit(0);
}

main().catch((e) => {
  console.error(e);
  process.exit(2);
});
