/**
 * Desktop regression baseline: exact timed keyboard inputs + video + metrics.
 *
 *   ./scripts/web-serve-dist.sh          # http://127.0.0.1:17880/
 *   node scripts/desktop_baseline.mjs record
 *   node scripts/desktop_baseline.mjs replay
 *   node scripts/desktop_baseline.mjs compare
 *
 * Env: E2E_URL | RUSTY_PORT | PORT | BASELINE_DIR | BASELINE_PLAY_MS | BASELINE_FORMAT
 */
import puppeteer from 'puppeteer-core';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { chromeExecutable, chromeGpuArgs, logChromeGlMode } from './chrome_launch.mjs';
import { startPageRecording, extractReviewStills } from './record.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(__dirname, '..');
const MATRIX = JSON.parse(fs.readFileSync(path.join(__dirname, 'qa_matrix.json'), 'utf8'));

const PORT = process.env.PORT || process.env.RUSTY_PORT || '17880';
const BASE = (process.env.E2E_URL || `http://127.0.0.1:${PORT}/`).replace(/\/?$/, '/');
const OUT = process.env.BASELINE_DIR
  ? path.resolve(process.env.BASELINE_DIR)
  : path.join(ROOT, 'screenshots/desktop_baseline');
const PLAY_MS = Number(process.env.BASELINE_PLAY_MS || 22000);
const FORMAT_ID = process.env.BASELINE_FORMAT || '1080p';
const FORCE_GO_MS = Number(process.env.E2E_FORCE_GO_MS || PLAY_MS + 5000);

