//! Window-relative UI scale + viewport class for responsive layouts.
//!
//! Target surfaces (must all look good):
//!   - 4K desktop
//!   - 1080p desktop
//!   - Tablet vertical (portrait)
//!   - Tablet horizontal (landscape)
//!   - Phone vertical (portrait)
//!   - Phone horizontal (landscape)

use bevy::prelude::*;

/// Logical form-factor used for layout, chrome, and QA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportClass {
    PhonePortrait,
    PhoneLandscape,
    TabletPortrait,
    TabletLandscape,
    #[default]
    Desktop1080,
    Desktop4k,
}

impl ViewportClass {
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            Self::PhonePortrait => "phone_portrait",
            Self::PhoneLandscape => "phone_landscape",
            Self::TabletPortrait => "tablet_portrait",
            Self::TabletLandscape => "tablet_landscape",
            Self::Desktop1080 => "1080p",
            Self::Desktop4k => "4k",
        }
    }

    /// Phone or tablet — touch-first layout (point-to-move, second-finger dash).
    pub fn is_handheld(self) -> bool {
        matches!(
            self,
            Self::PhonePortrait
                | Self::PhoneLandscape
                | Self::TabletPortrait
                | Self::TabletLandscape
        )
    }

    #[allow(dead_code)]
    pub fn is_portrait(self) -> bool {
        matches!(self, Self::PhonePortrait | Self::TabletPortrait)
    }

    pub fn is_phone(self) -> bool {
        matches!(self, Self::PhonePortrait | Self::PhoneLandscape)
    }

    pub fn is_compact(self) -> bool {
        matches!(
            self,
            Self::PhonePortrait | Self::PhoneLandscape | Self::TabletPortrait
        )
    }
}

/// Classify CSS/logical window size into one of the six target surfaces.
///
/// Heuristics use short/long sides (orientation-independent) plus aspect:
/// - Phone: short side ≤ 500 CSS px (≈ iPhone / small Android)
/// - Tablet: short ≤ 900 and long ≤ 1400 (iPad / Android tablets)
/// - Else desktop; 4K when min dimension ≥ 2000 or max ≥ 3000
pub fn classify_viewport(width: f32, height: f32) -> ViewportClass {
    let w = width.max(1.0);
    let h = height.max(1.0);
    let short = w.min(h);
    let long = w.max(h);
    let portrait = h >= w;
    let aspect = w / h;

    // Desktop / large first by absolute size so a 4K window is never a "tablet".
    if short >= 2000.0 || long >= 3000.0 {
        return ViewportClass::Desktop4k;
    }
    if short > 900.0 || long > 1400.0 {
        // Large enough to be a laptop/monitor; still 1080-class.
        return if short >= 1400.0 && long >= 2500.0 {
            ViewportClass::Desktop4k
        } else {
            ViewportClass::Desktop1080
        };
    }

    // Handheld band
    if short <= 500.0 {
        return if portrait {
            ViewportClass::PhonePortrait
        } else {
            ViewportClass::PhoneLandscape
        };
    }

    // Tablet band (and small laptop windows treated as tablet landscape for layout)
    if portrait {
        ViewportClass::TabletPortrait
    } else if aspect > 1.85 && h < 600.0 {
        // Very wide short height → phone landscape class (foldables / browser chrome)
        ViewportClass::PhoneLandscape
    } else {
        ViewportClass::TabletLandscape
    }
}

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
    /// Logical form-factor for layout branching + touch chrome.
    pub class: ViewportClass,
}

impl Default for UiScale {
    fn default() -> Self {
        Self {
            text: 1.0,
            panel: 1.0,
            aspect: 16.0 / 9.0,
            design: menu_design_for(ViewportClass::Desktop1080, 16.0 / 9.0),
            class: ViewportClass::Desktop1080,
        }
    }
}

