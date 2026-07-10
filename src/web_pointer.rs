//! Web pointer / resolution helpers: keep touch & mouse coords aligned with the
//! canvas when DPR, browser zoom, or soft-caps desync Bevy's window size.

use bevy::prelude::*;

/// Remap a Bevy window-space pointer into coordinates that match the camera
/// viewport (logical window size). On native this is identity.
///
/// On WASM, browser zoom / visualViewport / DPR overrides can make Bevy's
/// reported touch position disagree with the painted canvas. We rescale from
/// the live canvas bounding box into the window's logical size.
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

    // If canvas display size matches Bevy logical size, keep as-is.
    if (rw - ww).abs() < 1.5 && (rh - wh).abs() < 1.5 {
        // Still correct for visualViewport offset when the canvas is shifted
        // (mobile URL bar / safe area). Bevy positions are relative to the
        // window origin; if winit already accounts for that, identity is fine.
        return Some(pos);
    }

    // Scale from canvas-space into Bevy logical window space.
    Some(Vec2::new(pos.x * (ww / rw), pos.y * (wh / rh)))
}

/// CSS (logical) canvas size and effective device pixel ratio for resolution sync.
#[cfg(target_arch = "wasm32")]
pub fn canvas_css_and_dpr() -> Option<(f32, f32, f32)> {
    use wasm_bindgen::JsCast;

    let win = web_sys::window()?;
    let doc = win.document()?;
    let el = doc.query_selector("canvas").ok().flatten()?;
    let canvas = el.dyn_into::<web_sys::HtmlCanvasElement>().ok()?;

    let mut css_w = canvas.client_width() as f32;
    let mut css_h = canvas.client_height() as f32;

    // Prefer visualViewport when present (accounts for mobile chrome / pinch zoom).
    if let Some(vv) = win.visual_viewport() {
        let vw = vv.width() as f32;
        let vh = vv.height() as f32;
        if vw > 2.0 && vh > 2.0 {
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

    // devicePixelRatio can jump with zoom; clamp for VRAM but keep ratio correct.
    let dpr = (win.device_pixel_ratio() as f32).clamp(1.0, 2.5);
    Some((css_w, css_h, dpr))
}
