/**
 * Minimal Chrome DevTools Protocol client over WebSocket.
 * Used for real-device phone debug (no Puppeteer).
 *
 * Note: On Android Chrome, Input.dispatchTouchEvent often does NOT synthesize
 * real touchstart events. Prefer adb shell input (see e2e_phone.mjs) for
 * OS-level touches; use CDP for navigation, evaluate, and diagnostics.
 */
import WebSocket from 'ws';

/**
 * @param {string} wsUrl
 */
export async function connectCdp(wsUrl) {
  const ws = new WebSocket(wsUrl);
  await new Promise((resolve, reject) => {
    ws.once('open', resolve);
    ws.once('error', reject);
  });

  let nextId = 0;
  const pending = new Map();
  const eventHandlers = new Set();

  ws.on('message', (raw) => {
    let msg;
    try {
      msg = JSON.parse(raw.toString());
    } catch {
      return;
    }
    if (msg.id != null && pending.has(msg.id)) {
      const { resolve, reject } = pending.get(msg.id);
      pending.delete(msg.id);
      if (msg.error) reject(new Error(`${msg.error.message || ''} ${JSON.stringify(msg.error)}`));
      else resolve(msg.result);
      return;
    }
    if (msg.method) {
      for (const h of eventHandlers) h(msg);
    }
  });

  function send(method, params = {}) {
    const id = ++nextId;
    return new Promise((resolve, reject) => {
      pending.set(id, { resolve, reject });
      try {
        ws.send(JSON.stringify({ id, method, params }));
      } catch (e) {
        pending.delete(id);
        reject(e);
        return;
      }
      setTimeout(() => {
        if (pending.has(id)) {
          pending.delete(id);
          reject(new Error(`CDP timeout: ${method}`));
        }
      }, Number(process.env.CDP_TIMEOUT_MS || 120000));
    });
  }

  return {
    send,
    close: () => {
      try {
        ws.close();
      } catch (_) {}
    },
    onEvent: (fn) => {
      eventHandlers.add(fn);
      return () => eventHandlers.delete(fn);
    },
  };
}

export async function listPages(cdpPort = 9222) {
  const res = await fetch(`http://127.0.0.1:${cdpPort}/json/list`);
  if (!res.ok) throw new Error(`CDP list failed: ${res.status}`);
  return res.json();
}

export async function cdpVersion(cdpPort = 9222) {
  const res = await fetch(`http://127.0.0.1:${cdpPort}/json/version`);
  if (!res.ok) throw new Error(`CDP version failed: ${res.status}`);
  return res.json();
}

/** Evaluate JS in page; returns JSON-serializable value. */
export async function evaluate(cdp, expression) {
  const { result, exceptionDetails } = await cdp.send('Runtime.evaluate', {
    expression,
    returnByValue: true,
    awaitPromise: true,
  });
  if (exceptionDetails) {
    throw new Error(exceptionDetails.text || JSON.stringify(exceptionDetails));
  }
  return result?.value;
}

export async function evaluateJson(cdp, expression) {
  const v = await evaluate(cdp, expression);
  if (typeof v === 'string') {
    try {
      return JSON.parse(v);
    } catch {
      return v;
    }
  }
  return v;
}

/**
 * CDP touch — may be a no-op for touchstart on Android Chrome.
 * Prefer adbInput helpers in e2e_phone for real device touches.
 */
export async function touchTap(cdp, x, y, { holdMs = 50 } = {}) {
  await cdp.send('Input.dispatchTouchEvent', {
    type: 'touchStart',
    touchPoints: [{ x, y, id: 1, radiusX: 5, radiusY: 5, force: 0.5 }],
  });
  await sleep(holdMs);
  await cdp.send('Input.dispatchTouchEvent', {
    type: 'touchEnd',
    touchPoints: [],
  });
  // Fallback mouse (fires pointer events on Android)
  await cdp.send('Input.dispatchMouseEvent', {
    type: 'mousePressed',
    x,
    y,
    button: 'left',
    clickCount: 1,
    pointerType: 'touch',
  });
  await sleep(30);
  await cdp.send('Input.dispatchMouseEvent', {
    type: 'mouseReleased',
    x,
    y,
    button: 'left',
    clickCount: 1,
    pointerType: 'touch',
  });
}

export async function touchDrag(cdp, from, to, { steps = 6, stepMs = 30 } = {}) {
  await cdp.send('Input.dispatchTouchEvent', {
    type: 'touchStart',
    touchPoints: [{ x: from.x, y: from.y, id: 1, radiusX: 5, radiusY: 5, force: 0.5 }],
  });
  for (let i = 1; i <= steps; i++) {
    const t = i / steps;
    const x = from.x + (to.x - from.x) * t;
    const y = from.y + (to.y - from.y) * t;
    await cdp.send('Input.dispatchTouchEvent', {
      type: 'touchMove',
      touchPoints: [{ x, y, id: 1, radiusX: 5, radiusY: 5, force: 0.5 }],
    });
    await sleep(stepMs);
  }
}

export async function touchEndAll(cdp) {
  await cdp.send('Input.dispatchTouchEvent', {
    type: 'touchEnd',
    touchPoints: [],
  });
}

export async function touchStickAndDash(cdp, stick, stick2, dash) {
  await cdp.send('Input.dispatchTouchEvent', {
    type: 'touchStart',
    touchPoints: [{ x: stick.x, y: stick.y, id: 1, radiusX: 5, radiusY: 5, force: 0.5 }],
  });
  await sleep(80);
  await cdp.send('Input.dispatchTouchEvent', {
    type: 'touchMove',
    touchPoints: [{ x: stick2.x, y: stick2.y, id: 1, radiusX: 5, radiusY: 5, force: 0.5 }],
  });
  await sleep(100);
  await cdp.send('Input.dispatchTouchEvent', {
    type: 'touchStart',
    touchPoints: [
      { x: stick2.x, y: stick2.y, id: 1, radiusX: 5, radiusY: 5, force: 0.5 },
      { x: dash.x, y: dash.y, id: 2, radiusX: 5, radiusY: 5, force: 0.5 },
    ],
  });
  await sleep(180);
  await cdp.send('Input.dispatchTouchEvent', {
    type: 'touchEnd',
    touchPoints: [{ x: stick2.x, y: stick2.y, id: 1, radiusX: 5, radiusY: 5, force: 0.5 }],
  });
  await sleep(80);
  await cdp.send('Input.dispatchTouchEvent', {
    type: 'touchEnd',
    touchPoints: [],
  });
}

export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