/// Design panel (world units) that fits the viewport class shape.
/// Phone designs stay compact so content is dense (less empty "garbo" padding).
pub fn menu_design_for(class: ViewportClass, aspect: f32) -> Vec2 {
    match class {
        // Tighter portrait: content-sized, not a tall empty card
        ViewportClass::PhonePortrait => Vec2::new(340.0, 420.0),
        ViewportClass::PhoneLandscape => Vec2::new(720.0, 300.0),
        ViewportClass::TabletPortrait => Vec2::new(460.0, 560.0),
        ViewportClass::TabletLandscape => Vec2::new(680.0, 440.0),
        ViewportClass::Desktop1080 | ViewportClass::Desktop4k => {
            if aspect < 0.80 {
                Vec2::new(400.0, 520.0)
            } else {
                Vec2::new(720.0, 500.0)
            }
        }
    }
}

/// Back-compat helper used by tests / older call sites.
#[allow(dead_code)]
pub fn menu_design(aspect: f32, window_h: f32) -> Vec2 {
    // Approximate class from aspect + height alone (no width).
    let class = if aspect < 0.80 {
        ViewportClass::PhonePortrait
    } else if window_h < 520.0 || aspect > 2.0 {
        ViewportClass::PhoneLandscape
    } else if window_h >= 2000.0 {
        ViewportClass::Desktop4k
    } else {
        ViewportClass::Desktop1080
    };
    menu_design_for(class, aspect)
}

/// Design fonts assume FixedVertical VIEW_HEIGHT = 1080 world units.
pub fn compute_ui_scale(window_width: f32, window_height: f32) -> UiScale {
    let h = window_height.max(200.0);
    let w = window_width.max(200.0);
    let aspect = w / h;
    let class = classify_viewport(w, h);
    let view_h = 1080.0;
    let view_w = view_h * aspect;
    let design = menu_design_for(class, aspect);

    // Fit design panel with ~10% padding on each axis.
    let fit = (0.90 * view_w / design.x).min(0.90 * view_h / design.y);

    // Big screens: comfortable, not huge. Short/narrow: allow higher scale so
    // body text stays ~14+ physical pixels.
    let comfort_max = match class {
        ViewportClass::Desktop4k => 1.05,
        ViewportClass::Desktop1080 => 1.15,
        ViewportClass::TabletLandscape => 1.40,
        ViewportClass::TabletPortrait => 1.55,
        ViewportClass::PhoneLandscape => 2.20,
        ViewportClass::PhonePortrait => 2.55,
    };
    let panel = fit.min(comfort_max).clamp(0.42, 2.55);

    // HUD text: grow as the window shrinks so physical size stays legible.
    let text = match class {
        ViewportClass::Desktop4k => 0.95,
        ViewportClass::Desktop1080 => 1.0,
        ViewportClass::TabletLandscape => (1080.0 / h).clamp(1.0, 1.45),
        ViewportClass::TabletPortrait => (1080.0 / h).clamp(1.05, 1.70),
        ViewportClass::PhoneLandscape => (1080.0 / h).clamp(1.2, 2.20),
        ViewportClass::PhonePortrait => (1080.0 / h).clamp(1.25, 2.35),
    };

    UiScale {
        text,
        panel,
        aspect,
        design,
        class,
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
        || next.class != scale.class
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_six_targets() {
        assert_eq!(
            classify_viewport(390.0, 844.0),
            ViewportClass::PhonePortrait
        );
        assert_eq!(
            classify_viewport(844.0, 390.0),
            ViewportClass::PhoneLandscape
        );
        assert_eq!(
            classify_viewport(768.0, 1024.0),
            ViewportClass::TabletPortrait
        );
        assert_eq!(
            classify_viewport(1024.0, 768.0),
            ViewportClass::TabletLandscape
        );
        assert_eq!(
            classify_viewport(1920.0, 1080.0),
            ViewportClass::Desktop1080
        );
        assert_eq!(classify_viewport(3840.0, 2160.0), ViewportClass::Desktop4k);
    }

    #[test]
    fn handheld_flags() {
        assert!(ViewportClass::PhonePortrait.is_handheld());
        assert!(ViewportClass::TabletLandscape.is_handheld());
        assert!(!ViewportClass::Desktop1080.is_handheld());
        assert!(!ViewportClass::Desktop4k.is_handheld());
    }
}
