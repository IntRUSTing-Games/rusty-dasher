//! Window-relative UI scale + viewport class for responsive layouts.
//!
//! Classification is by **CSS viewport** (logical pixels), not physical panel
//! resolution. Sources of truth for common sizes: StatCounter / industry
//! breakpoint practice (phone short≤500; tablets ~768–1180; laptops include
//! **1366×768** and **1536×864**; desktops 1920×1080+).
//!
//! Important: 1366×768 budget laptops are **desktop**, never tablet. Distinguish
//! landscape 16:9 laptops from 4:3 tablet landscape (1024×768) via aspect.

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

    /// Phone or tablet — touch-first layout (virtual stick + dash chrome).
    pub fn is_handheld(self) -> bool {
        matches!(
            self,
            Self::PhonePortrait
                | Self::PhoneLandscape
                | Self::TabletPortrait
                | Self::TabletLandscape
        )
    }

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

    /// Orthographic FixedVertical world height for this class.
    ///
    /// **Must match** the camera / [`crate::viewport::PlayBounds`] world height.
    /// Menu panel fit uses this so panels do not overflow the actual camera view
    /// (phone portrait/landscape use a shorter world than desktop 1080).
    pub fn view_height_world(self) -> f32 {
        match self {
            // Landscape phones were ~2× tinier than portrait under FixedVertical 1080.
            Self::PhoneLandscape => 560.0,
            Self::PhonePortrait => 820.0,
            Self::TabletLandscape => 900.0,
            Self::TabletPortrait => 980.0,
            Self::Desktop1080 | Self::Desktop4k => 1080.0,
        }
    }
}

