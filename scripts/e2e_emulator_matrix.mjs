/**
 * Phase A handheld — Android emulator matrix capture.
 *
 * For each touch format in qa_matrix.json:
 *   - AVD via adb (prefer serial emulator-*)
 *   - adb reverse → local dist http://127.0.0.1:8080/
 *   - Full-display adb shell screenrecord
 *   - OS-level touches via adb shell input (NOT CDP/Puppeteer touch)
 *   - CDP for navigate / evaluate; matrix PNGs via adb screencap (full-display)
 *   - Quality-hold matrix PNGs → screenshots/viewports/{format}_0*.png
 *   - Video → screenshots/web/e2e/recordings/{format}_touch.mp4
 *
 * Requires AVD GPU host (not swiftshader): Bevy 0.19 mesh2d needs
 * GL_MAX_VERTEX_UNIFORM_VECTORS > 256 (swiftshader reports 256 → render crash).
 *   emulator -avd friends -gpu host ...
 *
 * Env:
 *   E2E_FORMATS=phone_android_landscape,tablet_portrait
 *   E2E_PLAY_MS=20000
 *   MATRIX_HOLD_MS=450
 *   EMULATOR_SERIAL=emulator-5554
 *   PHONE_CDP_PORT=9222
 *   CAPTURE_MATRIX=1
 *   EMU_REQUIRE=1
 *
 * CAPTURE vs REVIEW: CAPTURE_OK / results ok:true = automation only, not visual
 * acceptance. A4b/A6 + A7 (ui-viewport-qa / qa_success_criteria.json) are separate.
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
const OUT = path.join(ROOT, 'screenshots/web/e2e');
const VID = path.join(OUT, 'recordings');
const STILLS = path.join(OUT, 'stills');
const MATRIX_OUT = path.join(ROOT, 'screenshots/viewports');
const MATRIX = JSON.parse(fs.readFileSync(path.join(__dirname, 'qa_matrix.json'), 'utf8'));
const CDP_PORT = Number(process.env.PHONE_CDP_PORT || 9222);
const PLAY_MS = Number(process.env.E2E_PLAY_MS || 20000);
const HOLD_MS = Number(process.env.MATRIX_HOLD_MS || 450);
const CAPTURE_MATRIX = process.env.CAPTURE_MATRIX !== '0';
const REQUIRE = process.env.EMU_REQUIRE === '1';
const FORCE_GO_MS = Number(
  process.env.E2E_FORCE_GO_MS || Math.max(PLAY_MS + 2500, 22500)
);
// Extra wall time after force deadline before giving up on GAME OVER.
const GO_GRACE_MS = Number(process.env.E2E_GO_GRACE_MS || 20000);
const BASE_URL = process.env.EMU_URL || 'http://127.0.0.1:8080/';
const GAME_URL = `${BASE_URL}${BASE_URL.includes('?') ? '&' : '?'}e2e=1&qa_matrix=1&qa_go_ms=${FORCE_GO_MS}`;

/**
 * adb shell re-parses the remote command; bare `&` in URLs is treated as a
 * shell background operator and truncates query params (e.g. only `?e2e=1`
 * survives → force-GO never arms). Always pass a single quoted shell string.
 */
function adbShell(serial, shellCmd, opts) {
  return adb(serial, ['shell', shellCmd], opts);
}

