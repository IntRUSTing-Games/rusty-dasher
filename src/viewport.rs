//! Full-viewport canvas + playfield with equal margins (preview strip for approaching hazards).
//! Scales cleanly from phone → 1080p → 4K.

use crate::components::MainCamera;
use bevy::camera::ScalingMode;
use bevy::prelude::*;
use bevy::window::WindowResized;

/// World-space layout that tracks the real window aspect ratio.
#[derive(Resource, Clone, Copy, Debug)]
pub struct PlayBounds {
    /// Half-extents of the playable rectangle.
    pub half: Vec2,
    /// Center of the play rectangle (kept at origin for equal margins).
    pub center: Vec2,
    /// Half-extents of the full camera view.
    pub view_half: Vec2,
    /// Equal world-space inset from every view edge to the play rectangle.
    pub margin: f32,
    pub hud_top_y: f32,
    pub hud_bottom_y: f32,
}

impl Default for PlayBounds {
    fn default() -> Self {
        Self::from_aspect(16.0 / 9.0)
    }
}

impl PlayBounds {
    /// World units spanning the full window height.
    /// ~1080 keeps UI/sprites feeling natural from 720p through 4K.
    pub const VIEW_HEIGHT: f32 = 1080.0;

    /// Equal margin as a fraction of the shorter view half-axis.
    /// Large enough to see hazards approach before they cross the blue border.
    const MARGIN_FRAC: f32 = 0.11;
    const MARGIN_MIN: f32 = 64.0;
    const MARGIN_MAX: f32 = 140.0;

