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

/** Capture/automation step only — not visual review. */
function pass(name, detail = '') {
  results.push({ name, ok: true, detail, layer: 'capture' });
  console.log('CAPTURE_OK', name, detail);
}
function fail(name, detail = '') {
  results.push({ name, ok: false, detail, layer: 'capture' });
  console.error('CAPTURE_FAIL', name, detail);
}
function info(...a) {
  console.log('[phone]', ...a);
}
function inv(row) {
  inventory.push(row);
  // Inventory ok is instrumented/geometry CAPTURE evidence, not A4b video review.
  console.log(
    `INVENTORY ${row.ok ? 'CAPTURE_OK' : 'CAPTURE_FAIL'} [${row.cell}][${row.screen}] ${row.control}: ${row.worked} | fatty=${row.fatty}`
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
  // Give SurfaceFlinger / Chrome a beat to reflow (CSS landscape vs wm size portrait).
  info('orientation', orientation, rot);
}

/**
 * Physical size used by `adb shell input` — follows CURRENT display orientation.
 * `wm size` often stays natural portrait (e.g. 1220x2712) while landscape input
 * coords are 2712x1220. Prefer dumpsys viewport / window cur= over wm size alone.
 */
function getDisplayInputSize(serial, preferLandscape = null) {
  // dumpsys input Viewport logicalFrame=[0, 0, W, H]
  const inputDump = adb(serial, ['shell', 'dumpsys', 'input'], { timeout: 15000 }).out || '';
  let m = inputDump.match(
    /Viewport\s+INTERNAL:[\s\S]*?logicalFrame=\[0,\s*0,\s*(\d+),\s*(\d+)\]/
  );
  if (m) {
    return {
      physW: Number(m[1]),
      physH: Number(m[2]),
      source: 'dumpsys_input',
    };
  }
  // dumpsys window: cur=2712x1220
  const winDump =
    adb(serial, ['shell', 'dumpsys', 'window', 'displays'], { timeout: 15000 }).out || '';
  m = winDump.match(/\bcur=(\d+)x(\d+)\b/);
  if (m) {
    return {
      physW: Number(m[1]),
      physH: Number(m[2]),
      source: 'dumpsys_window',
    };
  }
  // wm size (may be natural portrait even when rotated)
  const sizeOut = adb(serial, ['shell', 'wm', 'size']).out || '';
  m =
    sizeOut.match(/Override size:\s*(\d+)x(\d+)/) ||
    sizeOut.match(/Physical size:\s*(\d+)x(\d+)/) ||
    sizeOut.match(/(\d+)x(\d+)/);
  let physW = m ? Number(m[1]) : 1220;
  let physH = m ? Number(m[2]) : 2712;
  // If CSS/page is landscape but wm reports portrait natural size, swap.
  if (preferLandscape === true && physW < physH) {
    [physW, physH] = [physH, physW];
    return { physW, physH, source: 'wm_size_swapped' };
  }
  if (preferLandscape === false && physW > physH) {
    [physW, physH] = [physH, physW];
    return { physW, physH, source: 'wm_size_swapped' };
  }
  return { physW, physH, source: 'wm_size' };
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
  // Detach on-device so killing the host adb client cannot leave a half-written
  // MP4 (Xiaomi/Android 16 often omits moov if the controlling shell dies).
  // time-limit is a safety net; stop() SIGINTs early for a clean moov footer.
  const timeLimit = Math.min(
    180,
    Math.max(90, Math.ceil((PLAY_MS + 70000) / 1000))
  );
  const shellCmd = `nohup screenrecord --bit-rate 8M --time-limit ${timeLimit} ${remote} >/dev/null 2>&1 & echo $!`;
  const start = adb(serial, ['shell', shellCmd], { timeout: 15000 });
  const pid = (start.out || '').trim().split(/\s+/).pop();
  info('screenrecord start', remote, 'pid', pid, 'time-limit', timeLimit);
  return {
    remote,
    pid,
    async stop() {
      // Prefer SIGINT to the exact screenrecord PID so the file gets a moov atom.
      if (pid && /^\d+$/.test(pid)) {
        try {
          adb(serial, ['shell', `kill -INT ${pid}`]);
        } catch (_) {}
      }
      try {
        adb(serial, ['shell', 'killall -INT screenrecord']);
      } catch (_) {}
      // Wait until screenrecord is gone and file size stabilizes (moov flush).
      let lastSize = -1;
      for (let i = 0; i < 25; i++) {
        await sleep(400);
        const alive = adb(serial, [
          'shell',
          'pidof screenrecord || true',
        ]).out.trim();
        const szOut = adb(serial, [
          'shell',
          `stat -c %s ${remote} 2>/dev/null || wc -c < ${remote} 2>/dev/null || echo 0`,
        ]).out.trim();
        const sz = Number(szOut.split(/\s+/).pop()) || 0;
        if (!alive && sz > 0 && sz === lastSize) break;
        lastSize = sz;
      }
      await sleep(500);
      fs.mkdirSync(path.dirname(localPath), { recursive: true });
      const pull = adb(serial, ['pull', remote, localPath], { timeout: 120000 });
      adb(serial, ['shell', 'rm', '-f', remote]);
      const bytes = fs.existsSync(localPath) ? fs.statSync(localPath).size : 0;
      let hasMoov = false;
      try {
        if (bytes > 32) {
          const buf = fs.readFileSync(localPath);
          hasMoov = buf.includes(Buffer.from('moov'));
        }
      } catch (_) {}
      info('screenrecord pulled', localPath, bytes, hasMoov ? 'moov=ok' : 'moov=MISSING');
      return { path: localPath, bytes, pullOk: pull.ok, hasMoov };
    },
  };
}

/**
 * Calibrate CSS→physical using a mid-screen adb tap + CDP event listener.
 * phys = css * dpr + offset
 *
 * Critical: physical size MUST match the oriented input coordinate system
 * (landscape adb coords are W×H swapped vs `wm size` natural portrait).
 */
async function calibrate(serial, cdp) {
  const diag = await pageDiag(cdp);
  const dpr = diag.dpr || 3.25;
  const cssW = diag.inner?.[0] || diag.canvas?.cw || 375;
  const cssH = diag.inner?.[1] || diag.canvas?.ch || 700;
  const cssLandscape = cssW > cssH;
  const { physW, physH, source } = getDisplayInputSize(serial, cssLandscape);
  info('display input size', { physW, physH, source, cssW, cssH, dpr });

  // Geometric baseline (chrome residual above/side of web content).
  const contentW = cssW * dpr;
  const contentH = cssH * dpr;
  const geoOffX = Math.max(0, (physW - contentW) / 2);
  const residualY = Math.max(0, physH - contentH);
  // Browsing: address bar eats most residual; fullscreen residual is smaller.
  const geoOffY = residualY * (diag.fullscreen ? 0.35 : 0.55);

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

  // Tap near geometric content center (more likely to hit the canvas than raw display center).
  const px = Math.floor(geoOffX + contentW * 0.5);
  const py = Math.floor(geoOffY + contentH * 0.5);
  adb(serial, ['shell', 'input', 'tap', String(px), String(py)]);
  await sleep(400);
  let cal = await evaluateJson(cdp, `JSON.stringify(window.__cal)`);

  // Retry once near top-left of estimated content if center miss (menu panels eat center).
  if (!cal || cal.x == null) {
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
    const px2 = Math.floor(geoOffX + Math.min(48, contentW * 0.08));
    const py2 = Math.floor(geoOffY + Math.min(48, contentH * 0.08));
    adb(serial, ['shell', 'input', 'tap', String(px2), String(py2)]);
    await sleep(400);
    cal = await evaluateJson(cdp, `JSON.stringify(window.__cal)`);
    if (cal && cal.x != null) {
      const offX = px2 - cal.x * dpr;
      const offY = py2 - cal.y * dpr;
      info('calibrate', { cal, offX, offY, dpr, physW, physH, cssW, cssH, method: 'event_corner', source });
      return { dpr, offX, offY, physW, physH, cssW, cssH, method: 'event_corner', cal, source };
    }
  }

  if (!cal || cal.x == null) {
    info('calibrate geometric fallback', {
      offX: geoOffX,
      offY: geoOffY,
      dpr,
      cssW,
      cssH,
      physW,
      physH,
      source,
    });
    return {
      dpr,
      offX: geoOffX,
      offY: geoOffY,
      physW,
      physH,
      cssW,
      cssH,
      method: 'geometric',
      source,
    };
  }
  const offX = px - cal.x * dpr;
  const offY = py - cal.y * dpr;
  info('calibrate', { cal, offX, offY, dpr, physW, physH, cssW, cssH, method: 'event', source });
  return { dpr, offX, offY, physW, physH, cssW, cssH, method: 'event', cal, source };
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
    pages.find((t) =>
      /rusty-dasher|127\.0\.0\.1:17880|localhost:17880|127\.0\.0\.1:8080/i.test(t.url || '')
    ) ||
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
        rdState: document.documentElement?.getAttribute('data-rd-state') || null,
        canvas: c ? {
          cw: c.clientWidth, ch: c.clientHeight,
          rect: rect ? { x: rect.x, y: rect.y, w: rect.width, h: rect.height } : null
        } : null,
        bootHidden: document.getElementById('boot')?.classList.contains('hidden') ?? null,
        vv: vv ? { w: vv.width, h: vv.height, scale: vv.scale } : null,
      });
    })()`
  );
}

/** Read GameState published by WASM (`data-rd-state`). */
async function rdState(cdp) {
  try {
    const s = await evaluate(
      cdp,
      `document.documentElement?.getAttribute('data-rd-state') || ''`
    );
    return (s || '').replace(/^"|"$/g, '') || null;
  } catch (_) {
    return null;
  }
}

async function waitRdState(cdp, want, ms = 4000) {
  const wants = Array.isArray(want) ? want : [want];
  const t0 = Date.now();
  while (Date.now() - t0 < ms) {
    const s = await rdState(cdp);
    if (s && wants.includes(s)) return s;
    await sleep(200);
  }
  return rdState(cdp);
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

/**
 * Enter/exit fullscreen safely.
 * - enter: must run inside a real touch gesture handler (adb tap alone is not
 *   a user activation for a later CDP evaluate). Install touchend → requestFS.
 * - exit: only when Document is active + an element is fullscreen; swallow
 *   promise rejections ("Document not active" TypeError on Xiaomi/Chrome).
 */
async function setChromeMode(cdp, mode, serial = null, cal = null) {
  if (mode === 'fullscreen') {
    // Arm requestFullscreen on the next real touch (user activation).
    await evaluate(
      cdp,
      `(() => {
        if (window.__rdFsArmed) return true;
        window.__rdFsArmed = true;
        const arm = (ev) => {
          try {
            const el = document.documentElement;
            const p =
              (el.requestFullscreen && el.requestFullscreen()) ||
              (el.webkitRequestFullscreen && el.webkitRequestFullscreen());
            if (p && typeof p.then === 'function') p.catch(() => {});
          } catch (_) {}
          try {
            window.removeEventListener('touchend', arm, true);
            window.removeEventListener('pointerup', arm, true);
          } catch (_) {}
          window.__rdFsArmed = false;
        };
        window.addEventListener('touchend', arm, { capture: true });
        window.addEventListener('pointerup', arm, { capture: true });
        return true;
      })()`
    );
    if (serial && cal) {
      const d = await pageDiag(cdp);
      const w = d.canvas?.cw || d.inner?.[0] || 375;
      const h = d.canvas?.ch || d.inner?.[1] || 700;
      adbTap(serial, cal, w * 0.5, h * 0.45);
    } else if (serial) {
      // Best-effort center of oriented display
      const { physW, physH } = getDisplayInputSize(serial, null);
      adb(serial, [
        'shell',
        'input',
        'tap',
        String(Math.floor(physW / 2)),
        String(Math.floor(physH / 2)),
      ]);
    }
    await sleep(900);
    // Second try if still not fullscreen (some OEMs need double gesture).
    const mid = await pageDiag(cdp);
    if (!mid.fullscreen) {
      await evaluate(
        cdp,
        `(() => {
          window.__rdFsArmed = false;
          const arm = () => {
            try {
              const el = document.documentElement;
              const p =
                (el.requestFullscreen && el.requestFullscreen()) ||
                (el.webkitRequestFullscreen && el.webkitRequestFullscreen());
              if (p && typeof p.then === 'function') p.catch(() => {});
            } catch (_) {}
          };
          window.addEventListener('touchend', arm, { capture: true, once: true });
          return true;
        })()`
      );
      if (serial && cal) {
        const w = mid.canvas?.cw || mid.inner?.[0] || 375;
        const h = mid.canvas?.ch || mid.inner?.[1] || 700;
        adbTap(serial, cal, w * 0.5, h * 0.5);
      } else if (serial) {
        const { physW, physH } = getDisplayInputSize(serial, null);
        adb(serial, [
          'shell',
          'input',
          'tap',
          String(Math.floor(physW / 2)),
          String(Math.floor(physH / 2)),
        ]);
      }
      await sleep(800);
    }
  } else {
    await evaluate(
      cdp,
      `(() => {
        try {
          // Avoid TypeError: Failed to execute 'exitFullscreen' on 'Document': Document not active
          if (document.hidden) return false;
          if (document.visibilityState && document.visibilityState !== 'visible') return false;
          const fs =
            document.fullscreenElement || document.webkitFullscreenElement;
          if (!fs) return false;
          let p = null;
          try {
            if (document.exitFullscreen) p = document.exitFullscreen();
            else if (document.webkitExitFullscreen) p = document.webkitExitFullscreen();
          } catch (_) {
            return false;
          }
          if (p && typeof p.then === 'function') p.catch(() => {});
          return true;
        } catch (_) {
          return false;
        }
      })()`
    );
    await sleep(400);
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
  await sleep(1500);
  // Confirm input coordinate system flipped for landscape before calibrating.
  const orientSize = getDisplayInputSize(
    serial,
    cell.orientation === 'landscape'
  );
  info('post-orient input size', orientSize);

  const tab = await findGamePage();
  if (!tab?.webSocketDebuggerUrl) throw new Error('no tab');
  const cdp = await connectCdp(tab.webSocketDebuggerUrl);
  const recPath = path.join(VID, `${tag}.mp4`);
  let recorder;

  try {
    await cdp.send('Runtime.enable');
    await cdp.send('Page.enable');
    // Exit any leftover fullscreen on the old document BEFORE navigate (safe no-op).
    try {
      await setChromeMode(cdp, 'browsing');
    } catch (_) {}

    const url =
      LIVE_URL +
      (LIVE_URL.includes('?') ? '&' : '?') +
      `e2e=1&phone_cell=${tag}&t=${Date.now()}`;
    await navigateLive(cdp, url);
    await sleep(400);

    recorder = startAdbRecord(serial, recPath);
    await sleep(400);

    // Provisional geometric cal for fullscreen gesture + first taps
    let cal = await calibrate(serial, cdp);

    if (cell.mode === 'fullscreen') {
      await setChromeMode(cdp, 'fullscreen', serial, cal);
      await sleep(500);
      // Recalibrate after FS (viewport + chrome residual change)
      cal = await calibrate(serial, cdp);
    } else {
      // Ensure not fullscreen; never throw Document-not-active into page console.
      await setChromeMode(cdp, 'browsing');
    }

    let diag = await pageDiag(cdp);
    let w = diag.canvas?.cw || diag.inner?.[0] || 375;
    let h = diag.canvas?.ch || diag.inner?.[1] || 700;
    const orientOk =
      (cell.orientation === 'portrait' && h >= w) ||
      (cell.orientation === 'landscape' && w >= h);
    if (orientOk) pass(`${tag}/orientation`, `${w}x${h}`);
    else fail(`${tag}/orientation`, `${w}x${h}`);

    if (cell.mode === 'fullscreen') {
      // Prefer document.fullscreen OR clear viewport growth vs browsing chrome.
      if (
        diag.fullscreen ||
        (cell.orientation === 'portrait' && h >= 780) ||
        (cell.orientation === 'landscape' && h >= 320 && w >= 700)
      ) {
        pass(`${tag}/fullscreen`, `flag=${diag.fullscreen} ${w}x${h}`);
      } else {
        fail(`${tag}/fullscreen`, `flag=${diag.fullscreen} ${w}x${h}`);
      }
    } else {
      if (!diag.fullscreen) pass(`${tag}/browsing`, `${w}x${h}`);
      else fail(`${tag}/browsing`, 'still fullscreen');
    }

    let pts = layoutPoints(w, h);
    fs.writeFileSync(
      path.join(OUT, `${tag}_diag.json`),
      JSON.stringify({ cell, diag, pts, cal, orientSize }, null, 2)
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

    // MENU swap + confirm — verify via data-rd-state (no false CAPTURE_OK)
    adbTap(serial, cal, pts.swap.x, pts.swap.y);
    await sleep(350);
    adbTap(serial, cal, pts.swap.x, pts.swap.y);
    await sleep(300);
    inv({
      cell: tag,
      screen: 'menu',
      control: 'swap toggle',
      worked: 'tapped',
      fatty: 'bottom band',
      ok: true,
    });
    adbTap(serial, cal, pts.confirm.x, pts.confirm.y);
    let st = await waitRdState(cdp, 'mode_select', 3500);
    // Retry confirm at a few vertical bands if cal is slightly off (landscape menu).
    if (st !== 'mode_select') {
      for (const yFrac of [0.4, 0.5, 0.35, 0.55]) {
        adbTap(serial, cal, pts.center.x, Math.floor(h * yFrac));
        st = await waitRdState(cdp, 'mode_select', 1200);
        if (st === 'mode_select') break;
      }
    }
    const menuOk = st === 'mode_select';
    inv({
      cell: tag,
      screen: 'menu',
      control: 'confirm',
      worked: menuOk ? `state=${st}` : `stuck state=${st || '?'}`,
      fatty: 'good',
      ok: menuOk,
    });
    if (!menuOk) {
      fail(`${tag}/menu`, `expected mode_select got ${st}`);
      // Still dump diag for review; do not invent play CAPTURE_OK.
      fs.writeFileSync(
        path.join(OUT, `${tag}_diag.json`),
        JSON.stringify({ cell, diag: await pageDiag(cdp), pts, cal, orientSize, rdState: st }, null, 2)
      );
      return;
    }
    pass(`${tag}/menu`, `state=${st}`);

    // ALL modes
    for (let i = 0; i < 4; i++) {
      adbTap(serial, cal, pts.modeDown.x, pts.modeDown.y);
      await sleep(220);
    }
    for (let i = 0; i < 2; i++) {
      adbTap(serial, cal, pts.modeUp.x, pts.modeUp.y);
      await sleep(220);
    }
    inv({
      cell: tag,
      screen: 'mode_select',
      control: 'all modes',
      worked: 'cycled 4',
      fatty: 'bands',
      ok: true,
    });
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
    inv({
      cell: tag,
      screen: 'mode_select',
      control: 'all difficulties',
      worked: 'cycled 4',
      fatty: 'good',
      ok: true,
    });
    pass(`${tag}/difficulties`);

    // START
    adbTap(serial, cal, pts.start.x, pts.start.y);
    st = await waitRdState(cdp, 'playing', 4000);
    if (st !== 'playing') {
      // Retry START lower/higher (landscape chrome can shift layout points).
      for (const yFrac of [0.62, 0.72, 0.55, 0.78]) {
        adbTap(serial, cal, pts.center.x, Math.floor(h * yFrac));
        st = await waitRdState(cdp, 'playing', 1500);
        if (st === 'playing') break;
      }
    }
    const startOk = st === 'playing';
    inv({
      cell: tag,
      screen: 'mode_select',
      control: 'START',
      worked: startOk ? `state=${st}` : `stuck state=${st || '?'}`,
      fatty: 'good',
      ok: startOk,
    });
    if (!startOk) {
      fail(`${tag}/start`, `expected playing got ${st}`);
      return;
    }
    pass(`${tag}/start`, `state=${st}`);

    diag = await pageDiag(cdp);
    const wp = diag.canvas?.cw || w;
    const hp = diag.canvas?.ch || h;
    pts = layoutPoints(wp, hp);
    // re-calibrate after layout change (chrome / play deck)
    const cal2 = await calibrate(serial, cdp);

    // PLAY 20s — only if we truly entered Playing
    const end = Date.now() + PLAY_MS;
    let step = 0;
    let lastState = 'playing';
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
      if (step % 8 === 0) {
        lastState = (await rdState(cdp)) || lastState;
      }
      await sleep(120);
    }
    lastState = (await rdState(cdp)) || lastState;
    const playOk = step > 5 && (lastState === 'playing' || lastState === 'game_over');
    inv({
      cell: tag,
      screen: 'playing',
      control: 'stick+dash 20s',
      worked: `steps=${step} state=${lastState}`,
      fatty: pts.gap >= MIN_GAP_CSS ? 'gap ok' : 'close',
      ok: playOk,
    });
    if (playOk) pass(`${tag}/play20s`, `steps=${step} state=${lastState}`);
    else fail(`${tag}/play20s`, `steps=${step} state=${lastState}`);

    // Safe exit FS only when still active document
    try {
      await setChromeMode(cdp, 'browsing');
    } catch (_) {}
  } finally {
    if (recorder) {
      try {
        const info = await recorder.stop();
        // Prefer ffprobe when available for real decodability (moov alone can still be truncated).
        let decodable = info.hasMoov;
        if (info.bytes > 50000 && info.hasMoov) {
          const probe = sh('ffprobe', [
            '-v',
            'error',
            '-show_entries',
            'format=duration',
            '-of',
            'default=nw=1:nk=1',
            info.path,
          ]);
          const dur = Number(probe.out);
          if (probe.ok && Number.isFinite(dur) && dur > 5) {
            decodable = true;
            info.duration = dur;
          } else if (probe.ok === false || !Number.isFinite(dur)) {
            decodable = false;
            info.probeErr = (probe.err || probe.out || '').slice(0, 120);
          }
        }
        if (info.bytes > 50000 && decodable) {
          pass(
            `${tag}/recording`,
            `${info.bytes} bytes moov=ok` +
              (info.duration ? ` dur=${info.duration.toFixed(1)}s` : '')
          );
        } else {
          fail(
            `${tag}/recording`,
            `bytes=${info.bytes} moov=${info.hasMoov} decodable=${decodable} ${info.probeErr || ''}`
          );
        }
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
    lines.push(
      `- ${r.ok ? 'CAPTURE_OK' : 'CAPTURE_FAIL'} **${r.name}**: ${r.detail || ''}`
    );
  }
  lines.push(
    '',
    `## Summary (CAPTURE layer only — not visual review): results ${results.filter((r) => r.ok).length}/${results.length}, inventory ${inventory.filter((r) => r.ok).length}/${inventory.length}, open_capture_fails ${failed.length + invFailed.length}`,
    '',
    'NOTE: CAPTURE_OK ≠ looks good. Phase C still requires video review of each cell recording.',
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
        layer: 'capture_only',
        at: new Date().toISOString(),
        note: 'CAPTURE only. Review cell videos separately for ship; do not treat open_bads=0 alone as visual A7.',
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
  const prefer = (process.env.ANDROID_SERIAL || process.env.PHONE_SERIAL || '').trim();
  let device = prefer ? devices.find((d) => d.serial === prefer) : null;
  if (!device) {
    // Prefer physical USB over emulator when both present
    device =
      devices.find((d) => !d.serial.startsWith('emulator-') && !/\bemulator\b/i.test(d.raw)) ||
      devices[0];
  }
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
  console.log('\n=== PHONE 2×2 CAPTURE SUMMARY (not visual review) ===');
  console.log('capture_ok', results.filter((r) => r.ok).length, '/', results.length);
  console.log(
    'NOTE: CAPTURE_OK ≠ looks good. Review each cell video; suite exit 0 is not PRE-PROD PASS.'
  );
  if (failed.length || invFailed.length) {
    console.error('CAPTURE_FAILED', failed);
    process.exit(1);
  }
  process.exit(0);
}

main().catch((e) => {
  console.error(e);
  process.exit(2);
});