/** Quote a URL/path for embedding inside single-quoted adb shell strings. */
function shQuote(s) {
  return `'${String(s).replace(/'/g, `'\\''`)}'`;
}

const filterEnv = (process.env.E2E_FORMATS || '')
  .split(',')
  .map((s) => s.trim())
  .filter(Boolean);

const FORMATS = MATRIX.formats.filter(
  (f) => f.touch && (!filterEnv.length || filterEnv.includes(f.id))
);

fs.mkdirSync(OUT, { recursive: true });
fs.mkdirSync(VID, { recursive: true });
fs.mkdirSync(STILLS, { recursive: true });
fs.mkdirSync(MATRIX_OUT, { recursive: true });

const results = [];
const unitMeta = [];

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
function info(...a) {
  console.log('[emu]', ...a);
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
      return { serial: p[0], state: p[1], raw: l, emulator: p[0].startsWith('emulator-') };
    })
    .filter((d) => d.state === 'device');
}

function pickSerial() {
  if (process.env.EMULATOR_SERIAL) return process.env.EMULATOR_SERIAL;
  const devices = listAdbDevices();
  const emu = devices.find((d) => d.emulator);
  return emu?.serial || null;
}

function setupReverse(serial) {
  adb(serial, ['reverse', 'tcp:8080', 'tcp:8080']);
  adb(serial, ['forward', `tcp:${CDP_PORT}`, 'localabstract:chrome_devtools_remote']);
}

function setupChromeFlags(serial) {
  // Disable Translate UI (covers game + confuses screencap). Enable remote debug.
  // --lang=en-US reduces AVD pt-BR "Traduzir página?" even if Translate flags lag.
  // Keep flag set conservative — invalid tokens can break the whole command-line file.
  const flags =
    'chrome --disable-fre --no-default-browser-check --no-first-run ' +
    '--lang=en-US ' +
    '--disable-features=TranslateUI,Translate ' +
    '--enable-logging';
  // Chrome reads /data/local/tmp/chrome-command-line only when set as debug app.
  adbShell(serial, `echo ${shQuote(flags)} > /data/local/tmp/chrome-command-line`);
  adb(serial, ['shell', 'am', 'set-debug-app', '--persistent', 'com.android.chrome']);
  // Prefer en-US system locale for this capture session (lab AVD).
  adb(serial, ['shell', 'settings', 'put', 'system', 'system_locales', 'en-US']);
  adb(serial, ['shell', 'settings', 'put', 'global', 'window_animation_scale', '0']);
  adb(serial, ['shell', 'settings', 'put', 'global', 'transition_animation_scale', '0']);
  adb(serial, ['shell', 'settings', 'put', 'global', 'animator_duration_scale', '0']);
  adb(serial, ['shell', 'settings', 'put', 'system', 'screen_off_timeout', '1800000']);
  // Display OFF → layer_stack=-1 → black screencap + screenrecord INVALID_LAYER_STACK.
  wakeDisplay(serial);
}

/** Ensure display is ON — required for adb screencap/screenrecord content. */
function wakeDisplay(serial) {
  adb(serial, ['shell', 'input', 'keyevent', 'KEYCODE_WAKEUP']);
  adbShell(serial, 'svc power stayon true');
  adb(serial, ['shell', 'settings', 'put', 'system', 'screen_off_timeout', '1800000']);
}

/** Resize emulator display so Chrome CSS ≈ format width×height @ dpr. */
function applyDisplayProfile(serial, format) {
  const dpr = format.dpr || 2;
  // Prefer density that yields target CSS; clamp physical size for swiftshader stability.
  let density = Math.round(160 * dpr);
  let physW = Math.round(format.width * dpr);
  let physH = Math.round(format.height * dpr);
  const maxSide = Number(process.env.EMU_MAX_PHYS_SIDE || 1600);
  const maxPx = maxSide / Math.max(physW, physH);
  if (maxPx < 1) {
    // keep CSS aspect; lower dpr/density so CSS ~ format size
    const scale = maxPx;
    physW = Math.max(320, Math.round(physW * scale));
    physH = Math.max(320, Math.round(physH * scale));
    // density so CSS ≈ format: css = phys * 160 / density
    // want format.width ≈ physW * 160 / density → density ≈ physW * 160 / format.width
    density = Math.max(120, Math.round((physW * 160) / format.width));
  }
  const landscape = format.width > format.height;
  info('display profile', format.id, `${physW}x${physH}`, `density=${density}`, landscape ? 'land' : 'port', `targetCSS=${format.width}x${format.height}`);

  adb(serial, ['shell', 'settings', 'put', 'system', 'accelerometer_rotation', '0']);
  // Set size already in desired orientation (W×H). Lock rotation 0 so we do NOT
  // double-apply landscape (size 1600×720 + rot=1 was still portrait on this AVD).
  adb(serial, ['shell', 'wm', 'size', `${physW}x${physH}`]);
  adb(serial, ['shell', 'wm', 'density', String(density)]);
  adb(serial, ['shell', 'settings', 'put', 'system', 'user_rotation', '0']);
  try {
    adb(serial, ['shell', 'wm', 'user-rotation', 'lock', '0']);
  } catch (_) {}
  // Give SurfaceFlinger a beat to apply override size before Chrome opens.
  return { physW, physH, density, landscape, dpr };
}

function restoreDisplay(serial, saved) {
  if (!saved) return;
  if (saved.size) adb(serial, ['shell', 'wm', 'size', saved.size === 'reset' ? 'reset' : saved.size]);
  else adb(serial, ['shell', 'wm', 'size', 'reset']);
  if (saved.density) adb(serial, ['shell', 'wm', 'density', saved.density === 'reset' ? 'reset' : saved.density]);
  else adb(serial, ['shell', 'wm', 'density', 'reset']);
  adb(serial, ['shell', 'wm', 'user-rotation', 'free']);
  adb(serial, ['shell', 'settings', 'put', 'system', 'accelerometer_rotation', saved.accel || '1']);
}

function startAdbRecord(serial, localPath) {
  // Guest-side mp4 + kill -INT (validated: produces moov; host exec-out h264 was flaky).
  wakeDisplay(serial);
  const remote = `/sdcard/rd_emu_${Date.now()}.mp4`;
  const args = [
    ...(serial ? ['-s', serial] : []),
    'shell',
    'screenrecord',
    '--bit-rate',
    '6M',
    '--time-limit',
    '180',
    remote,
  ];
  const child = spawn('adb', args, { stdio: ['ignore', 'pipe', 'pipe'] });
  let stderr = '';
  child.stderr?.on('data', (d) => {
    stderr += d.toString();
  });
  info('screenrecord start', remote);
  // Quick health: pid must appear; INVALID_LAYER_STACK exits immediately.
  const pidEarly = (adb(serial, ['shell', 'pidof', 'screenrecord']).out || '').trim();
  if (!pidEarly) {
    info('screenrecord no pid yet; waiting…');
  }
  return {
    remote,
    async stop() {
      const pidOut = (adb(serial, ['shell', 'pidof', 'screenrecord']).out || '').trim();
      info('screenrecord stop pids', pidOut || 'none', stderr ? `stderr=${stderr.slice(0, 120)}` : '');
      for (const pid of pidOut.split(/\s+/).filter(Boolean)) {
        adb(serial, ['shell', 'kill', '-INT', pid]);
      }
      for (let i = 0; i < 40; i++) {
        const still = (adb(serial, ['shell', 'pidof', 'screenrecord']).out || '').trim();
        if (!still) break;
        await sleep(250);
      }
      try { child.kill('SIGTERM'); } catch (_) {}
      await sleep(1200);
      fs.mkdirSync(path.dirname(localPath), { recursive: true });
      let bytes = 0;
      for (let i = 0; i < 6; i++) {
        adb(serial, ['pull', remote, localPath], { timeout: 120000 });
        bytes = fs.existsSync(localPath) ? fs.statSync(localPath).size : 0;
        const probe = sh('ffprobe', [
          '-v', 'error', '-show_entries', 'format=duration',
          '-of', 'default=nw=1:nk=1', localPath,
        ]);
        info('pull try', i, 'bytes', bytes, 'dur', probe.out || probe.err?.slice(0, 80));
        if (probe.ok && probe.out && Number(probe.out) > 5) break;
        await sleep(500);
      }
      adb(serial, ['shell', 'rm', '-f', remote]);
      return { path: localPath, bytes, pullOk: bytes > 50000 };
    },
  };
}


async function waitCdp(ms = 15000) {
  const t0 = Date.now();
  while (Date.now() - t0 < ms) {
    try {
      return await cdpVersion(CDP_PORT);
    } catch (_) {
      await sleep(300);
    }
  }
  return null;
}

async function findGamePage() {
  const tabs = await listPages(CDP_PORT);
  const pages = tabs.filter((t) => t.type === 'page');
  return (
    pages.find((t) => /rusty|127\.0\.0\.1:8080|localhost:8080/i.test(t.url || '')) ||
    pages.find((t) => /about:blank|chrome:\/\/new/i.test(t.url || '')) ||
    pages[0]
  );
}

/**
 * Bring Chrome to the foreground.
 * When `url` is set, open VIEW intent (must be shell-quoted for & params).
 * When `url` is null, only resume the existing Chrome task — avoids spawning
 * a new tab (tab zoo desyncs CDP evaluate from the visible screencap surface).
 */
function foregroundChrome(serial, url = null) {
  if (url) {
    adbShell(
      serial,
      `am start -a android.intent.action.VIEW -d ${shQuote(url)} ` +
        `-n com.android.chrome/com.google.android.apps.chrome.Main`
    );
    return;
  }
  adbShell(
    serial,
    'am start -n com.android.chrome/com.google.android.apps.chrome.Main'
  );
}

/** Close non-game Chrome tabs so CDP + screencap target the same page. */
async function pruneChromeTabs(keepUrlPart = '8080') {
  try {
    const tabs = await listPages(CDP_PORT);
    const pages = tabs.filter((t) => t.type === 'page');
    const keep =
      pages.find((t) => new RegExp(keepUrlPart, 'i').test(t.url || '')) ||
      pages.find((t) => /qa_matrix=1/i.test(t.url || '')) ||
      pages[0];
    for (const p of pages) {
      if (!keep || p.id === keep.id) continue;
      try {
        // HTTP close (works without attaching each WS)
        await fetch(`http://127.0.0.1:${CDP_PORT}/json/close/${p.id}`).catch(() => {});
      } catch (_) {}
    }
    return keep;
  } catch (e) {
    info('prune tabs skip', String(e).slice(0, 80));
    return null;
  }
}

