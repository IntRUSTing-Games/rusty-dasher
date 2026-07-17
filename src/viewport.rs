//! Full-viewport canvas + playfield with equal (or handheld-chrome) margins.
//! Scales cleanly across: 4K, 1080p, tablet V/H, phone V/H.
//!
//! On handheld while playing, reserves a Game Boy / PSP-style control deck so
//! the virtual stick + dash button never cover the playfield.

use crate::components::MainCamera;
use crate::state::GameState;
use crate::ui_scale::{classify_viewport, ViewportClass};
use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::WindowResized;

/// World-space layout that tracks the real window aspect ratio.
#[derive(Resource, Clone, Copy, Debug)]
pub struct PlayBounds {
    /// Half-extents of the playable rectangle.
    pub half: Vec2,
    /// Center of the play rectangle (may shift when chrome is asymmetric).
    pub center: Vec2,
    /// Half-extents of the full camera view.
    pub view_half: Vec2,
    /// World units spanning full window height (camera FixedVertical).
    pub view_height: f32,
    /// Equal-ish average margin (legacy helpers / HUD).
    pub margin: f32,
    pub inset_left: f32,
    pub inset_right: f32,
    pub inset_top: f32,
    pub inset_bottom: f32,
    pub hud_top_y: f32,
    pub hud_bottom_y: f32,
    /// True when Game Boy / PSP chrome is active (playing on handheld).
    pub chrome: bool,
    pub class: ViewportClass,
}

impl Default for PlayBounds {
    fn default() -> Self {
        Self::compute(16.0 / 9.0, ViewportClass::Desktop1080, false)
    }
}

impl PlayBounds {
    /// Desktop reference world height.
    pub const VIEW_HEIGHT: f32 = 1080.0;

    /// Equal margin as a fraction of the shorter view half-axis (no chrome).
    const MARGIN_FRAC: f32 = 0.11;
    const MARGIN_MIN: f32 = 64.0;
    const MARGIN_MAX: f32 = 140.0;

    /// Choose orthographic height so short phone screens don't look microscopic.
    /// Lower world height → larger physical sprites on the same CSS pixels.
    pub fn view_height_for(class: ViewportClass) -> f32 {
        match class {
            // Landscape phones were ~2× tinier than portrait under FixedVertical 1080.
            ViewportClass::PhoneLandscape => 560.0,
            ViewportClass::PhonePortrait => 820.0,
            ViewportClass::TabletLandscape => 900.0,
            ViewportClass::TabletPortrait => 980.0,
            ViewportClass::Desktop1080 | ViewportClass::Desktop4k => Self::VIEW_HEIGHT,
        }
    }

    pub fn from_aspect(aspect: f32) -> Self {
        Self::compute(aspect, ViewportClass::Desktop1080, false)
    }

    pub fn compute(aspect: f32, class: ViewportClass, chrome: bool) -> Self {
        let aspect = aspect.clamp(0.45, 3.5);
        let view_h = Self::view_height_for(class);
        let view_w = view_h * aspect;
        let view_half = Vec2::new(view_w * 0.5, view_h * 0.5);

        let (inset_l, inset_r, inset_t, inset_b) = if chrome {
            chrome_insets(class, view_w, view_h)
        } else {
            let m = (view_half.x.min(view_half.y) * Self::MARGIN_FRAC)
                .clamp(Self::MARGIN_MIN, Self::MARGIN_MAX);
            (m, m, m, m)
        };

        let left = -view_half.x + inset_l;
        let right = view_half.x - inset_r;
        let bottom = -view_half.y + inset_b;
        let top = view_half.y - inset_t;
        let center = Vec2::new((left + right) * 0.5, (bottom + top) * 0.5);
        let half = Vec2::new(
            ((right - left) * 0.5).max(120.0),
            ((top - bottom) * 0.5).max(100.0),
        );

        let margin = (inset_l + inset_r + inset_t + inset_b) * 0.25;

        // HUD: score/hearts in the top margin band; status clear of the blue border
        // (V-PLAY-HUD-CLEAR — never sit mid-glyph on the field edge).
        let hud_top_y = top + (inset_t * 0.50).min(inset_t * 0.92).max(16.0);
        let hud_bottom_y = if chrome {
            // Sit well inside the bottom inset (deck / grip gap), not on the play border.
            // Midpoint of the strip between play bottom and view bottom.
            let strip_mid = (bottom + (-view_half.y)) * 0.5;
            // Prefer a bit above mid so text clears the stick deck edge on portrait.
            (strip_mid + inset_b * 0.12).clamp(-view_half.y + 14.0, bottom - 16.0)
        } else {
            // At least ~20 world units below the play bottom so "Dash READY" clears.
            let prefer = bottom - (inset_b * 0.55).clamp(20.0, inset_b * 0.92);
            prefer.min(bottom - 20.0)
        };
        // Clamp HUD into view
        let hud_top_y = hud_top_y.min(view_half.y - 8.0).max(top + 10.0);
        let hud_bottom_y = hud_bottom_y.max(-view_half.y + 8.0).min(bottom - 14.0);

        Self {
            half,
            center,
            view_half,
            view_height: view_h,
            margin,
            inset_left: inset_l,
            inset_right: inset_r,
            inset_top: inset_t,
            inset_bottom: inset_b,
            hud_top_y,
            hud_bottom_y,
            chrome,
            class,
        }
    }

