//! Web pointer / resolution helpers: keep touch & mouse coords aligned with the
//! canvas when DPR, browser zoom, or soft-caps desync Bevy's window size.
//!
//! **Critical (mobile Chrome):** winit converts pointer events with
//! `device_pixel_ratio()` then Bevy divides by `Window` scale_factor. If we
//! override scale_factor to a *lower* value (old 2.5 clamp on a 3.25 phone),
//! every touch is scaled by `real_dpr / our_sf` (e.g. 1.3×) and misses the
//! virtual stick / DASH. Scale factor must always equal the browser DPR.

use bevy::prelude::*;

/// Remap a Bevy window-space pointer into coordinates that match the camera
/// viewport (logical window size). On native this is identity.
///
/// On WASM, browser zoom / visualViewport / residual size mismatches can make
/// Bevy's reported touch disagree with the painted canvas. We rescale from the
/// live canvas bounding box into the window's logical size.
#[inline]
pub fn remap_to_window(pos: Vec2, window: &Window) -> Vec2 {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(mapped) = remap_wasm(pos, window) {
            return mapped;
        }
    }
    let _ = window;
    pos
}

#[cfg(target_arch = "wasm32")]
fn remap_wasm(pos: Vec2, window: &Window) -> Option<Vec2> {
    use wasm_bindgen::JsCast;

    let win = web_sys::window()?;
    let doc = win.document()?;
    let el = doc.query_selector("canvas").ok().flatten()?;
    let canvas = el.dyn_into::<web_sys::HtmlCanvasElement>().ok()?;
    let rect = canvas.get_bounding_client_rect();
    let rw = rect.width() as f32;
    let rh = rect.height() as f32;
    if rw < 2.0 || rh < 2.0 {
        return None;
    }

    let ww = window.width().max(1.0);
    let wh = window.height().max(1.0);

    // Correct scale_factor path: Bevy logical ≈ CSS offset coords; identity.
    if (rw - ww).abs() < 1.5 && (rh - wh).abs() < 1.5 {
        return Some(pos);
    }

    // Residual mismatch (safe-area / temporary resize): scale into window space.
    Some(Vec2::new(pos.x * (ww / rw), pos.y * (wh / rh)))
}

/// CSS (logical) canvas size and **real** device pixel ratio for resolution sync.
#[cfg(target_arch = "wasm32")]
pub fn canvas_css_and_dpr() -> Option<(f32, f32, f32)> {
    use wasm_bindgen::JsCast;

    let win = web_sys::window()?;
    let doc = win.document()?;
    let el = doc.query_selector("canvas").ok().flatten()?;
    let canvas = el.dyn_into::<web_sys::HtmlCanvasElement>().ok()?;

    // Prefer the canvas client box (what the user sees / offsetX space).
    let mut css_w = canvas.client_width() as f32;
    let mut css_h = canvas.client_height() as f32;

    // Fallback to bounding rect if client is 0 during layout.
    if css_w < 2.0 || css_h < 2.0 {
        let rect = canvas.get_bounding_client_rect();
        css_w = rect.width() as f32;
        css_h = rect.height() as f32;
    }

    // visualViewport when canvas is full-bleed (mobile chrome / pinch).
    if let Some(vv) = win.visual_viewport() {
        let vw = vv.width() as f32;
        let vh = vv.height() as f32;
        // Only trust VV if it is close to the canvas (avoid double-counting UI chrome).
        if vw > 2.0
            && vh > 2.0
            && (css_w < 2.0 || ((vw - css_w).abs() < 8.0 && (vh - css_h).abs() < 8.0))
        {
            css_w = vw;
            css_h = vh;
        }
    }

    if css_w < 2.0 || css_h < 2.0 {
        if let Some(body) = doc.document_element() {
            css_w = body.client_width() as f32;
            css_h = body.client_height() as f32;
        }
    }
    if css_w < 2.0 || css_h < 2.0 {
        return None;
    }

    // MUST match winit's scale_factor (= devicePixelRatio). Do not clamp to 2.5 —
    // flagship phones are 3–3.5; a lower override scales all touches wrong.
    let dpr = (win.device_pixel_ratio() as f32).clamp(1.0, 4.0);
    Some((css_w, css_h, dpr))
}