const format = MATRIX.formats.find((f) => f.id === FORMAT_ID);
if (!format || format.touch) {
  console.error('Need non-touch format, e.g. BASELINE_FORMAT=1080p');
  process.exit(2);
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** Deterministic play segment relative to play_t0 (ms). */
function buildPlayScript(playMs) {
  const events = [];
  let t = 0;
  const seq = [
    ['KeyW', 450, 80],
    ['KeyD', 500, 80],
    ['KeyS', 450, 80],
    ['KeyA', 500, 80],
    ['ArrowUp', 400, 80],
    ['ArrowRight', 450, 80],
    ['ArrowDown', 400, 80],
    ['ArrowLeft', 450, 80],
  ];
  let i = 0;
  while (t < playMs - 900) {
    const [key, hold, gap] = seq[i % seq.length];
    events.push({ t_ms: t, type: 'keyDown', key });
    events.push({ t_ms: t + hold, type: 'keyUp', key });
    t += hold + gap;
    if (i % 3 === 0 && t + 250 < playMs - 700) {
      events.push({ t_ms: t, type: 'keyPress', key: 'Space' });
      t += 130;
    }
    i++;
  }
  for (const key of [
    'KeyW', 'KeyA', 'KeyS', 'KeyD',
    'ArrowUp', 'ArrowLeft', 'ArrowDown', 'ArrowRight', 'Space',
  ]) {
    events.push({ t_ms: playMs - 40, type: 'keyUp', key });
  }
  return events.sort((a, b) => a.t_ms - b.t_ms);
}

/**
 * Menu path as discrete timed steps from session_t0.
 * Mirrors e2e_inputs desktop keyboard primary (no Escape-quit race).
 */
function buildMenuScript() {
  return [
    { t_ms: 0, type: 'note', text: 'after_ready' },
    { t_ms: 200, type: 'clickBootCta' },
    { t_ms: 1400, type: 'focusCanvas' },
    { t_ms: 1600, type: 'keyPress', key: 'Enter' }, // menu → mode
    { t_ms: 2600, type: 'keyPress', key: 'ArrowDown' },
    { t_ms: 2760, type: 'keyPress', key: 'ArrowDown' },
    { t_ms: 2920, type: 'keyPress', key: 'ArrowDown' },
    { t_ms: 3080, type: 'keyPress', key: 'ArrowDown' },
    { t_ms: 3240, type: 'keyPress', key: 'ArrowUp' },
    { t_ms: 3400, type: 'keyPress', key: 'ArrowUp' },
    { t_ms: 3560, type: 'keyPress', key: 'ArrowUp' },
    { t_ms: 3720, type: 'keyPress', key: 'ArrowUp' },
    { t_ms: 4000, type: 'keyPress', key: 'ArrowRight' },
    { t_ms: 4160, type: 'keyPress', key: 'ArrowRight' },
    { t_ms: 4320, type: 'keyPress', key: 'ArrowRight' },
    { t_ms: 4480, type: 'keyPress', key: 'ArrowRight' },
    { t_ms: 4640, type: 'keyPress', key: 'ArrowLeft' },
    { t_ms: 4800, type: 'keyPress', key: 'ArrowLeft' },
    { t_ms: 4960, type: 'keyPress', key: 'ArrowLeft' },
    { t_ms: 5120, type: 'keyPress', key: 'ArrowLeft' },
    { t_ms: 5400, type: 'keyPress', key: 'Space' }, // START
    { t_ms: 6500, type: 'note', text: 'play_settle' },
  ];
}

async function safeEval(page, fn) {
  try {
    if (page.isClosed()) return null;
    return await page.evaluate(fn);
  } catch (_) {
    return null;
  }
}

async function sampleMetrics(page) {
  const m = await safeEval(page, () => {
    const canvas = document.querySelector('canvas');
    const r = canvas?.getBoundingClientRect?.();
    return {
      t_wall: Date.now(),
      state: document.documentElement?.getAttribute('data-rd-state') || null,
      inner: [window.innerWidth, window.innerHeight],
      dpr: window.devicePixelRatio || 1,
      canvas: r ? { w: r.width, h: r.height } : null,
    };
  });
  return m || { t_wall: Date.now(), state: null, error: 'eval_failed' };
}

async function dismissBoot(page) {
  try {
    await page.waitForSelector('#boot-cta', { timeout: 20000 });
    await sleep(300);
    const box = await safeEval(page, () => {
      const cta = document.getElementById('boot-cta');
      if (!cta) return null;
      const r = cta.getBoundingClientRect();
      if (r.width < 2) return null;
      return { x: r.x + r.width / 2, y: r.y + r.height / 2 };
    });
    if (box) await page.mouse.click(box.x, box.y);
    await sleep(200);
    await safeEval(page, () => {
      document.getElementById('boot')?.classList.add('hidden');
      document.querySelector('canvas')?.focus?.();
    });
  } catch (e) {
    console.warn('[baseline] boot dismiss', e.message || e);
  }
}

async function focusCanvas(page) {
  await safeEval(page, () => {
    const c = document.querySelector('canvas');
    if (c) {
      c.focus();
      c.click();
    }
  });
}

async function applyEvent(page, ev) {
  if (page.isClosed()) return;
  switch (ev.type) {
    case 'keyDown':
      await page.keyboard.down(ev.key);
      break;
    case 'keyUp':
      try {
        await page.keyboard.up(ev.key);
      } catch (_) {}
      break;
    case 'keyPress':
      await page.keyboard.press(ev.key);
      break;
    case 'clickBootCta':
      await dismissBoot(page);
      break;
    case 'focusCanvas':
      await focusCanvas(page);
      break;
    case 'note':
      break;
    default:
      console.warn('unknown', ev.type);
  }
}

async function runTimedScript(page, events, t0, log) {
  for (const ev of events) {
    if (page.isClosed()) break;
    const wait = t0 + ev.t_ms - Date.now();
    if (wait > 0) await sleep(wait);
    const wall = Date.now();
    await applyEvent(page, ev);
    log.push({ ...ev, wall_ms: wall, offset_ms: wall - t0 });
  }
}

async function newPage(browser) {
  const page = await browser.newPage();
  page.on('pageerror', (err) => console.warn('[pageerror]', err.message));
  await page.setViewport({
    width: format.width,
    height: format.height,
    deviceScaleFactor: format.dpr || 1,
    isMobile: false,
    hasTouch: false,
  });
  return page;
}

async function waitReady(page) {
  await page.waitForSelector('canvas', { timeout: 120000 });
  await page.waitForFunction(
    () => {
      const boot = document.getElementById('boot');
      const cta = document.getElementById('boot-cta');
      if (boot?.classList.contains('hidden')) return true;
      if (!cta) return false;
      const d = cta.style?.display || '';
      if (d === 'inline-block' || d === 'block') return true;
      try {
        const cs = getComputedStyle(cta);
        return cs.display !== 'none' && cs.visibility !== 'hidden';
      } catch (_) {
        return false;
      }
    },
    { timeout: 120000 }
  );
}

async function recordBaseline() {
  fs.mkdirSync(OUT, { recursive: true });
  logChromeGlMode();

  const menuScript = buildMenuScript();
  const playScript = buildPlayScript(PLAY_MS);
  const url = `${BASE}?e2e=1&qa_matrix=1&qa_go_ms=${FORCE_GO_MS}&baseline=1`;

  const meta = {
    kind: 'desktop_baseline',
    version: 2,
    created_at: new Date().toISOString(),
    purpose:
      'Exact desktop keyboard input log + video + metrics. Replay after mobile-only changes to prove desktop feel unchanged.',
    base_url: BASE,
    url,
    format: {
      id: format.id,
      width: format.width,
      height: format.height,
      dpr: format.dpr || 1,
      touch: false,
      expected_class: format.expected_class || null,
    },
    play_ms: PLAY_MS,
    force_go_ms: FORCE_GO_MS,
    paths: {
      inputs: 'inputs.json',
      metrics: 'metrics_record.json',
      video: `baseline_${format.id}_keyboard.webm`,
      stills: `stills_${format.id}_keyboard`,
      meta: 'baseline_meta.json',
    },
  };

  const browser = await puppeteer.launch({
    executablePath: chromeExecutable(),
    headless: 'new',
    args: chromeGpuArgs(),
    protocolTimeout: 300000,
  });

  const inputLog = [];
  const metricsTimeline = [];
  let recStop = null;
  let sampleIv = null;

  try {
    const page = await newPage(browser);
    const recPath = path.join(OUT, meta.paths.video);
    await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 180000 });
    await waitReady(page);
    await sleep(500);

    recStop = await startPageRecording(page, recPath, {
      everyNthFrame: 1,
      fps: 15,
      quality: 60,
    });

    const sessionT0 = Date.now();
    metricsTimeline.push({ phase: 'ready', ...(await sampleMetrics(page)) });

    await runTimedScript(page, menuScript, sessionT0, inputLog);
    metricsTimeline.push({ phase: 'after_menu', ...(await sampleMetrics(page)) });

    // Nudge into play if needed (e2e style)
    await sleep(400);
    await focusCanvas(page);

    const playT0 = Date.now();
    metricsTimeline.push({
      phase: 'play_start',
      play_t0_wall: playT0,
      ...(await sampleMetrics(page)),
    });

    sampleIv = setInterval(() => {
      sampleMetrics(page)
        .then((m) => {
          if (!m.error) {
            metricsTimeline.push({
              phase: 'play',
              play_offset_ms: Date.now() - playT0,
              ...m,
            });
          }
        })
        .catch(() => {});
    }, 500);

    await runTimedScript(page, playScript, playT0, inputLog);

    if (sampleIv) clearInterval(sampleIv);
    sampleIv = null;
    await sleep(300);
    metricsTimeline.push({
      phase: 'play_end',
      play_offset_ms: Date.now() - playT0,
      ...(await sampleMetrics(page)),
    });

    // No Space after play — force GO may fire; wait out remaining window quietly
    await sleep(1500);
    metricsTimeline.push({ phase: 'session_end', ...(await sampleMetrics(page)) });

    const rec = await recStop.stop();
    recStop = null;

    try {
      await extractReviewStills(rec.path, path.join(OUT, meta.paths.stills), 8);
    } catch (e) {
      console.warn('[baseline] stills', e.message || e);
    }

    const inputsPayload = {
      version: 2,
      format_id: format.id,
      session_t0_note: 'menu_script t_ms from session_t0 after waitReady',
      play_t0_note: 'play_script t_ms from play_t0 (after menu_script completes)',
      play_ms: PLAY_MS,
      menu_script: menuScript,
      play_script: playScript,
      executed: inputLog,
      replay_instructions:
        'Apply menu_script timed from session start after ready; then play_script timed from play_t0. Prefer scripts over executed[].',
    };

    const metricsPayload = {
      version: 2,
      format_id: format.id,
      recorded_at: new Date().toISOString(),
      video: {
        path: path.relative(ROOT, rec.path),
        frames: rec.frames,
        bytes: rec.bytes,
      },
      timeline: metricsTimeline,
      summary: {
        final_state: metricsTimeline.at(-1)?.state ?? null,
        samples: metricsTimeline.length,
        viewport: `${format.width}x${format.height}`,
        dpr: format.dpr || 1,
        play_ms: PLAY_MS,
      },
    };

    fs.writeFileSync(path.join(OUT, 'baseline_meta.json'), JSON.stringify(meta, null, 2));
    fs.writeFileSync(path.join(OUT, 'inputs.json'), JSON.stringify(inputsPayload, null, 2));
    fs.writeFileSync(
      path.join(OUT, 'metrics_record.json'),
      JSON.stringify(metricsPayload, null, 2)
    );
    fs.writeFileSync(
      path.join(OUT, 'README.md'),
      `# Desktop baseline (protect desktop feel)

Recorded: **${meta.created_at}**  
Format: **${format.id}** ${format.width}×${format.height}  
Video: \`${meta.paths.video}\`  
Inputs: \`inputs.json\` (exact timed key sequence)  
Metrics: \`metrics_record.json\`

## After mobile-only changes

\`\`\`bash
./scripts/web-serve-dist.sh
node scripts/desktop_baseline.mjs replay
node scripts/desktop_baseline.mjs compare
# open baseline_*.webm vs replay_*.webm
\`\`\`

If desktop motion/dash/layout changed, **revert non-handheld-gated code**.
`
    );

    console.log('\n=== DESKTOP BASELINE RECORD OK ===');
    console.log('out', OUT);
    console.log('video', rec.path, 'frames', rec.frames, 'bytes', rec.bytes);
    console.log('input events', inputLog.length);
    console.log('metric samples', metricsTimeline.length);
  } catch (e) {
    console.error('[baseline] FAILED', e);
    throw e;
  } finally {
    if (sampleIv) clearInterval(sampleIv);
    if (recStop) {
      try {
        await recStop.stop();
      } catch (_) {}
    }
    try {
      await browser.close();
    } catch (_) {}
  }
}

