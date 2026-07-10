//! Window-relative UI scale so menus stay on-screen from phone → 1080p → 4K.

use bevy::prelude::*;

/// Multipliers for design-time sizes.
#[derive(Resource, Debug, Clone, Copy)]
pub struct UiScale {
    /// HUD / in-game text (boosted on small windows for readability).
    pub text: f32,
    /// Menu panels + menu body text (fits on screen; boosts on short/narrow).
    pub panel: f32,
    /// Current window aspect (width / height).
    pub aspect: f32,
    /// Design panel size used for this aspect class (world units).
    pub design: Vec2,
}

impl Default for UiScale {
    fn default() -> Self {
        Self {
            text: 1.0,
            panel: 1.0,
            aspect: 16.0 / 9.0,
            design: menu_design(16.0 / 9.0, 1080.0),
        }
    }
}

/// Text whose size tracks [`UiScale`].
#[derive(Component, Debug, Clone, Copy)]
pub struct ScaledText {
    pub base_px: f32,
    /// When true, use `panel` scale (menus). When false, use `text` (HUD).
    pub menu: bool,
}

/// Sprite panel whose `custom_size` tracks panel scale × design base.
#[derive(Component, Debug, Clone, Copy)]
pub struct ScaledPanel {
    /// Size at scale 1.0 (world units).
    pub base: Vec2,
}

/// Design-space position; reapplied when [`UiScale`] changes (responsive layouts).
#[derive(Component, Debug, Clone, Copy)]
pub struct ScaledPos {
    pub base: Vec2,
    pub menu: bool,
}

/// Pick a design panel that fits the shape of the screen.
/// Portrait = tall/narrow; short landscape = wide/short; else 16:9-ish.
pub fn menu_design(aspect: f32, window_h: f32) -> Vec2 {
    if aspect < 0.80 {
        // Phone portrait — stay inside narrow view with padding
        Vec2::new(360.0, 580.0)
    } else if window_h < 520.0 || aspect > 2.0 {
        // Phone landscape / very short window — short panel, large scale
        Vec2::new(800.0, 340.0)
    } else {
        // 1080p / 4K / tablet landscape
        Vec2::new(720.0, 520.0)
    }
}

/// Design fonts assume FixedVertical VIEW_HEIGHT = 1080 world units.
pub fn compute_ui_scale(window_width: f32, window_height: f32) -> UiScale {
    let h = window_height.max(200.0);
    let w = window_width.max(200.0);
    let aspect = w / h;
    let view_h = 1080.0;
    let view_w = view_h * aspect;
    let design = menu_design(aspect, h);

    // Fit design panel with ~10% padding on each axis.
    let fit = (0.90 * view_w / design.x).min(0.90 * view_h / design.y);

    // Big screens: comfortable, not huge. Short/narrow: allow higher scale so
    // body text stays ~14+ physical pixels.
    let comfort_max = if h >= 1000.0 {
        1.12
    } else if h >= 700.0 {
        1.35
    } else if h >= 500.0 {
        1.75
    } else {
        2.55
    };
    let panel = fit.min(comfort_max).clamp(0.42, 2.55);

    // HUD text: grow as the window shrinks so physical size stays legible.
    let text = (1080.0 / h).clamp(0.8, 2.35);

    UiScale {
        text,
        panel,
        aspect,
        design,
    }
}

pub fn sync_ui_scale(windows: Query<&Window>, mut scale: ResMut<UiScale>) {
    let Ok(window) = windows.single() else {
        return;
    };
    let next = compute_ui_scale(window.width(), window.height());
    if (next.text - scale.text).abs() > 0.01
        || (next.panel - scale.panel).abs() > 0.01
        || (next.aspect - scale.aspect).abs() > 0.02
        || (next.design - scale.design).length_squared() > 1.0
    {
        *scale = next;
    }
}

pub fn apply_scaled_text(scale: Res<UiScale>, mut q: Query<(&ScaledText, &mut TextFont)>) {
    if !scale.is_changed() && !scale.is_added() {
        return;
    }
    for (spec, mut font) in &mut q {
        let f = if spec.menu { scale.panel } else { scale.text };
        let px = (spec.base_px * f).clamp(11.0, 160.0);
        font.font_size = FontSize::Px(px);
    }
}

pub fn apply_scaled_panels(scale: Res<UiScale>, mut q: Query<(&ScaledPanel, &mut Sprite)>) {
    if !scale.is_changed() && !scale.is_added() {
        return;
    }
    for (spec, mut sprite) in &mut q {
        let size = spec.base * scale.panel;
        sprite.custom_size = Some(Vec2::new(
            size.x.clamp(160.0, 2400.0),
            size.y.clamp(120.0, 2000.0),
        ));
    }
}

pub fn apply_scaled_pos(scale: Res<UiScale>, mut q: Query<(&ScaledPos, &mut Transform)>) {
    if !scale.is_changed() && !scale.is_added() {
        return;
    }
    for (sp, mut tf) in &mut q {
        let s = if sp.menu { scale.panel } else { scale.text };
        tf.translation.x = sp.base.x * s;
        tf.translation.y = sp.base.y * s;
    }
}

pub fn confirm_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::NumpadEnter)
        || keyboard.just_pressed(KeyCode::Space)
}

pub fn back_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Backspace)
}

pub fn menu_up_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
        || keyboard.just_pressed(KeyCode::KeyK)
}

pub fn menu_down_just_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.just_pressed(KeyCode::ArrowDown)
        || keyboard.just_pressed(KeyCode::KeyS)
        || keyboard.just_pressed(KeyCode::KeyJ)
}

pub fn font(base_px: f32, scale: f32) -> TextFont {
    TextFont {
        font_size: FontSize::Px((base_px * scale).clamp(11.0, 160.0)),
        ..default()
    }
}
