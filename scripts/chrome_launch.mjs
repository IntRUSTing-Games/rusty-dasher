/**
 * Shared Chrome/Puppeteer launch args for QA scripts.
 *
 * Default: AMD/Intel iGPU via ANGLE (much faster than CPU SwiftShader).
 * Override for software-only CI boxes:
 *   CHROME_GL=swiftshader node scripts/e2e_inputs.mjs
 *   CHROME_GL=vulkan      # RADV / discrete when preferred
 */
import fs from 'fs';

const CHROME_CANDIDATES = [
  process.env.CHROME_PATH,
  '/home/viny/bin/google-chrome',
  '/usr/bin/google-chrome',
  '/usr/bin/google-chrome-stable',
  '/usr/bin/chromium-browser',
  '/usr/bin/chromium',
].filter(Boolean);

export function chromeExecutable() {
  for (const p of CHROME_CANDIDATES) {
    if (fs.existsSync(p)) return p;
  }
  throw new Error(
    'Chrome/Chromium not found. Set CHROME_PATH or install google-chrome.'
  );
}

/**
 * @returns {string[]} Chromium flags for WebGL/WASM Bevy QA
 */
export function chromeGpuArgs() {
  const mode = (process.env.CHROME_GL || 'gpu').toLowerCase();
  const base = [
    '--no-sandbox',
    '--disable-setuid-sandbox',
    '--disable-gpu-sandbox',
    '--ignore-gpu-blocklist',
    '--enable-webgl',
    '--enable-webgl2',
    '--enable-gpu',
    '--window-size=1920,1080',
    '--force-device-scale-factor=1',
  ];

  if (mode === 'swiftshader' || mode === 'cpu' || mode === 'sw') {
    // Explicit CPU fallback (slow; only for GPU-less environments).
    return [
      ...base,
      '--enable-unsafe-swiftshader',
      '--use-gl=angle',
      '--use-angle=swiftshader-webgl',
    ];
  }

  if (mode === 'vulkan') {
    return [...base, '--use-gl=angle', '--use-angle=vulkan'];
  }

  // Default: ANGLE → host OpenGL ES (AMD Radeon Vega iGPU on this machine).
  // Verified: unmasked renderer "ANGLE (AMD, AMD Radeon Graphics …)"
  return [...base, '--use-gl=angle', '--use-angle=default'];
}

export function logChromeGlMode() {
  const mode = (process.env.CHROME_GL || 'gpu').toLowerCase();
  console.log(
    `[chrome] GL mode=${mode} (set CHROME_GL=gpu|vulkan|swiftshader). exe=${chromeExecutable()}`
  );
}