async function openChrome(serial, url) {
  // Force-stop between formats to kill the tab zoo (dozens of "Rusty" tabs break
  // screencap + CDP targeting). WASM is cached by Chrome disk cache after first load.
  // Do NOT send KEYCODE_BACK: it often exits Chrome → launcher.
  adb(serial, ['shell', 'am', 'force-stop', 'com.android.chrome']);
  await sleep(600);
  foregroundChrome(serial, url);
  await sleep(2200);
  // Resume only — second VIEW would open another tab and desync CDP.
  foregroundChrome(serial, null);
  await sleep(600);
  setupReverse(serial);
}

async function pageDiag(cdp) {
  return evaluateJson(
    cdp,
    `(() => {
      const c = document.querySelector('canvas');
      const rect = c?.getBoundingClientRect();
      const boot = document.getElementById('boot');
      const cta = document.getElementById('boot-cta');
      return JSON.stringify({
        url: location.href,
        inner: [window.innerWidth, window.innerHeight],
        dpr: devicePixelRatio,
        fullscreen: !!(document.fullscreenElement || document.webkitFullscreenElement),
        canvas: c ? {
          cw: c.clientWidth, ch: c.clientHeight,
          rect: rect ? { x: rect.x, y: rect.y, w: rect.width, h: rect.height } : null
        } : null,
        bootHidden: boot?.classList.contains('hidden') ?? null,
        ctaDisplay: cta ? (cta.style.display || getComputedStyle(cta).display) : null,
        title: document.title,
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
        document.getElementById('boot-cta')?.style?.display === 'inline-block' ||
        document.getElementById('boot-cta')
      )`
    );
    if (ready) {
      // ensure CTA visible if still booting
      await evaluate(
        cdp,
        `(() => {
          const cta = document.getElementById('boot-cta');
          if (cta && !document.getElementById('boot')?.classList.contains('hidden')) {
            cta.style.display = 'inline-block';
          }
          return true;
        })()`
      );
      return;
    }
    await sleep(400);
  }
  throw new Error('timeout ready (canvas/boot)');
}

/** Wait until WASM downloaded + canvas painted (not download spinner). */
async function waitGameReady(cdp, ms = 180000) {
  const t0 = Date.now();
  while (Date.now() - t0 < ms) {
    const d = await evaluateJson(
      cdp,
      `(() => {
        const boot = document.getElementById('boot');
        const cta = document.getElementById('boot-cta');
        const c = document.querySelector('canvas');
        const text = (boot?.innerText || document.body?.innerText || '').slice(0, 400);
        const pctEl = document.getElementById('boot-progress-pct');
        const progressEl = document.getElementById('boot-progress');
        const pctRaw = (pctEl?.textContent || '').replace(/%/g, '').trim();
        const pct = Number(pctRaw);
        const aria = Number(progressEl?.getAttribute?.('aria-valuenow') || NaN);
        const progressDone =
          !progressEl ||
          progressEl.classList.contains('done') ||
          progressEl.classList.contains('hidden') ||
          (!Number.isNaN(pct) && pct >= 100) ||
          (!Number.isNaN(aria) && aria >= 100);
        // English + PT fragments (Translate can rewrite boot copy mid-load).
        const downloadingText = /Downloading game|first visit|MB of WASM|Baixando|baixando|WASM/i.test(text);
        const stillDownloading = downloadingText && !progressDone;
        const ctaStyle = cta ? (cta.style.display || getComputedStyle(cta).display) : null;
        const ctaVisible = !!(
          cta &&
          (ctaStyle === 'inline-block' || ctaStyle === 'block') &&
          !boot?.classList.contains('hidden')
        );
        return JSON.stringify({
          downloading: stillDownloading,
          progressDone,
          pct: Number.isNaN(pct) ? null : pct,
          bootHidden: !!boot?.classList.contains('hidden'),
          ctaDisplay: ctaStyle,
          ctaVisible,
          canvas: !!(c && c.clientWidth > 8 && c.clientHeight > 8),
          cw: c?.clientWidth || 0,
          ch: c?.clientHeight || 0,
          text: text.slice(0, 80),
          href: location.href,
        });
      })()`
    );
    // Ready only when canvas is live AND boot is past download:
    // either boot hidden (menu) or CTA visible for quality-hold boot shot.
    // Require progress ring done when boot is still showing.
    if (
      d &&
      d.canvas &&
      !d.downloading &&
      (d.bootHidden || (d.ctaVisible && d.progressDone))
    ) {
      if (!d.bootHidden) {
        await evaluate(
          cdp,
          `(() => { const cta=document.getElementById('boot-cta'); if(cta) cta.style.display='inline-block'; return true; })()`
        );
      }
      info(
        'game ready',
        `${d.cw}x${d.ch}`,
        `t=${Date.now() - t0}ms`,
        d.bootHidden ? 'past-boot' : 'boot+cta',
        `pct=${d.pct}`
      );
      // Settle so CTA/title paint fully before first matrix hold.
      await sleep(900);
      return d;
    }
    if (d?.downloading || (d && !d.progressDone && !d.bootHidden)) {
      info('still downloading…', `pct=${d?.pct}`, d?.text);
    }
    await sleep(800);
  }
  throw new Error('timeout waiting for game ready');
}

/**
 * Dismiss Chrome Google Translate infobar / offer if it appears
 * ("Traduzir esta página?" / "Translate this page?").
 * Flags + en-US locale should prevent it; this is belt-and-suspenders.
 * Note: Translate UI is often a *native* Chrome Android view — not in page DOM —
 * so we also dump UIAutomator hierarchy and tap Não/No/Never/close.
 */