    pub fn from_aspect(aspect: f32) -> Self {
        let aspect = aspect.clamp(0.45, 3.5);
        let view_h = Self::VIEW_HEIGHT;
        let view_w = view_h * aspect;
        let view_half = Vec2::new(view_w * 0.5, view_h * 0.5);

        // Same distance on left, right, top, and bottom.
        let margin = (view_half.x.min(view_half.y) * Self::MARGIN_FRAC)
            .clamp(Self::MARGIN_MIN, Self::MARGIN_MAX);

        let half = Vec2::new(
            (view_half.x - margin).max(140.0),
            (view_half.y - margin).max(120.0),
        );
        let center = Vec2::ZERO;

        // HUD sits in the equal margin bands (mid-line of each band).
        let hud_top_y = view_half.y - margin * 0.5;
        let hud_bottom_y = -view_half.y + margin * 0.5;

        Self {
            half,
            center,
            view_half,
            margin,
            hud_top_y,
            hud_bottom_y,
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

/// Recompute playfield size with equal margins; resize field geometry.
pub fn sync_play_bounds(
    windows: Query<&Window>,
    mut bounds: ResMut<PlayBounds>,
    mut cam_q: Query<&mut Projection, With<MainCamera>>,
    mut pieces: Query<(&FieldPiece, &mut Sprite, &mut Transform), Without<Mesh2d>>,
    mut glow: Query<&mut Transform, (With<FieldPiece>, With<Mesh2d>)>,
    mut resize: MessageReader<WindowResized>,
    mut frames: Local<u32>,
) {
    *frames = frames.saturating_add(1);
    let resized = resize.read().count() > 0;
    if !resized && *frames != 1 && *frames % 15 != 0 {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let w = window.width().max(32.0);
    let h = window.height().max(32.0);
    let next = PlayBounds::from_aspect(w / h);

    if (next.half - bounds.half).length_squared() < 0.25
        && (next.view_half - bounds.view_half).length_squared() < 0.25
        && (next.margin - bounds.margin).abs() < 0.5
        && *frames > 2
    {
        return;
    }
    *bounds = next;

    if let Ok(mut proj) = cam_q.single_mut() {
        if let Projection::Orthographic(ortho) = proj.as_mut() {
            ortho.scaling_mode = ScalingMode::FixedVertical {
                viewport_height: PlayBounds::VIEW_HEIGHT,
            };
        }
    }

    for (piece, mut sprite, mut tf) in &mut pieces {
        match *piece {
            FieldPiece::Backdrop => {
                sprite.custom_size = Some(bounds.view_half * 2.05);
                tf.translation = Vec3::new(0.0, 0.0, -6.0);
            }
            FieldPiece::OuterBorder => {
                sprite.custom_size = Some(Vec2::new(
                    bounds.half.x * 2.0 + 10.0,
                    bounds.half.y * 2.0 + 10.0,
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

    // Mesh glow: reposition (radius is approximate via scale from unit)
    for mut tf in &mut glow {
        // Only FieldPiece::Glow has Mesh2d among field pieces we care about
        tf.translation = bounds.center.extend(-5.0);
        let s = bounds.half.length().max(200.0) * 0.55;
        tf.scale = Vec3::splat(s.clamp(80.0, 900.0));
    }
}


/// Reposition HUD into the equal margin bands (outside the play rectangle).
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
    // Keep HUD clear of the play border, inset slightly from the view edge
    let left = -bounds.view_half.x + bounds.margin * 0.55;
    let right = bounds.view_half.x - bounds.margin * 0.55;

    if let Ok(mut tf) = score.single_mut() {
        tf.translation = Vec3::new(left + 70.0, top, 20.0);
    }
    let heart_size = 28.0 * scale.text.clamp(0.75, 1.8);
    let spacing = heart_size + 6.0;
    for (heart, mut tf, mut sprite) in &mut hearts {
        let x = right - 14.0 - (2 - heart.index) as f32 * spacing;
        tf.translation = Vec3::new(x, top, 20.0);
        sprite.custom_size = Some(Vec2::splat(heart_size));
    }
    if let Ok(mut tf) = combo.single_mut() {
        tf.translation = Vec3::new(0.0, top, 20.0);
    }
    if let Ok(mut tf) = level.single_mut() {
        tf.translation = Vec3::new(0.0, top - (bounds.margin * 0.28).clamp(18.0, 32.0), 20.0);
    }
    if let Ok(mut tf) = status.single_mut() {
        tf.translation = Vec3::new(0.0, bot, 20.0);
    }
}

#[cfg(target_arch = "wasm32")]
fn sync_web_resolution(windows: &mut Query<&mut Window>) {
    use wasm_bindgen::JsCast;

    let Some(win) = web_sys::window() else {
        return;
    };
    let Some(doc) = win.document() else {
        return;
    };
    let Ok(Some(el)) = doc.query_selector("canvas") else {
        return;
    };
    let Ok(canvas) = el.dyn_into::<web_sys::HtmlCanvasElement>() else {
        return;
    };

    let mut css_w = canvas.client_width();
    let mut css_h = canvas.client_height();
    if css_w < 2 || css_h < 2 {
        if let Some(body) = doc.document_element() {
            css_w = body.client_width();
            css_h = body.client_height();
        }
    }
    if css_w < 2 || css_h < 2 {
        return;
    }

    // Full device pixel ratio for sharp 4K (and retina 4K up to cap below).
    let dpr = win.device_pixel_ratio().clamp(1.0, 2.5) as f32;
    let mut phys_w = ((css_w as f32) * dpr).round() as u32;
    let mut phys_h = ((css_h as f32) * dpr).round() as u32;

    // Allow true 4K framebuffers; only soft-cap beyond UHD.
    const MAX_W: u32 = 3840;
    const MAX_H: u32 = 2160;
    if phys_w > MAX_W || phys_h > MAX_H {
        let s = (MAX_W as f32 / phys_w as f32)
            .min(MAX_H as f32 / phys_h as f32)
            .min(1.0);
        phys_w = ((phys_w as f32) * s).round().max(320.0) as u32;
        phys_h = ((phys_h as f32) * s).round().max(240.0) as u32;
    }

    for mut window in windows.iter_mut() {
        let cur_w = window.resolution.physical_width();
        let cur_h = window.resolution.physical_height();
        let cur_sf = window.resolution.scale_factor();
        if cur_w == phys_w && cur_h == phys_h && (cur_sf - dpr).abs() < 0.01 {
            continue;
        }
        window.resolution.set_scale_factor_override(Some(dpr));
        window.resolution.set_physical_resolution(phys_w, phys_h);
    }
}