/// Classify CSS/logical window size into a layout class.
///
/// Rules (CSS logical pixels — what Bevy/`window.width()` sees):
/// 1. **4K-class:** short ≥ 2000 or long ≥ 3000, or QHD-ish (short≥1400 & long≥2500)
/// 2. **Phone:** short ≤ 500 (covers 360×800 Android, 390×844 iPhone, etc.)
/// 3. **Phone landscape (short height):** landscape, aspect > 1.85, height < 560
/// 4. **Laptop / desktop:** landscape 16:10+ with height ≥ 700 and width ≥ 1200
///    (catches **1366×768**, 1280×720, 1440×900, 1536×864, 1600×900, 1920×1080),
///    OR width ≥ 1100 with height ≥ 650, OR short > 900 / long > 1400
/// 5. **Tablet:** remaining mid sizes (768×1024, 1024×768 4:3, 820×1180, …)
///
/// Why not `long > 1400` alone for desktop? **1366×768** has long=1366 and used
/// to fall into tablet landscape — that shipped stick chrome on real laptops.
pub fn classify_viewport(width: f32, height: f32) -> ViewportClass {
    let w = width.max(1.0);
    let h = height.max(1.0);
    let short = w.min(h);
    let long = w.max(h);
    let portrait = h >= w;
    let aspect = w / h;

    // 4K / ultra-wide large panels first.
    if short >= 2000.0 || long >= 3000.0 {
        return ViewportClass::Desktop4k;
    }
    if short >= 1400.0 && long >= 2500.0 {
        return ViewportClass::Desktop4k;
    }

    // Phones (CSS short side).
    if short <= 500.0 {
        return if portrait {
            ViewportClass::PhonePortrait
        } else {
            ViewportClass::PhoneLandscape
        };
    }

    // Ultra-wide short viewports (browser chrome / foldables) → phone landscape UI.
    if !portrait && aspect > 1.85 && h < 560.0 {
        return ViewportClass::PhoneLandscape;
    }

    // Laptops & desktops — including budget HD (1366×768) and Win 125% (1536×864).
    // Aspect ≥ 1.45 separates 16:10 / 16:9 laptops from 4:3 tablets (1024×768 ≈ 1.33).
    let landscape_laptop = !portrait
        && short >= 700.0
        && long >= 1200.0
        && aspect >= 1.45;
    let desktop_window = w >= 1100.0 && h >= 650.0;
    if landscape_laptop || desktop_window || short > 900.0 || long > 1400.0 {
        return ViewportClass::Desktop1080;
    }

    // Tablets (iPad classic, modern iPad CSS, small Android tablets).
    if portrait {
        ViewportClass::TabletPortrait
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

/// Design fonts scale against the **same** FixedVertical world height the camera uses.
/// Phone portrait/landscape use a shorter world (`ViewportClass::view_height_world`) so
/// fitting against 1080 would oversize menus and clip them on the laterals.
pub fn compute_ui_scale(window_width: f32, window_height: f32) -> UiScale {
    let h = window_height.max(200.0);
    let w = window_width.max(200.0);
    let aspect = w / h;
    let class = classify_viewport(w, h);
    // Match PlayBounds / camera orthographic height — not always 1080.
    let view_h = class.view_height_world();
    let view_w = view_h * aspect;
    let design = menu_design_for(class, aspect);

    // Fit design **plus** the 14px border ring with side padding so laterals never clip
    // (V-PANEL-IN-CANVAS / V-CLIP-TEXT). Phone portrait is the high-risk case.
    let pad = match class {
        ViewportClass::PhonePortrait | ViewportClass::PhoneLandscape => 0.84,
        ViewportClass::TabletPortrait | ViewportClass::TabletLandscape => 0.88,
        _ => 0.90,
    };
    // design is content; border is design+14 on each axis in base units at scale 1.
    let border_x = design.x + 14.0;
    let border_y = design.y + 14.0;
    let fit = (pad * view_w / border_x).min(pad * view_h / border_y);

    // Big screens: comfortable, not huge. Short/narrow: allow higher scale so
    // body text stays ~14+ physical pixels — but never above fit (clipping wins).
    let comfort_max = match class {
        ViewportClass::Desktop4k => 1.05,
        ViewportClass::Desktop1080 => 1.15,
        ViewportClass::TabletLandscape => 1.40,
        ViewportClass::TabletPortrait => 1.55,
        ViewportClass::PhoneLandscape => 1.85,
        ViewportClass::PhonePortrait => 1.90,
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
    fn classifies_core_targets() {
        assert_eq!(
            classify_viewport(390.0, 844.0),
            ViewportClass::PhonePortrait
        );
        assert_eq!(
            classify_viewport(844.0, 390.0),
            ViewportClass::PhoneLandscape
        );
        assert_eq!(
            classify_viewport(360.0, 800.0),
            ViewportClass::PhonePortrait
        );
        assert_eq!(
            classify_viewport(800.0, 360.0),
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
            classify_viewport(820.0, 1180.0),
            ViewportClass::TabletPortrait
        );
        assert_eq!(
            classify_viewport(1920.0, 1080.0),
            ViewportClass::Desktop1080
        );
        assert_eq!(classify_viewport(3840.0, 2160.0), ViewportClass::Desktop4k);
    }

    #[test]
    fn flagship_phone_css_not_physical_panel() {
        // Physical panels are ~1300×2800+; CSS viewports stay phone-class.
        assert_eq!(
            classify_viewport(440.0, 956.0),
            ViewportClass::PhonePortrait
        ); // iPhone 16 Pro Max CSS
        assert_eq!(
            classify_viewport(956.0, 440.0),
            ViewportClass::PhoneLandscape
        );
        assert_eq!(
            classify_viewport(412.0, 915.0),
            ViewportClass::PhonePortrait
        ); // Galaxy Ultra-class CSS
        assert_eq!(
            classify_viewport(915.0, 412.0),
            ViewportClass::PhoneLandscape
        );
        assert_eq!(
            classify_viewport(430.0, 932.0),
            ViewportClass::PhonePortrait
        );
    }

    #[test]
    fn laptops_are_desktop_not_tablet() {
        // Budget HD laptop — the regression that showed stick chrome on a friend's PC.
        assert_eq!(
            classify_viewport(1366.0, 768.0),
            ViewportClass::Desktop1080
        );
        assert_eq!(
            classify_viewport(1360.0, 768.0),
            ViewportClass::Desktop1080
        );
        // Windows 125% scale on a 1080p panel → ~1536×864 CSS.
        assert_eq!(
            classify_viewport(1536.0, 864.0),
            ViewportClass::Desktop1080
        );
        assert_eq!(
            classify_viewport(1280.0, 720.0),
            ViewportClass::Desktop1080
        );
        assert_eq!(
            classify_viewport(1440.0, 900.0),
            ViewportClass::Desktop1080
        );
        assert_eq!(
            classify_viewport(1600.0, 900.0),
            ViewportClass::Desktop1080
        );
        // 4:3 tablet landscape must stay tablet (not swept into laptop rule).
        assert_eq!(
            classify_viewport(1024.0, 768.0),
            ViewportClass::TabletLandscape
        );
    }

    #[test]
    fn handheld_flags() {
        assert!(ViewportClass::PhonePortrait.is_handheld());
        assert!(ViewportClass::TabletLandscape.is_handheld());
        assert!(!ViewportClass::Desktop1080.is_handheld());
        assert!(!ViewportClass::Desktop4k.is_handheld());
    }

    /// Regression: phone camera world height is shorter than 1080; menu fit must
    /// use that height so the blue panel border stays inside the canvas.
    #[test]
    fn phone_menu_panel_fits_camera_world() {
        for (w, h) in [
            (390.0, 844.0),
            (360.0, 800.0),
            (412.0, 915.0),
            (747.0, 1600.0),
            (844.0, 390.0),
            (800.0, 360.0),
        ] {
            let ui = compute_ui_scale(w, h);
            let view_h = ui.class.view_height_world();
            let view_w = view_h * ui.aspect;
            // Worst-case content uses ~design; border is design+14 base units.
            let border = ui.design + Vec2::new(14.0, 14.0);
            let world = border * ui.panel;
            assert!(
                world.x <= view_w + 0.5,
                "css {w}x{h}: border width {bw:.1} > view_w {view_w:.1} (panel scale {})",
                ui.panel,
                bw = world.x,
            );
            assert!(
                world.y <= view_h + 0.5,
                "css {w}x{h}: border height {bh:.1} > view_h {view_h:.1} (panel scale {})",
                ui.panel,
                bh = world.y,
            );
            // Lateral margin should remain visible (not flush).
            assert!(
                (view_w - world.x) * 0.5 >= 4.0,
                "css {w}x{h}: lateral margin too small"
            );
        }
    }
}
