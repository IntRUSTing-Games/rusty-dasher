/**
 * Chrome device emulation profiles for phone/tablet formats.
 * Phone formats must use full mobile device metrics (not a bare resized desktop window).
 */
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const MATRIX = JSON.parse(fs.readFileSync(path.join(__dirname, 'qa_matrix.json'), 'utf8'));

const ANDROID_UA =
  'Mozilla/5.0 (Linux; Android 14; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36';
const IPHONE_UA =
  'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1';

/**
 * Build a Puppeteer/CDP device descriptor from a qa_matrix format entry.
 * Whole-chain phone tests use this so layout/UA/DPR match real mobile Chrome.
 */
export function deviceFromFormat(format) {
  const isIphone = /iphone|phone_portrait|phone_landscape|phone_large/i.test(format.id);
  const ua = format.userAgent || (isIphone ? IPHONE_UA : ANDROID_UA);
  return {
    name: format.label || format.id,
    userAgent: ua,
    viewport: {
      width: format.width,
      height: format.height,
      deviceScaleFactor: format.dpr || 2,
      isMobile: true,
      hasTouch: true,
      isLandscape: format.width > format.height,
    },
  };
}

/**
 * Apply full device emulation on a Puppeteer page (metrics + touch + UA).
 * Also sets a mobile window size so screenshots are the device viewport chain,
 * not an isolated desktop tab crop.
 */
export async function applyDeviceEmulation(page, format) {
  const device = deviceFromFormat(format);
  await page.emulate(device);

  const client = await page.createCDPSession();
  await client.send('Emulation.setTouchEmulationEnabled', {
    enabled: true,
    maxTouchPoints: 5,
  });
  // Match Android Chrome: mobile viewport meta behavior
  await client.send('Emulation.setEmitTouchEventsForMouse', {
    enabled: true,
    configuration: 'mobile',
  });
  return { client, device };
}

/** True if this matrix format should use device emulation (phones + tablets). */
export function isHandheldFormat(format) {
  return !!format.touch || /phone|tablet/i.test(format.id);
}

export function handheldFormats() {
  return MATRIX.formats.filter(isHandheldFormat);
}

export function desktopFormats() {
  return MATRIX.formats.filter((f) => !isHandheldFormat(f));
}

export { MATRIX, ANDROID_UA, IPHONE_UA };