    pub fn left(&self) -> f32 {
        self.center.x - self.half.x
    }
    pub fn right(&self) -> f32 {
        self.center.x + self.half.x
    }
    pub fn bottom(&self) -> f32 {
        self.center.y - self.half.y
    }
    pub fn top(&self) -> f32 {
        self.center.y + self.half.y
    }

    pub fn clamp(&self, pos: Vec3, radius: f32) -> Vec3 {
        Vec3::new(
            pos.x.clamp(self.left() + radius, self.right() - radius),
            pos.y.clamp(self.bottom() + radius, self.top() - radius),
            pos.z,
        )
    }

    pub fn random_pos(&self, margin: f32) -> Vec2 {
        use crate::util::rand_f32;
        Vec2::new(
            self.center.x + (rand_f32() * 2.0 - 1.0) * (self.half.x - margin).max(1.0),
            self.center.y + (rand_f32() * 2.0 - 1.0) * (self.half.y - margin).max(1.0),
        )
    }
}

/// Game Boy (portrait) / PSP (landscape) chrome insets in world units.
fn chrome_insets(class: ViewportClass, view_w: f32, view_h: f32) -> (f32, f32, f32, f32) {
    match class {
        // Game Boy: big bottom control deck, modest bezel elsewhere.
        ViewportClass::PhonePortrait | ViewportClass::TabletPortrait => {
            let deck = (view_h * 0.34).clamp(200.0, 340.0);
            let side = (view_w * 0.055).clamp(18.0, 48.0);
            let top = (view_h * 0.055).clamp(22.0, 52.0);
            (side, side, top, deck)
        }
        // PSP: grips left (stick) + right (dash), thin top/bottom bezel.
        ViewportClass::PhoneLandscape | ViewportClass::TabletLandscape => {
            let grip = (view_w * 0.20).clamp(110.0, 220.0);
            let vert = (view_h * 0.09).clamp(22.0, 48.0);
            (grip, grip, vert, vert)
        }
        _ => {
            let m = 64.0;
            (m, m, m, m)
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum FieldPiece {
    Backdrop,
    OuterBorder,
    InnerField,
    Glow,
}

pub fn spawn_camera(commands: &mut Commands) {
    commands.spawn((
        MainCamera,
        Camera2d,
        // Smooth vector edges on high-DPI / 4K
        Msaa::Sample4,
        Projection::from(OrthographicProjection {
            // Full-bleed: height fixed in world units; width follows aspect (phone → 4K).
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: PlayBounds::VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// Match the Bevy window framebuffer to the real browser canvas size × devicePixelRatio.
pub fn sync_resolution(
    mut windows: Query<&mut Window>,
    mut resize: MessageReader<WindowResized>,
    mut frames: Local<u32>,
) {
    *frames = frames.saturating_add(1);
    let resized = resize.read().count() > 0;
    let boot_resync = *frames <= 180 && (*frames == 1 || *frames % 20 == 0);
    if !resized && !boot_resync {
        return;
    }

    #[cfg(target_arch = "wasm32")]
    {
        sync_web_resolution(&mut windows);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Native: allow large windows up to 4K without artificial soft caps.
        for mut window in &mut windows {
            let w = window.resolution.physical_width();
            let h = window.resolution.physical_height();
            // Soft-cap only above 4K UHD so multi-monitor ultra-wides don't explode VRAM.
            const MAX_W: u32 = 3840;
            const MAX_H: u32 = 2160;
            if w > MAX_W || h > MAX_H {
                let s = (MAX_W as f32 / w as f32)
                    .min(MAX_H as f32 / h as f32)
                    .min(1.0);
                let nw = ((w as f32) * s).round().max(320.0) as u32;
                let nh = ((h as f32) * s).round().max(240.0) as u32;
                window.resolution.set_physical_resolution(nw, nh);
            }
        }
    }
}

/// Apply orthographic height + field piece geometry for the current [`PlayBounds`].
pub fn apply_bounds_geometry(
    bounds: &PlayBounds,
    cam_q: &mut Query<&mut Projection, With<MainCamera>>,
    pieces: &mut Query<(&FieldPiece, &mut Sprite, &mut Transform), Without<Mesh2d>>,
    glow: &mut Query<&mut Transform, (With<FieldPiece>, With<Mesh2d>)>,
) {
    if let Ok(mut proj) = cam_q.single_mut() {
        if let Projection::Orthographic(ortho) = proj.as_mut() {
            ortho.scaling_mode = ScalingMode::FixedVertical {
                viewport_height: bounds.view_height,
            };
        }
    }

    for (piece, mut sprite, mut tf) in pieces.iter_mut() {
        match *piece {
            FieldPiece::Backdrop => {
                sprite.custom_size = Some(bounds.view_half * 2.05);
                tf.translation = Vec3::new(0.0, 0.0, -6.0);
            }
            FieldPiece::OuterBorder => {
                // Single coherent rim: 6 world units outside the play half-extents.
                sprite.custom_size = Some(Vec2::new(
                    bounds.half.x * 2.0 + 12.0,
                    bounds.half.y * 2.0 + 12.0,
                ));
                tf.translation = bounds.center.extend(-4.0);
            }
            FieldPiece::InnerField => {
                sprite.custom_size = Some(bounds.half * 2.0);
                tf.translation = bounds.center.extend(-3.0);
            }
            FieldPiece::Glow => {}
        }
    }

    // Glow is a soft fill *inside* the field only. Scale to the short axis so it
    // never peeks out as half-disks past the blue border (V-PLAY-NO-WEIRD-POLYGONS).
    for mut tf in glow.iter_mut() {
        tf.translation = bounds.center.extend(-3.5);
        let s = bounds.half.x.min(bounds.half.y) * 0.92;
        tf.scale = Vec3::splat(s.clamp(40.0, 700.0));
    }
}

/// Recompute playfield size with equal margins or handheld chrome; resize field geometry.
pub fn sync_play_bounds(
    windows: Query<&Window>,
    mut bounds: ResMut<PlayBounds>,
    mut cam_q: Query<&mut Projection, With<MainCamera>>,
    mut pieces: Query<(&FieldPiece, &mut Sprite, &mut Transform), Without<Mesh2d>>,
    mut glow: Query<&mut Transform, (With<FieldPiece>, With<Mesh2d>)>,
    mut resize: MessageReader<WindowResized>,
    mut frames: Local<u32>,
    mut last_playing: Local<bool>,
    state: Res<State<GameState>>,
    ui: Res<crate::ui_scale::UiScale>,
) {
    *frames = frames.saturating_add(1);
    let resized = resize.read().count() > 0;
    let playing = *state.get() == GameState::Playing;
    let playing_flipped = playing != *last_playing;
    *last_playing = playing;

    if !resized && !playing_flipped && *frames != 1 && *frames % 15 != 0 {
        let want_chrome = playing && ui.class.is_handheld();
        if want_chrome == bounds.chrome && ui.class == bounds.class {
            return;
        }
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let w = window.width().max(32.0);
    let h = window.height().max(32.0);
    let class = classify_viewport(w, h);
    let chrome = playing && class.is_handheld();
    let next = PlayBounds::compute(w / h, class, chrome);

    if (next.half - bounds.half).length_squared() < 0.25
        && (next.view_half - bounds.view_half).length_squared() < 0.25
        && (next.center - bounds.center).length_squared() < 0.25
        && (next.margin - bounds.margin).abs() < 0.5
        && (next.view_height - bounds.view_height).abs() < 0.5
        && next.chrome == bounds.chrome
        && next.class == bounds.class
        && *frames > 2
        && !playing_flipped
    {
        return;
    }
    *bounds = next;
    apply_bounds_geometry(&bounds, &mut cam_q, &mut pieces, &mut glow);
}

/// Hide playfield border/glow under menu overlays so empty blue rects do not
/// ghost through (V-GHOST-FIELD). Backdrop stays as the dim scene base.
pub fn sync_field_overlay_visibility(
    state: Res<State<GameState>>,
    mut pieces: Query<(&FieldPiece, &mut Visibility)>,
) {
    let playing = *state.get() == GameState::Playing;
    for (piece, mut vis) in &mut pieces {
        let show = match *piece {
            FieldPiece::Backdrop => true,
            // Inner field + border + glow only while playing.
            FieldPiece::OuterBorder | FieldPiece::InnerField | FieldPiece::Glow => playing,
        };
        *vis = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Reposition HUD into the margin bands (outside the play rectangle).
pub fn sync_hud_layout(
    bounds: Res<PlayBounds>,
    mut score: Query<
        &mut Transform,
        (
            With<crate::components::HudScore>,
            Without<crate::components::HudLives>,
            Without<crate::components::HudCombo>,
            Without<crate::components::HudStatus>,
            Without<crate::components::HudLevel>,
        ),
    >,
    mut hearts: Query<
        (&crate::components::HudHeart, &mut Transform, &mut Sprite),
        (
            With<crate::components::HudLives>,
            Without<crate::components::HudScore>,
            Without<crate::components::HudCombo>,
            Without<crate::components::HudStatus>,
            Without<crate::components::HudLevel>,
        ),
    >,
    mut combo: Query<
        &mut Transform,
        (
            With<crate::components::HudCombo>,
            Without<crate::components::HudScore>,
            Without<crate::components::HudLives>,
            Without<crate::components::HudStatus>,
            Without<crate::components::HudLevel>,
        ),
    >,
    mut level: Query<
        &mut Transform,
        (
            With<crate::components::HudLevel>,
            Without<crate::components::HudScore>,
            Without<crate::components::HudLives>,
            Without<crate::components::HudCombo>,
            Without<crate::components::HudStatus>,
        ),
    >,
    mut status: Query<
        &mut Transform,
        (
            With<crate::components::HudStatus>,
            Without<crate::components::HudScore>,
            Without<crate::components::HudLives>,
            Without<crate::components::HudCombo>,
            Without<crate::components::HudLevel>,
        ),
    >,
    scale: Res<crate::ui_scale::UiScale>,
) {
    let top = bounds.hud_top_y;
    let bot = bounds.hud_bottom_y;
    // Keep HUD clear of the play border, inset slightly from the play screen edges
    let left = bounds.left() + 8.0;
    let right = bounds.right() - 8.0;

    let phone = scale.class.is_phone();
    let score_x = left + if phone { 4.0 } else { 12.0 };
    if let Ok(mut tf) = score.single_mut() {
        tf.translation = Vec3::new(score_x, top, 20.0);
    }
    let heart_size = (if phone { 22.0 } else { 28.0 }) * scale.text.clamp(0.75, 1.8);
    let spacing = heart_size + 6.0;
    for (heart, mut tf, mut sprite) in &mut hearts {
        let x = right - (2 - heart.index) as f32 * spacing;
        tf.translation = Vec3::new(x, top, 20.0);
        sprite.custom_size = Some(Vec2::splat(heart_size));
    }
    if let Ok(mut tf) = combo.single_mut() {
        tf.translation = Vec3::new(bounds.center.x, top, 20.0);
    }
    if let Ok(mut tf) = level.single_mut() {
        // Inside the playfield, below the top border — avoids clipping on the blue edge.
        // z=50: above world hazards (≈1.5) so mode|diff line stays readable (V-PLAY-HUD-CLEAR).
        let level_y = if phone || bounds.chrome {
            bounds.top() - if phone { 18.0 } else { 20.0 }
        } else {
            top - 26.0
        };
        tf.translation = Vec3::new(bounds.center.x, level_y, 50.0);
    }
    if let Ok(mut tf) = status.single_mut() {
        tf.translation = Vec3::new(bounds.center.x, bot, 20.0);
    }
}

#[cfg(target_arch = "wasm32")]
fn sync_web_resolution(windows: &mut Query<&mut Window>) {
    // CSS size from canvas / visualViewport (handles mobile chrome + zoom).
    let Some((css_w, css_h, dpr)) = crate::web_pointer::canvas_css_and_dpr() else {
        return;
    };

    // scale_factor MUST equal browser devicePixelRatio (what winit uses in
    // LogicalPosition::to_physical). Never invent a lower factor (e.g. old 2.5
    // clamp) — that multiplies every touch by real_dpr/our_sf and breaks hit tests.
    let scale_factor = dpr.clamp(1.0, 4.0);

    // physical = css × real DPR so logical = physical/sf = css (matches offsetX/Y).
    // Do NOT soft-cap the framebuffer in a way that changes physical without
    // changing sf — that shrinks logical size vs the painted CSS canvas and
    // misplaces chrome (stick/DASH). Cap only absurd multi-monitor extremes.
    let phys_w = (css_w * scale_factor).round().max(320.0).min(8192.0) as u32;
    let phys_h = (css_h * scale_factor).round().max(240.0).min(8192.0) as u32;

    for mut window in windows.iter_mut() {
        let cur_w = window.resolution.physical_width();
        let cur_h = window.resolution.physical_height();
        let cur_sf = window.resolution.scale_factor();
        if cur_w == phys_w && cur_h == phys_h && (cur_sf - scale_factor).abs() < 0.02 {
            continue;
        }
        window.resolution.set_scale_factor_override(Some(scale_factor));
        window.resolution.set_physical_resolution(phys_w, phys_h);
    }
}
