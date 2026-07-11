/**
 * Session video recorder for E2E.
 *
 * Preferred artifact for input e2e is continuous video (catches flicker, hitches,
 * wrong transitions) — not a handful of still screenshots.
 *
 * Local Chrome (Puppeteer CDP): Page.startScreencast → JPEG frames → ffmpeg webm/mp4
 * Android phone: use adb screenrecord helpers (see e2e_phone.mjs)
 */
import { spawn } from 'child_process';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

/**
 * Start CDP screencast recording on a Puppeteer page.
 * @returns {{ stop: () => Promise<{ path: string, frames: number, bytes: number }> }}
 */
export async function startPageRecording(page, outPath, opts = {}) {
  // everyNthFrame=2: denser timeline under parallel WASM (was 3 → sparse/truncated
  // keyboard primaries when frames were also dropped under IO pressure).
  const quality = opts.quality ?? 55;
  const everyNthFrame = opts.everyNthFrame ?? 2;
  const format = opts.format ?? 'jpeg';
  const fps = opts.fps ?? 12;

  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  const framesDir = outPath + '.frames';
  fs.rmSync(framesDir, { recursive: true, force: true });
  fs.mkdirSync(framesDir, { recursive: true });

  const client = await page.createCDPSession();
  let frameIdx = 0;
  let stopped = false;
  let dropped = 0;

  // Sequential write queue: never block the screencast hot path on disk, but
  // prefer buffering over dropping so long keyboard journeys stay ≥20s in video.
  // Previous MAX_WRITES_IN_FLIGHT=24 dropped most mid/late frames under CONCURRENCY≥2.
  const queue = [];
  let writing = false;
  let writesDone = Promise.resolve();
  const MAX_QUEUE = Number(process.env.E2E_REC_MAX_QUEUE || 400);

  const pumpWrites = () => {
    if (writing) return;
    writing = true;
    writesDone = (async () => {
      while (queue.length) {
        const job = queue.shift();
        try {
          await fs.promises.writeFile(job.file, job.buf);
        } catch (_) {}
      }
      writing = false;
      // Another frame may have enqueued while we drained.
      if (queue.length) pumpWrites();
    })();
  };

  const onFrame = (event) => {
    if (stopped) return;
    // Always ack immediately so Chrome keeps sending frames.
    client
      .send('Page.screencastFrameAck', { sessionId: event.sessionId })
      .catch(() => {});
    if (queue.length >= MAX_QUEUE) {
      dropped++;
      return;
    }
    const i = frameIdx++;
    const file = path.join(framesDir, `f${String(i).padStart(6, '0')}.jpg`);
    queue.push({ file, buf: Buffer.from(event.data, 'base64') });
    pumpWrites();
  };

  client.on('Page.screencastFrame', onFrame);
  await client.send('Page.startScreencast', {
    format,
    quality,
    everyNthFrame,
  });

  return {
    async stop() {
      if (stopped) return { path: outPath, frames: frameIdx, bytes: 0 };
      stopped = true;
      try {
        await client.send('Page.stopScreencast');
      } catch (_) {}
      try {
        client.off('Page.screencastFrame', onFrame);
      } catch (_) {}

      // Drain pending JPEG writes before ffmpeg sees the frame dir.
      for (let i = 0; i < 50; i++) {
        if (!queue.length && !writing) break;
        await writesDone.catch(() => {});
        await new Promise((r) => setTimeout(r, 40));
      }
      await writesDone.catch(() => {});

      const frames = frameIdx;
      if (dropped > 0) {
        console.warn(`[rec] dropped ${dropped} frames (queue cap ${MAX_QUEUE}) → ${outPath}`);
      }
      if (frames < 2) {
        fs.writeFileSync(outPath + '.error.txt', `too few frames: ${frames}`);
        return { path: outPath, frames, bytes: 0, error: 'too few frames' };
      }

      // Encode with ffmpeg (webm vp8 — widely viewable)
      const ext = path.extname(outPath).toLowerCase();
      const args =
        ext === '.mp4'
          ? [
              '-y',
              '-framerate',
              String(fps),
              '-i',
              path.join(framesDir, 'f%06d.jpg'),
              '-c:v',
              'libx264',
              '-pix_fmt',
              'yuv420p',
              '-movflags',
              '+faststart',
              outPath,
            ]
          : [
              '-y',
              '-framerate',
              String(fps),
              '-i',
              path.join(framesDir, 'f%06d.jpg'),
              '-c:v',
              'libvpx',
              '-b:v',
              '1M',
              '-auto-alt-ref',
              '0',
              outPath,
            ];

      try {
        await new Promise((resolve, reject) => {
          const ff = spawn('ffmpeg', args, { stdio: ['ignore', 'pipe', 'pipe'] });
          let err = '';
          ff.stderr.on('data', (d) => {
            err += d.toString();
          });
          ff.on('close', (code) => {
            if (code === 0) resolve();
            else reject(new Error(`ffmpeg exit ${code}: ${err.slice(-800)}`));
          });
        });
      } catch (e) {
        fs.writeFileSync(outPath + '.error.txt', String(e));
        // Keep frames dir for debug when encode fails
        return { path: outPath, frames, bytes: 0, error: String(e) };
      }

      // Keep frames only if KEEP_E2E_FRAMES=1 (disk heavy)
      if (process.env.KEEP_E2E_FRAMES !== '1') {
        fs.rmSync(framesDir, { recursive: true, force: true });
      }

      const bytes = fs.existsSync(outPath) ? fs.statSync(outPath).size : 0;
      return { path: outPath, frames, bytes, dropped };
    },
  };
}

/**
 * Extract a few sample stills from a video for quick agent review.
 * @returns {string[]} paths
 */
export async function extractReviewStills(videoPath, outDir, count = 6) {
  fs.mkdirSync(outDir, { recursive: true });
  if (!fs.existsSync(videoPath) || fs.statSync(videoPath).size < 500) return [];

  // Probe duration
  const dur = await new Promise((resolve) => {
    const p = spawn(
      'ffprobe',
      ['-v', 'error', '-show_entries', 'format=duration', '-of', 'csv=p=0', videoPath],
      { stdio: ['ignore', 'pipe', 'pipe'] }
    );
    let o = '';
    p.stdout.on('data', (d) => (o += d));
    p.on('close', () => resolve(parseFloat(o) || 10));
  });

  const paths = [];
  for (let i = 0; i < count; i++) {
    const t = ((i + 0.5) / count) * Math.max(dur, 1);
    const out = path.join(outDir, `still_${String(i).padStart(2, '0')}.jpg`);
    await new Promise((resolve) => {
      const ff = spawn(
        'ffmpeg',
        ['-y', '-ss', String(t.toFixed(2)), '-i', videoPath, '-frames:v', '1', '-q:v', '4', out],
        { stdio: 'ignore' }
      );
      ff.on('close', () => resolve());
    });
    if (fs.existsSync(out) && fs.statSync(out).size > 200) paths.push(out);
  }
  return paths;
}

export { __dirname };
