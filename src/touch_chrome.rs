//! On-screen control chrome: Fortnite-style virtual joystick + dash button.
//!
//! Stick look (Fortnite mobile-inspired):
//!   - Soft translucent dark base disc
//!   - Bright white outer ring (rim)
//!   - Solid white/light thumb knob that tracks the finger
//!   - Floats in the chrome margin (no solid Game Boy deck panel under it)
//!
//! Layout still uses Game Boy bottom / PSP side chrome insets so controls stay
//! outside the playfield (V-PLAY-CONTROLS-OUTSIDE-FIELD).
//!
//! Visual placement is driven by [`PlayBounds`] world geometry (chrome insets),
//! not camera `viewport_to_world`.

use crate::components::PlayEntity;
use crate::mesh_gfx;
use crate::state::GameState;
use crate::touch_controls::{TouchChromeLayout, TouchControls};
use crate::viewport::PlayBounds;
use bevy::prelude::*;

#[derive(Component)]
pub struct TouchChromeRoot;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum ChromePart {
    /// Optional grip/deck fill — kept transparent for Fortnite floating stick.
    ShellPrimary,
    ShellSecondary,
    /// Soft fill under the stick (Fortnite translucent pad).
    StickBase,
    /// White rim ring around the stick base.
    StickRing,
    /// Thumb knob (follows finger).
    StickKnob,
    DashBtn,
    /// White rim around dash (matches stick language).
    DashRing,
    DashLabel,
}

/// Spawn / despawn chrome with Playing state on handheld.
pub fn sync_touch_chrome_presence(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    layout: Res<TouchChromeLayout>,
    existing: Query<Entity, With<TouchChromeRoot>>,
    state: Res<State<GameState>>,
) {
    let want = layout.active && *state.get() == GameState::Playing;
    let has = !existing.is_empty();

    if want && !has {
        spawn_chrome(&mut commands, &mut meshes, &mut materials);
    } else if !want && has {
        for e in &existing {
            commands.entity(e).despawn();
        }
    }
}

fn spawn_chrome(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    // Park off-screen until the first bounds-driven update.
    let park = Transform::from_xyz(0.0, -10_000.0, 14.5);

    // Transparent shells — Fortnite stick floats; we still reserve chrome insets
    // for hit layout, but no solid plastic deck under the controls.
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::ShellPrimary,
        Sprite::from_color(Color::srgba(0.0, 0.0, 0.0, 0.0), Vec2::new(1.0, 1.0)),
        park,
    ));
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::ShellSecondary,
        Sprite::from_color(Color::srgba(0.0, 0.0, 0.0, 0.0), Vec2::new(1.0, 1.0)),
        park,
    ));

    // Fortnite-like stick base: soft dark translucent disc.
    let (m, mat) = mesh_gfx::circle(
        meshes,
        materials,
        1.0,
        Color::srgba(0.05, 0.06, 0.08, 0.42),
    );
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::StickBase,
        m,
        mat,
        park.with_scale(Vec3::splat(1.0)),
    ));

    // White outer rim (iconic Fortnite mobile ring).
    let (m, mat) = mesh_gfx::ring(
        meshes,
        materials,
        0.86,
        Color::srgba(1.0, 1.0, 1.0, 0.72),
    );
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::StickRing,
        m,
        mat,
        park.with_scale(Vec3::splat(1.0)),
    ));

    // Thumb: solid-ish white disc (slightly cool).
    let (m, mat) = mesh_gfx::circle(
        meshes,
        materials,
        1.0,
        Color::srgba(0.96, 0.97, 1.0, 0.92),
    );
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::StickKnob,
        m,
        mat,
        park.with_scale(Vec3::splat(1.0)),
    ));

    // Dash: soft red fill + white rim (same language as stick).
    let (m, mat) = mesh_gfx::circle(
        meshes,
        materials,
        1.0,
        Color::srgba(0.95, 0.28, 0.32, 0.55),
    );
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::DashBtn,
        m,
        mat,
        park.with_scale(Vec3::splat(1.0)),
    ));

    let (m, mat) = mesh_gfx::ring(
        meshes,
        materials,
        0.86,
        Color::srgba(1.0, 1.0, 1.0, 0.65),
    );
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::DashRing,
        m,
        mat,
        park.with_scale(Vec3::splat(1.0)),
    ));

    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::DashLabel,
        Text2d::new("DASH"),
        TextFont {
            font_size: FontSize::Px(12.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.92)),
        Transform::from_xyz(0.0, -10_000.0, 18.0),
    ));
}