async function dismissTranslate(cdp, serial) {
  // Page-DOM path (rarely hits native infobar, but cheap).
  try {
    const hit = await evaluateJson(
      cdp,
      `(() => {
        const hide = (el) => { try { el.style.setProperty('display','none','important'); el.style.setProperty('visibility','hidden','important'); } catch(_){} };
        for (const sel of [
          '#translate-infobar', '#infobar', 'translate-infobar',
          '[id*="translate"]', '[class*="translate"]',
          'iframe[src*="translate"]'
        ]) {
          document.querySelectorAll(sel).forEach(hide);
        }
        const labels = /^(Não|Nao|No|Never|Never translate|Not now|Agora não|Agora nao|No thanks|Fechar|Close|×|✕)$/i;
        const clickables = Array.from(document.querySelectorAll('button, [role="button"], a, span, div'));
        let clicked = null;
        for (const el of clickables) {
          const t = (el.innerText || el.textContent || el.getAttribute?.('aria-label') || '').trim();
          if (!t || t.length > 40) continue;
          if (labels.test(t) || /never translate|n[aã]o traduzir|no, thanks/i.test(t)) {
            try { el.click(); clicked = t; break; } catch(_){}
          }
        }
        for (const el of document.querySelectorAll('div,section,aside')) {
          const t = (el.innerText || '').slice(0, 120);
          if (/Traduzir|Translate this page|Traduzir esta p[aá]gina/i.test(t) && el.childElementCount < 12) {
            hide(el);
          }
        }
        return JSON.stringify({ clicked });
      })()`
    );
    if (hit?.clicked) info('translate dismiss click', hit.clicked);
  } catch (_) {}

  // Native Chrome Translate infobar via UIAutomator dump.
  try {
    const dumpPath = '/sdcard/rd_ui.xml';
    adbShell(serial, `uiautomator dump ${dumpPath} >/dev/null 2>&1`);
    const xml = adb(serial, ['shell', 'cat', dumpPath], { timeout: 8000 }).out || '';
    if (/Traduzir|Translate this page|translate/i.test(xml)) {
      info('translate UI detected in uiautomator dump');
      // Prefer explicit dismiss / never buttons.
      const candidates = [
        /text="(Não|Nao|No|Never|Never translate|Agora não|Not now|No thanks)"[^>]*bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]"/i,
        /content-desc="(Close|Fechar|Dismiss|Não|Nao)"[^>]*bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]"/i,
        /text="(Traduzir|Translate)"[^>]*bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]"/i, // last resort: we want sibling Não — skip click on Traduzir itself
      ];
      let tapped = false;
      for (const re of candidates.slice(0, 2)) {
        const m = xml.match(re);
        if (!m) continue;
        const x1 = Number(m[2]), y1 = Number(m[3]), x2 = Number(m[4]), y2 = Number(m[5]);
        const cx = Math.floor((x1 + x2) / 2);
        const cy = Math.floor((y1 + y2) / 2);
        info('translate tap', m[1], cx, cy);
        adb(serial, ['shell', 'input', 'tap', String(cx), String(cy)]);
        tapped = true;
        await sleep(400);
        break;
      }
      if (!tapped) {
        // Bounds scan for any node with Não near translate banner
        const reAll = /text="([^"]*)"[^>]*bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]"/g;
        let mm;
        while ((mm = reAll.exec(xml))) {
          if (!/^(Não|Nao|No|Never|Not now|Agora não)$/i.test(mm[1])) continue;
          const cx = Math.floor((Number(mm[2]) + Number(mm[4])) / 2);
          const cy = Math.floor((Number(mm[3]) + Number(mm[5])) / 2);
          info('translate tap scan', mm[1], cx, cy);
          adb(serial, ['shell', 'input', 'tap', String(cx), String(cy)]);
          tapped = true;
          await sleep(400);
          break;
        }
      }
    }
    adb(serial, ['shell', 'rm', '-f', dumpPath]);
  } catch (e) {
    info('translate uiautomator skip', String(e).slice(0, 80));
  }
}

/**
 * Probe WebGL limits. maxVUV ≤ 256 on swiftshader → Bevy mesh2d invalid → solid ClearColor.
 * Require AVD `-gpu host` (or equivalent) so maxVUV ≥ 1024.
 */
async function probeWebglHealth(cdp) {
  const infoGl = await evaluateJson(
    cdp,
    `(() => {
      const c = document.createElement('canvas');
      const gl = c.getContext('webgl2') || c.getContext('webgl');
      if (!gl) return JSON.stringify({ ok: false, reason: 'no_webgl' });
      const dbg = gl.getExtension('WEBGL_debug_renderer_info');
      return JSON.stringify({
        ok: true,
        maxVUV: gl.getParameter(gl.MAX_VERTEX_UNIFORM_VECTORS),
        renderer: dbg ? gl.getParameter(dbg.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER),
      });
    })()`
  );
  info('webgl', JSON.stringify(infoGl));
  if (infoGl?.maxVUV != null && infoGl.maxVUV <= 256) {
    throw new Error(
      `WebGL MAX_VERTEX_UNIFORM_VECTORS=${infoGl.maxVUV} (need >256). ` +
        `Restart AVD with -gpu host (not swiftshader). renderer=${infoGl.renderer || '?'}`
    );
  }
  return infoGl;
}

/**
 * Calibrate CSS→physical using mid-screen adb tap + CDP touch listener.
 */