async function replayBaseline() {
  const inputsPath = path.join(OUT, 'inputs.json');
  if (!fs.existsSync(inputsPath)) {
    console.error('Run record first');
    process.exit(2);
  }
  const inputs = JSON.parse(fs.readFileSync(inputsPath, 'utf8'));
  const fmt = MATRIX.formats.find((f) => f.id === inputs.format_id) || format;

  logChromeGlMode();
  const browser = await puppeteer.launch({
    executablePath: chromeExecutable(),
    headless: 'new',
    args: chromeGpuArgs(),
    protocolTimeout: 300000,
  });

  const metricsTimeline = [];
  let recStop = null;
  let sampleIv = null;
  const url = `${BASE}?e2e=1&qa_matrix=1&qa_go_ms=${FORCE_GO_MS}&baseline=1&replay=1`;

  try {
    const page = await newPage(browser);
    await page.setViewport({
      width: fmt.width,
      height: fmt.height,
      deviceScaleFactor: fmt.dpr || 1,
      isMobile: false,
      hasTouch: false,
    });
    await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 180000 });
    await waitReady(page);
    await sleep(500);

    const recPath = path.join(OUT, `replay_${fmt.id}_keyboard.webm`);
    recStop = await startPageRecording(page, recPath, {
      everyNthFrame: 1,
      fps: 15,
      quality: 60,
    });

    const sessionT0 = Date.now();
    const execLog = [];
    await runTimedScript(page, inputs.menu_script, sessionT0, execLog);
    await sleep(400);
    await focusCanvas(page);

    const playT0 = Date.now();
    metricsTimeline.push({
      phase: 'play_start',
      ...(await sampleMetrics(page)),
    });
    sampleIv = setInterval(() => {
      sampleMetrics(page)
        .then((m) => {
          if (!m.error) {
            metricsTimeline.push({
              phase: 'play',
              play_offset_ms: Date.now() - playT0,
              ...m,
            });
          }
        })
        .catch(() => {});
    }, 500);

    await runTimedScript(page, inputs.play_script, playT0, execLog);
    if (sampleIv) clearInterval(sampleIv);
    sampleIv = null;
    await sleep(300);
    metricsTimeline.push({ phase: 'play_end', ...(await sampleMetrics(page)) });
    await sleep(1500);
    metricsTimeline.push({ phase: 'session_end', ...(await sampleMetrics(page)) });

    const rec = await recStop.stop();
    recStop = null;
    try {
      await extractReviewStills(rec.path, path.join(OUT, `stills_replay_${fmt.id}_keyboard`), 8);
    } catch (_) {}

    fs.writeFileSync(
      path.join(OUT, 'metrics_replay.json'),
      JSON.stringify(
        {
          version: 2,
          format_id: fmt.id,
          replayed_at: new Date().toISOString(),
          video: {
            path: path.relative(ROOT, rec.path),
            frames: rec.frames,
            bytes: rec.bytes,
          },
          timeline: metricsTimeline,
          summary: {
            final_state: metricsTimeline.at(-1)?.state ?? null,
            samples: metricsTimeline.length,
            viewport: `${fmt.width}x${fmt.height}`,
            play_ms: inputs.play_ms,
          },
          executed_events: execLog.length,
        },
        null,
        2
      )
    );
    console.log('\n=== DESKTOP BASELINE REPLAY OK ===');
    console.log(rec.path, 'frames', rec.frames);
  } finally {
    if (sampleIv) clearInterval(sampleIv);
    if (recStop) {
      try {
        await recStop.stop();
      } catch (_) {}
    }
    try {
      await browser.close();
    } catch (_) {}
  }
}