/// World-space stick / dash centers + radii from play chrome insets.
/// Portrait = bottom margin; landscape = side grips.
fn chrome_control_world(
    bounds: &PlayBounds,
    portrait: bool,
    swapped: bool,
) -> (Vec2, f32, Vec2, f32) {
    if portrait {
        let deck_top = bounds.bottom() - 2.0;
        let deck_bot = -bounds.view_half.y;
        let deck_cy = (deck_top + deck_bot) * 0.5;
        let deck_h = (deck_top - deck_bot).max(24.0);
        let view_w = bounds.view_half.x * 2.0;
        // Fortnite sticks are relatively large / fatty-finger friendly.
        let stick_r = (deck_h * 0.36).clamp(32.0, 78.0);
        let dash_r = (deck_h * 0.28).clamp(24.0, 58.0);
        let left = Vec2::new(-bounds.view_half.x + view_w * 0.28, deck_cy);
        let right = Vec2::new(-bounds.view_half.x + view_w * 0.75, deck_cy);
        if swapped {
            (right, stick_r, left, dash_r)
        } else {
            (left, stick_r, right, dash_r)
        }
    } else {
        let left_l = -bounds.view_half.x;
        let left_r = bounds.left() - 2.0;
        let left_w = (left_r - left_l).max(8.0);
        let left_cx = left_l + left_w * 0.5;

        let right_l = bounds.right() + 2.0;
        let right_r = bounds.view_half.x;
        let right_w = (right_r - right_l).max(8.0);
        let right_cx = right_l + right_w * 0.5;

        let cy = -bounds.view_half.y * 0.04;
        let stick_r = (left_w.min(right_w) * 0.40).clamp(30.0, 92.0);
        let dash_r = (left_w.min(right_w) * 0.32).clamp(24.0, 72.0);
        let left = Vec2::new(left_cx, cy);
        let right = Vec2::new(right_cx, cy);
        if swapped {
            (right, stick_r, left, dash_r)
        } else {
            (left, stick_r, right, dash_r)
        }
    }
}