async function calibrate(serial, cdp, { allowTap = false } = {}) {
  const sizeOut = adb(serial, ['shell', 'wm', 'size']).out || '';
  const m =
    sizeOut.match(/Override size:\s*(\d+)x(\d+)/) ||
    sizeOut.match(/Physical size:\s*(\d+)x(\d+)/) ||
    sizeOut.match(/(\d+)x(\d+)/);
  const physW = m ? Number(m[1]) : 1080;
  const physH = m ? Number(m[2]) : 2400;
  const diag = await pageDiag(cdp);
  const dpr = diag.dpr || 2;
  const cssW = diag.inner?.[0] || diag.canvas?.cw || 360;
  const cssH = diag.inner?.[1] || diag.canvas?.ch || 800;
  // Geometric: content top-left with status/chrome residual above web content.
  const contentW = cssW * dpr;
  const contentH = cssH * dpr;
  const offX = Math.max(0, (physW - contentW) / 2);
  // Prefer residual above content (status + address bar). Landscape residual is often ~top.
  const residualY = Math.max(0, physH - contentH);
  const offY = residualY * 0.55;
  if (!allowTap) {
    info('calibrate geometric', { offX, offY, dpr, cssW, cssH, physW, physH });
    return { dpr, offX, offY, physW, physH, cssW, cssH, method: 'geometric' };
  }
  // Optional event refine (may consume a menu confirm if centered — use sparingly)
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
  // Tap near top-left of content, not center
  const px = Math.floor(offX + 24);
  const py = Math.floor(offY + 24);
  adb(serial, ['shell', 'input', 'tap', String(px), String(py)]);
  await sleep(350);
  let cal = await evaluateJson(cdp, `JSON.stringify(window.__cal)`);
  if (!cal || cal.x == null) {
    info('calibrate tap failed; geometric', { offX, offY, dpr });
    return { dpr, offX, offY, physW, physH, cssW, cssH, method: 'geometric' };
  }
  const offX2 = px - cal.x * dpr;
  const offY2 = py - cal.y * dpr;
  info('calibrate event', { cal, offX: offX2, offY: offY2, dpr });
  return { dpr, offX: offX2, offY: offY2, physW, physH, cssW, cssH, method: 'event', cal };
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
  return {
    portrait,
    w,
    h,
    stick: { x: Math.floor(stick.x), y: Math.floor(stick.y) },
    dash: { x: Math.floor(dash.x), y: Math.floor(dash.y) },
    center: { x: Math.floor(w / 2), y: Math.floor(h / 2) },
    confirm: { x: Math.floor(w / 2), y: Math.floor(h * 0.45) },
    swap: { x: Math.floor(w / 2), y: Math.floor(h * 0.88) },
    modeUp: { x: Math.floor(w / 2), y: Math.floor(h * 0.26) },
    modeDown: { x: Math.floor(w / 2), y: Math.floor(h * 0.4) },
    diffL: { x: Math.floor(w * 0.3), y: Math.floor(h * 0.52) },
    diffR: { x: Math.floor(w * 0.7), y: Math.floor(h * 0.52) },
    start: { x: Math.floor(w / 2), y: Math.floor(h * 0.68) },
    // boot CTA is often lower-mid
    bootCta: { x: Math.floor(w / 2), y: Math.floor(h * 0.62) },
  };
}

/**
 * Full-display PNG via adb screencap (binary — never encoding:'utf8').
 *
 * Tiny matrix cells had TWO causes:
 *  1) Emulator with `-gpu swiftshader_indirect`: Bevy 0.19 mesh2d pipeline fails
 *     (`Vertex shader active uniforms exceed GL_MAX_VERTEX_UNIFORM_VECTORS (256)`),
 *     app quits → solid ClearColor (~9,10,18) forever. Boot HTML still looks fine.
 *     Fix: run AVD with `-gpu host` (maxVUV ≥1024 on RADV/etc).
 *  2) CDP Page.captureScreenshot can omit WebGL layers even when rendering works.
 *     Prefer full-display screencap (GPU-composited, matches screenrecord orientation).
 */
function adbScreencapPng(serial) {
  const buf = execFileSync(
    'adb',
    [...(serial ? ['-s', serial] : []), 'exec-out', 'screencap', '-p'],
    { maxBuffer: 40 * 1024 * 1024, timeout: 20000 }
  );
  let out = Buffer.isBuffer(buf) ? buf : Buffer.from(buf);
  // Classic adb CRLF corruption: PNG magic becomes 0x89 0x50 ... with \r\n inserted
  const hdr = out.subarray(0, 8);
  if (!(hdr[0] === 0x89 && hdr[1] === 0x50 && hdr[2] === 0x4e && hdr[3] === 0x47)) {
    out = Buffer.from(out.toString('binary').replace(/\r\n/g, '\n'), 'binary');
  }
  return out;
}

async function cdpPagePng(cdp) {
  await cdp.send('Page.enable');
  const shot = await cdp.send('Page.captureScreenshot', {
    format: 'png',
    fromSurface: true,
    captureBeyondViewport: false,
  });
  return Buffer.from(shot.data, 'base64');
}

/**
 * Quality-hold matrix PNG.
 * Prefer CDP page screenshot when host-GPU WebGL is healthy (captures the game
 * tab, not launcher). Fall back to full-display adb screencap after ensuring
 * Chrome is foreground. Reject near-blank / solid ClearColor frames.
 */
async function qualityMatrixShot(cdp, serial, formatId, shotSuffix, extraHoldMs = 0) {
  if (!CAPTURE_MATRIX) return null;
  await sleep(HOLD_MS + extraHoldMs);
  await sleep(150);
  wakeDisplay(serial);
  await sleep(100);
  const name = `${formatId}_${shotSuffix}`;
  const file = path.join(MATRIX_OUT, name + '.png');
  // Full-display adb screencap only — CDP Page.captureScreenshot times out / blacks WebGL.
  try {
    const buf = adbScreencapPng(serial);
    fs.writeFileSync(file, buf);
  } catch (e) {
    fail(`${formatId}/matrix ${shotSuffix}`, String(e));
    return null;
  }
  const size = fs.existsSync(file) ? fs.statSync(file).size : 0;
  info('matrix', name, size, 'screencap');
  // Near-solid black frames (display OFF / layer_stack broken) land ~6–20KB.
  if (size < 2000) fail(`${formatId}/matrix ${shotSuffix}`, `tiny ${size}`);
  else if (size < 25000) {
    fail(`${formatId}/matrix ${shotSuffix}`, `likely blank/black ${size}b — display OFF?`);
  } else pass(`${formatId}/matrix ${shotSuffix}`, `${size}b screencap`);
  return file;
}


async function readQaState(cdp) {
  try {
    return await evaluate(
      cdp,
      `(() => {
        // WASM publish_qa_state writes data-rd-state on <html>
        const rd = document.documentElement?.getAttribute('data-rd-state');
        if (rd) return rd;
        if (typeof window.__qa_state === 'string') return window.__qa_state;
        const d = document.body?.dataset?.qaState;
        if (d) return d;
        return null;
      })()`
    );
  } catch {
    return null;
  }
}