function compareMetrics() {
  const aPath = path.join(OUT, 'metrics_record.json');
  const bPath = path.join(OUT, 'metrics_replay.json');
  if (!fs.existsSync(aPath) || !fs.existsSync(bPath)) {
    console.error('Need metrics_record.json + metrics_replay.json');
    process.exit(2);
  }
  const a = JSON.parse(fs.readFileSync(aPath, 'utf8'));
  const b = JSON.parse(fs.readFileSync(bPath, 'utf8'));
  const report = {
    compared_at: new Date().toISOString(),
    baseline: a.summary,
    replay: b.summary,
    video_frames: { baseline: a.video?.frames, replay: b.video?.frames },
    frame_delta: Math.abs((a.video?.frames || 0) - (b.video?.frames || 0)),
    notes: [
      'Coarse auto check only — open baseline_*.webm vs replay_*.webm for desktop feel.',
      'If motion/dash/layout on desktop changed, revert non-handheld-gated code.',
    ],
    verdict:
      a.summary?.viewport === b.summary?.viewport &&
      Math.abs((a.video?.frames || 0) - (b.video?.frames || 0)) < 100
        ? 'LIKELY_OK_CHECK_VIDEOS'
        : 'INSPECT_VIDEOS',
  };
  fs.writeFileSync(path.join(OUT, 'compare_report.json'), JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report, null, 2));
}

const cmd = process.argv[2] || 'record';
const run =
  cmd === 'record'
    ? recordBaseline
    : cmd === 'replay'
      ? replayBaseline
      : cmd === 'compare'
        ? async () => compareMetrics()
        : null;
if (!run) {
  console.error('Usage: node scripts/desktop_baseline.mjs record|replay|compare');
  process.exit(2);
}
run().catch((e) => {
  console.error(e);
  process.exit(1);
});