/// Reposition chrome from [`PlayBounds`] chrome strips each frame.
pub fn update_touch_chrome_visuals(
    layout: Res<TouchChromeLayout>,
    bounds: Res<PlayBounds>,
    controls: Res<TouchControls>,
    mut q: Query<
        (
            &ChromePart,
            &mut Transform,
            Option<&mut Sprite>,
            Option<&mut MeshMaterial2d<ColorMaterial>>,
        ),
        With<TouchChromeRoot>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if !layout.active {
        return;
    }
    let place_bounds = if bounds.chrome {
        *bounds
    } else {
        PlayBounds::compute(
            (bounds.view_half.x / bounds.view_half.y.max(0.01)).clamp(0.45, 3.5),
            bounds.class,
            true,
        )
    };

    let (stick_world, stick_r_world, dash_world, dash_r_world) =
        chrome_control_world(&place_bounds, layout.portrait, layout.swapped);
    // Fortnite thumb is ~40–45% of base radius.
    let knob_r_world = stick_r_world * 0.42;
    let win_r = layout.stick_radius.max(1.0);
    let knob_world = stick_world
        + Vec2::new(
            controls.stick_knob_offset.x * (stick_r_world / win_r),
            -controls.stick_knob_offset.y * (stick_r_world / win_r),
        );
    let active = controls.stick_knob_offset.length_squared() > 1.0;
    let press = if controls.dash_held { 0.90 } else { 1.0 };

    // Active stick: slightly brighter rim/knob (Fortnite feedback).
    let base_a = if active { 0.50 } else { 0.42 };
    let ring_a = if active { 0.88 } else { 0.72 };
    let knob_a = if active { 0.98 } else { 0.92 };

    for (part, mut tf, sprite, mat_h) in &mut q {
        match *part {
            ChromePart::ShellPrimary | ChromePart::ShellSecondary => {
                // Fully transparent — floating Fortnite HUD, no solid deck/grips.
                if let Some(mut sprite) = sprite {
                    tf.translation = Vec3::new(0.0, -place_bounds.view_half.y * 3.0, 14.5);
                    sprite.custom_size = Some(Vec2::splat(1.0));
                    sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
                }
            }
            ChromePart::StickBase => {
                tf.translation = stick_world.extend(16.0);
                tf.scale = Vec3::splat(stick_r_world);
                if let Some(h) = mat_h {
                    if let Some(mut mat) = materials.get_mut(&h.0) {
                        mat.color = Color::srgba(0.05, 0.06, 0.08, base_a);
                    }
                }
            }
            ChromePart::StickRing => {
                tf.translation = stick_world.extend(16.5);
                tf.scale = Vec3::splat(stick_r_world);
                if let Some(h) = mat_h {
                    if let Some(mut mat) = materials.get_mut(&h.0) {
                        mat.color = Color::srgba(1.0, 1.0, 1.0, ring_a);
                    }
                }
            }
            ChromePart::StickKnob => {
                tf.translation = knob_world.extend(17.0);
                tf.scale = Vec3::splat(knob_r_world);
                if let Some(h) = mat_h {
                    if let Some(mut mat) = materials.get_mut(&h.0) {
                        mat.color = Color::srgba(0.96, 0.97, 1.0, knob_a);
                    }
                }
            }
            ChromePart::DashBtn => {
                tf.translation = dash_world.extend(16.0);
                tf.scale = Vec3::splat(dash_r_world * press);
                if let Some(h) = mat_h {
                    if let Some(mut mat) = materials.get_mut(&h.0) {
                        let a = if controls.dash_held { 0.72 } else { 0.55 };
                        mat.color = Color::srgba(0.95, 0.28, 0.32, a);
                    }
                }
            }
            ChromePart::DashRing => {
                tf.translation = dash_world.extend(16.5);
                tf.scale = Vec3::splat(dash_r_world * press);
                if let Some(h) = mat_h {
                    if let Some(mut mat) = materials.get_mut(&h.0) {
                        let a = if controls.dash_held { 0.90 } else { 0.65 };
                        mat.color = Color::srgba(1.0, 1.0, 1.0, a);
                    }
                }
            }
            ChromePart::DashLabel => {
                tf.translation = dash_world.extend(18.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_scale::ViewportClass;

    #[test]
    fn landscape_controls_sit_in_grip_strips_not_field() {
        let b = PlayBounds::compute(834.0 / 375.0, ViewportClass::PhoneLandscape, true);
        let (stick, stick_r, dash, dash_r) = chrome_control_world(&b, false, false);
        assert!(
            stick.x + stick_r < b.left(),
            "stick must clear play left: stick={} r={} left={}",
            stick.x,
            stick_r,
            b.left()
        );
        assert!(
            dash.x - dash_r > b.right(),
            "dash must clear play right: dash={} r={} right={}",
            dash.x,
            dash_r,
            b.right()
        );
        assert!(stick.x < -40.0 && dash.x > 40.0);
        assert!((stick.x - dash.x).abs() > 80.0);
    }

    #[test]
    fn portrait_controls_sit_in_bottom_deck() {
        let b = PlayBounds::compute(375.0 / 834.0, ViewportClass::PhonePortrait, true);
        let (stick, stick_r, dash, dash_r) = chrome_control_world(&b, true, false);
        assert!(stick.y + stick_r < b.bottom() + 4.0);
        assert!(dash.y + dash_r < b.bottom() + 4.0);
        assert!(stick.x < 0.0 && dash.x > 0.0);
    }
}