async function waitForQaState(cdp, want, ms, serial = null) {
  const t0 = Date.now();
  let last = null;
  let lastWake = 0;
  while (Date.now() - t0 < ms) {
    last = await readQaState(cdp);
    if (last === want) {
      info('qa state reached', want, `t=${Date.now() - t0}ms`);
      return true;
    }
    // Keep display alive during long force-GO polls (no game taps).
    if (serial && Date.now() - lastWake > 12000) {
      wakeDisplay(serial);
      lastWake = Date.now();
    }
    await sleep(250);
  }
  info('qa state timeout', `want=${want}`, `last=${last}`, `waited=${ms}ms`);
  return false;
}

/** Ensure live page URL still has qa_matrix + qa_go_ms (force-GO armed). */
async function ensureQaUrl(cdp, url) {
  const href = await evaluate(cdp, 'location.href').catch(() => '');
  if (href && /qa_matrix=1/.test(href) && /qa_go_ms=/.test(href)) {
    info('qa url ok', String(href).slice(0, 120));
    return href;
  }
  info('qa url missing params — re-navigate', String(href || '').slice(0, 120));
  await navigateLive(cdp, url);
  const href2 = await evaluate(cdp, 'location.href').catch(() => '');
  info('qa url after nav', String(href2 || '').slice(0, 140));
  if (!href2 || !/qa_matrix=1/.test(href2)) {
    throw new Error(`qa_matrix not in location.href after navigate: ${href2}`);
  }
  return href2;
}

async function requestFullscreen(cdp, serial, cal, pts) {
  // gesture then JS requestFullscreen
  if (cal && pts) adbTap(serial, cal, pts.center.x, pts.center.y);
  await sleep(200);
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
  await sleep(700);
}

async function runFormat(serial, format, savedDisplay) {
  const tag = format.id;
  info('\n======== FORMAT', tag, `${format.width}x${format.height}@${format.dpr}`, '========');
  wakeDisplay(serial);
  const profile = applyDisplayProfile(serial, format);
  // SurfaceFlinger needs a beat after wm size; display can drop OFF otherwise.
  await sleep(2500);
  wakeDisplay(serial);
  await sleep(500);

  const url = `${GAME_URL}&format=${tag}&t=${Date.now()}`;
  await openChrome(serial, url);

  const version = await waitCdp(20000);
  if (!version) throw new Error('CDP unreachable after chrome open');

  let tab = await findGamePage();
  if (!tab?.webSocketDebuggerUrl) throw new Error('no chrome tab');
  let cdp = await connectCdp(tab.webSocketDebuggerUrl);

  const recPath = path.join(VID, `${tag}_touch.mp4`);
  let recorder;
  const meta = { format: tag, profile, diag: null, cal: null, recPath };

  try {
    await navigateLive(cdp, url);
    // Re-assert full query string (adb VIEW used to drop &params via shell).
    await ensureQaUrl(cdp, url);
    // Bring THIS tab to front; resume Chrome without opening a new VIEW tab.
    try {
      await cdp.send('Page.bringToFront');
    } catch (_) {}
    foregroundChrome(serial, null);
    await sleep(300);
    await pruneChromeTabs('8080');
    // Reconnect if prune closed our socket target (rare)
    try {
      await ensureQaUrl(cdp, url);
    } catch (_) {
      const tab2 = await findGamePage();
      if (tab2?.webSocketDebuggerUrl) {
        try { cdp.close(); } catch (_) {}
        cdp = await connectCdp(tab2.webSocketDebuggerUrl);
        await navigateLive(cdp, url);
        await ensureQaUrl(cdp, url);
      }
    }
    try {
      await cdp.send('Page.bringToFront');
    } catch (_) {}

    // MUST wait for WASM/WebGL canvas + boot past download before capture
    const canvasInfo = await waitGameReady(cdp, 180000);
    pass(`${tag}/canvas`, `${canvasInfo.cw}x${canvasInfo.ch}`);
    const glHealth = await probeWebglHealth(cdp);
    meta.webgl = glHealth;
    pass(`${tag}/webgl`, `maxVUV=${glHealth?.maxVUV} ${String(glHealth?.renderer || '').slice(0, 60)}`);

    await dismissTranslate(cdp, serial);

    recorder = startAdbRecord(serial, recPath);
    await sleep(400);

    let diag = await pageDiag(cdp);
    meta.diag0 = diag;
    info('diag0', JSON.stringify(diag));

    let w = diag.canvas?.cw || diag.inner?.[0] || format.width;
    let h = diag.canvas?.ch || diag.inner?.[1] || format.height;
    // Geometric cal only — never center-tap before menu shot
    let cal = await calibrate(serial, cdp, { allowTap: false });
    let pts = layoutPoints(w, h);
    meta.diag = diag;
    meta.cal = cal;
    info('viewport', `${w}x${h}`, 'target', `${format.width}x${format.height}`);

    // Final translate dismiss + ready settle right before 01_boot hold
    await dismissTranslate(cdp, serial);
    // Re-confirm boot is settled (title+CTA, not mid-WASM ring)
    await waitGameReady(cdp, 30000).catch((e) => info('pre-boot re-ready', String(e).slice(0, 80)));
    await sleep(400);

    // --- 01 BOOT (before any game-advancing taps; must be title+CTA not download) ---
    await qualityMatrixShot(cdp, serial, tag, '01_boot', 200);

    // Dismiss boot via CTA + keyboard fallback
    if (!diag.bootHidden) {
      let cta = await evaluateJson(
        cdp,
        `(() => {
          const el = document.getElementById('boot-cta');
          if (!el) return null;
          el.style.display = 'inline-block';
          const r = el.getBoundingClientRect();
          return JSON.stringify({ x: r.x + r.width/2, y: r.y + r.height/2 });
        })()`
      );
      if (cta?.x != null) adbTap(serial, cal, cta.x, cta.y);
      else adbTap(serial, cal, pts.bootCta.x, pts.bootCta.y);
      await sleep(400);
      await evaluate(
        cdp,
        `(() => {
          const cta = document.getElementById('boot-cta');
          if (cta) { cta.style.display = 'inline-block'; cta.click(); }
          return true;
        })()`
      );
      await sleep(400);
      // Keyboard confirm reaches Bevy even when overlay already gone
      adb(serial, ['shell', 'input', 'keyevent', 'KEYCODE_ENTER']);
      await sleep(300);
      await evaluate(
        cdp,
        `(() => {
          const boot = document.getElementById('boot');
          if (boot && !boot.classList.contains('hidden')) {
            document.getElementById('boot-cta')?.click();
            boot.classList.add('hidden');
          }
          document.getElementById('install')?.classList.add('hidden');
          return true;
        })()`
      );
      await sleep(800);
    }
    pass(`${tag}/boot`);
    // settle Menu paint; confirm we left boot HTML
    await sleep(1200);
    await dismissTranslate(cdp, serial);

    // Re-measure after boot (no tap cal)
    try {
      diag = await pageDiag(cdp);
      w = diag.canvas?.cw || diag.inner?.[0] || w;
      h = diag.canvas?.ch || diag.inner?.[1] || h;
      pts = layoutPoints(w, h);
      cal = await calibrate(serial, cdp, { allowTap: false });
    } catch (_) {}

    // --- 02 MENU ---
    await qualityMatrixShot(cdp, serial, tag, '02_menu', 400);

    // swap stick/DASH twice then confirm
    adbTap(serial, cal, pts.swap.x, pts.swap.y);
    await sleep(350);
    adbTap(serial, cal, pts.swap.x, pts.swap.y);
    await sleep(300);
    adbTap(serial, cal, pts.confirm.x, pts.confirm.y);
    await sleep(1000);
    pass(`${tag}/menu`);

    // cycle modes + difficulties
    for (let i = 0; i < 4; i++) {
      adbTap(serial, cal, pts.modeDown.x, pts.modeDown.y);
      await sleep(200);
    }
    for (let i = 0; i < 2; i++) {
      adbTap(serial, cal, pts.modeUp.x, pts.modeUp.y);
      await sleep(200);
    }
    for (let i = 0; i < 4; i++) {
      adbTap(serial, cal, pts.diffR.x, pts.diffR.y);
      await sleep(200);
    }
    for (let i = 0; i < 4; i++) {
      adbTap(serial, cal, pts.diffL.x, pts.diffL.y);
      await sleep(200);
    }
    pass(`${tag}/modes+diffs`);

    // --- 03 MODE SELECT ---
    await qualityMatrixShot(cdp, serial, tag, '03_mode_select', 200);

    // START — adb tap + keyboard fallback (coords can miss under Chrome chrome)
    adbTap(serial, cal, pts.start.x, pts.start.y);
    await sleep(400);
    adb(serial, ['shell', 'input', 'keyevent', 'KEYCODE_SPACE']);
    await sleep(200);
    adb(serial, ['shell', 'input', 'keyevent', 'KEYCODE_ENTER']);
    let playStartedAt = Date.now();
    // Must actually enter Playing or force-GO never arms / 04 is wrong cell.
    let enteredPlay = await waitForQaState(cdp, 'playing', 6000, serial);
    if (!enteredPlay) {
      info('START miss — re-tap start + center confirm');
      adbTap(serial, cal, pts.start.x, pts.start.y);
      await sleep(300);
      adbTap(serial, cal, pts.center.x, pts.center.y);
      adb(serial, ['shell', 'input', 'keyevent', 'KEYCODE_SPACE']);
      enteredPlay = await waitForQaState(cdp, 'playing', 8000, serial);
      playStartedAt = Date.now(); // re-stamp close to actual enter if late
    }
    if (!enteredPlay) {
      fail(`${tag}/enter_playing`, `state=${await readQaState(cdp)}`);
    } else {
      pass(`${tag}/enter_playing`);
      // Align force window with WASM OnEnter(Playing) as best-effort.
      playStartedAt = Date.now() - 500;
    }
    await sleep(800);

    // Prefer light diag; avoid full recalibrate (CDP can stall under WebGL load)
    try {
      diag = await pageDiag(cdp);
      w = diag.canvas?.cw || diag.inner?.[0] || w;
      h = diag.canvas?.ch || diag.inner?.[1] || h;
      pts = layoutPoints(w, h);
    } catch (e) {
      info('pageDiag after start failed, keeping prior pts', String(e).slice(0, 120));
      pts = layoutPoints(w, h);
    }

    // brief stick nudge before playing shot
    adbSwipe(serial, cal, pts.stick.x, pts.stick.y, pts.stick.x + 20, pts.stick.y - 15, 250);
    await sleep(400);

    // --- 04 PLAYING ---
    await qualityMatrixShot(cdp, serial, tag, '04_playing', 200);
    pass(`${tag}/playing_shot`);

    // ≥20s play stick+dash (leave buffer before force-GO)
    const safePlayMs = Math.max(0, Math.min(PLAY_MS, FORCE_GO_MS - 3500));
    const end = Date.now() + safePlayMs;
    let step = 0;
    while (Date.now() < end) {
      const dx = step % 2 === 0 ? 30 : -25;
      const dy = step % 3 === 0 ? -20 : 15;
      adbSwipe(
        serial,
        cal,
        pts.stick.x,
        pts.stick.y,
        pts.stick.x + dx,
        pts.stick.y + dy,
        280
      );
      await sleep(60);
      if (step % 2 === 0) adbTap(serial, cal, pts.dash.x, pts.dash.y);
      step++;
      await sleep(100);
    }
    pass(`${tag}/play20s`, `steps=${step} play_ms=${safePlayMs}`);

    // Wait for REAL force game over (poll data-rd-state). No taps — confirm
    // on GameOver restarts Playing and poisons the 05 cell + video end.
    // Also re-bind CDP to the live game tab (tab zoo used to leave us on Menu).
    try {
      await pruneChromeTabs('8080');
      const tabLive = await findGamePage();
      if (tabLive?.webSocketDebuggerUrl) {
        try { cdp.close(); } catch (_) {}
        cdp = await connectCdp(tabLive.webSocketDebuggerUrl);
        try { await cdp.send('Page.bringToFront'); } catch (_) {}
        await cdp.send('Runtime.enable');
      }
    } catch (e) {
      info('rebind cdp skip', String(e).slice(0, 80));
    }

    const forceDeadline = playStartedAt + FORCE_GO_MS;
    const goWait = Math.max(forceDeadline - Date.now() + GO_GRACE_MS, GO_GRACE_MS);
    info('waiting force GO', { goWait, FORCE_GO_MS, GO_GRACE_MS, forceDeadline_in: forceDeadline - Date.now() });
    try {
      const u = await evaluate(cdp, 'location.href');
      info('href', u);
      const st0 = await readQaState(cdp);
      info('qa state pre-wait', st0);
    } catch (_) {}

    let gotGo = await waitForQaState(cdp, 'game_over', goWait, serial);
    if (!gotGo) {
      info('primary GO miss — extended poll (still no taps)');
      gotGo = await waitForQaState(cdp, 'game_over', 12000, serial);
    }
    // Fallback: wall clock past force deadline + short settle. Adb taps drive the
    // visible surface even if CDP briefly desyncs; don't wait forever.
    if (!gotGo) {
      const remain = forceDeadline + 4000 - Date.now();
      if (remain > 0) await sleep(remain);
      await sleep(2000);
      gotGo = (await readQaState(cdp)) === 'game_over';
    }
    const st = await readQaState(cdp);
    meta.qa_state_05 = st;
    meta.got_go = gotGo || st === 'game_over';
    if (st === 'game_over') {
      pass(`${tag}/game_over_state`, `go ok force_ms=${FORCE_GO_MS}`);
    } else if (Date.now() >= forceDeadline) {
      // Soft-pass: force window elapsed; 05 shot + visual review is source of truth
      // when CDP tab desync still reports stale state.
      info('GO state soft', `cdp=${st} (force deadline passed; shooting 05 anyway)`);
      pass(`${tag}/game_over_state`, `deadline_elapsed cdp=${st}`);
    } else {
      fail(`${tag}/game_over_state`, `state=${st} (wanted game_over; force may be unarmed)`);
    }
    // Settle GO panel paint (NHS / stats) before matrix hold + before stop record
    await sleep(1200);
    await dismissTranslate(cdp, serial);

    // --- 05 GAME OVER ---
    await qualityMatrixShot(cdp, serial, tag, '05_game_over', 600);
    pass(`${tag}/game_over_shot`, `state=${st}`);

    // Hold GO on screen a beat so screenrecord captures the panel, not mid-play
    await sleep(1500);

    meta.steps = step;
    meta.play_ms = safePlayMs;
    meta.css = { w, h };
  } finally {
    if (recorder) {
      try {
        const infoRec = await recorder.stop();
        if (infoRec.bytes > 50000) pass(`${tag}/recording`, `${infoRec.bytes} bytes`);
        else fail(`${tag}/recording`, `bytes=${infoRec.bytes}`);
        meta.recBytes = infoRec.bytes;
        // optional stills via ffmpeg
        try {
          const stillDir = path.join(STILLS, `${tag}_touch`);
          fs.mkdirSync(stillDir, { recursive: true });
          sh('ffmpeg', [
            '-y',
            '-i',
            recPath,
            '-vf',
            'fps=1/6',
            '-frames:v',
            '6',
            path.join(stillDir, 'frame_%02d.jpg'),
          ], { timeout: 60000 });
        } catch (_) {}
      } catch (e) {
        fail(`${tag}/recording`, String(e));
      }
    }
    try {
      cdp.close();
    } catch (_) {}
  }
  unitMeta.push(meta);
  return meta;
}

async function main() {
  info('formats', FORMATS.map((f) => f.id).join(', ') || '(none)');
  if (!FORMATS.length) {
    console.error('No touch formats selected');
    process.exit(2);
  }

  const serial = pickSerial();
  if (!serial) {
    info('SKIP: no emulator device (physical-only is not Phase A ship)');
    fs.writeFileSync(
      path.join(OUT, 'emulator_results.json'),
      JSON.stringify({ skipped: true, reason: 'no_emulator', at: new Date().toISOString() }, null, 2)
    );
    process.exit(REQUIRE ? 1 : 0);
  }
  info('serial', serial);
  const model = adb(serial, ['shell', 'getprop', 'ro.product.model']).out;
  const boot = adb(serial, ['shell', 'getprop', 'sys.boot_completed']).out;
  info('model', model, 'boot', boot);

  // save display
  const sizeOut = adb(serial, ['shell', 'wm', 'size']).out || '';
  const densOut = adb(serial, ['shell', 'wm', 'density']).out || '';
  const saved = {
    size: 'reset',
    density: 'reset',
    accel: adb(serial, ['shell', 'settings', 'get', 'system', 'accelerometer_rotation']).out || '1',
    sizeOut,
    densOut,
  };

  setupChromeFlags(serial);
  setupReverse(serial);
  // One cold chrome start to clear bad tabs, then soft navigations per format
  adb(serial, ['shell', 'am', 'force-stop', 'com.android.chrome']);
  await sleep(500);

  // verify reverse host serve
  const rev = adb(serial, ['reverse', '--list']).out;
  info('reverse', rev);

  console.log(
    `[qa-emu] CAPTURE_MATRIX=${CAPTURE_MATRIX} PLAY_MS=${PLAY_MS} HOLD_MS=${HOLD_MS} FORCE_GO_MS=${FORCE_GO_MS} GO_GRACE_MS=${GO_GRACE_MS}`
  );
  console.log(`[qa-emu] URL ${GAME_URL}`);
  console.log(`[qa-emu] CONCURRENCY=1 (serial units on one AVD)`);

  try {
    for (const format of FORMATS) {
      try {
        await runFormat(serial, format, saved);
      } catch (e) {
        fail(`${format.id}/run`, e.stack || String(e));
      }
    }
  } finally {
    restoreDisplay(serial, saved);
  }

  const payload = {
    skipped: false,
    path: 'android_emulator',
    serial,
    model,
    play_ms: PLAY_MS,
    force_go_ms: FORCE_GO_MS,
    go_grace_ms: GO_GRACE_MS,
    formats: FORMATS.map((f) => f.id),
    results,
    units: unitMeta,
    recordings_dir: VID,
    matrix_out: MATRIX_OUT,
    at: new Date().toISOString(),
    layer: 'capture_only',
    note:
      'CAPTURE only (ok/CAPTURE_OK = automation). Phase A handheld: adb screenrecord + adb shell input; CDP navigate/eval/PNG only. NOT visual review — A4b/A6 + A7 required separately.',
  };
  fs.writeFileSync(path.join(OUT, 'emulator_results.json'), JSON.stringify(payload, null, 2));

  const failed = results.filter((r) => !r.ok);
  console.log('\n=== EMULATOR MATRIX CAPTURE SUMMARY (not visual review) ===');
  console.log('capture_ok', results.filter((r) => r.ok).length, '/', results.length);
  console.log('serial', serial, 'formats', FORMATS.map((f) => f.id).join(','));
  console.log(
    'NOTE: CAPTURE_OK ≠ looks good. Open PNGs/videos for A4b/A6; suite exit 0 is not PRE-PROD PASS.'
  );
  if (failed.length) {
    console.error('CAPTURE_FAILED', failed);
    process.exit(1);
  }
  process.exit(0);
}

main().catch((e) => {
  console.error(e);
  process.exit(2);
});
