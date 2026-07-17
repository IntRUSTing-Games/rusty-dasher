//! On-screen Game Boy / PSP control chrome: shell, virtual stick, dash button.
//!
//! Visual placement is driven by [`PlayBounds`] world geometry (chrome insets),
//! not camera `viewport_to_world`. High fractional DPR (e.g. lab rodin 3.25)
//! can desync window logical size from the camera target long enough that
//! window-mapped stick/DASH collapse to the world origin mid-field while grip
//! shells (already bounds-based) look empty — V-FORM-FACTOR-CHROME /
//! V-PLAY-CONTROLS-OUTSIDE-FIELD / V-PLAY-NO-WEIRD-POLYGONS.

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
    ShellPrimary,
    ShellSecondary,
    StickBase,
    StickKnob,
    DashBtn,
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
    // Park off-screen until the first bounds-driven update (avoids a one-frame
    // mid-field stack of default 100×100 shells + circles at world origin).
    let park = Transform::from_xyz(0.0, -10_000.0, 14.5);

    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::ShellPrimary,
        // Opaque grips — translucent shells over the field caused V-PLAY-NO-SIDE-DIM-SLABS.
        Sprite::from_color(Color::srgb(0.07, 0.09, 0.14), Vec2::new(1.0, 1.0)),
        park,
    ));
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::ShellSecondary,
        Sprite::from_color(Color::srgb(0.07, 0.09, 0.14), Vec2::new(1.0, 1.0)),
        park,
    ));

    let (m, mat) = mesh_gfx::circle(
        meshes,
        materials,
        1.0,
        Color::srgba(0.22, 0.28, 0.42, 0.88),
    );
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::StickBase,
        m,
        mat,
        park.with_scale(Vec3::splat(1.0)),
    ));

    let (m, mat) = mesh_gfx::circle(
        meshes,
        materials,
        1.0,
        Color::srgba(0.45, 0.78, 1.0, 0.95),
    );
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::StickKnob,
        m,
        mat,
        park.with_scale(Vec3::splat(1.0)),
    ));

    let (m, mat) = mesh_gfx::circle(
        meshes,
        materials,
        1.0,
        Color::srgba(1.0, 0.42, 0.38, 0.94),
    );
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::DashBtn,
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
            font_size: FontSize::Px(13.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.94, 0.9)),
        Transform::from_xyz(0.0, -10_000.0, 18.0),
    ));
}

/// World-space stick / dash centers + radii from play chrome insets.
/// Portrait = Game Boy bottom deck; landscape = PSP side grips.
fn chrome_control_world(
    bounds: &PlayBounds,
    portrait: bool,
    swapped: bool,
) -> (Vec2, f32, Vec2, f32) {
    if portrait {
        // Deck is the bottom inset strip under the play rect.
        let deck_top = bounds.bottom() - 2.0;
        let deck_bot = -bounds.view_half.y;
        let deck_cy = (deck_top + deck_bot) * 0.5;
        let deck_h = (deck_top - deck_bot).max(24.0);
        let view_w = bounds.view_half.x * 2.0;
        let stick_r = (deck_h * 0.34).clamp(28.0, 72.0);
        let dash_r = (deck_h * 0.28).clamp(22.0, 56.0);
        let left = Vec2::new(-bounds.view_half.x + view_w * 0.28, deck_cy);
        let right = Vec2::new(-bounds.view_half.x + view_w * 0.75, deck_cy);
        if swapped {
            (right, stick_r, left, dash_r)
        } else {
            (left, stick_r, right, dash_r)
        }
    } else {
        // Left grip strip (outside play border) holds stick; right holds DASH.
        let left_l = -bounds.view_half.x;
        let left_r = bounds.left() - 2.0;
        let left_w = (left_r - left_l).max(8.0);
        let left_cx = left_l + left_w * 0.5;

        let right_l = bounds.right() + 2.0;
        let right_r = bounds.view_half.x;
        let right_w = (right_r - right_l).max(8.0);
        let right_cx = right_l + right_w * 0.5;

        // Slightly below vertical center (matches prior h*0.52 window layout).
        let cy = -bounds.view_half.y * 0.04;
        let stick_r = (left_w.min(right_w) * 0.38).clamp(28.0, 90.0);
        let dash_r = (left_w.min(right_w) * 0.32).clamp(22.0, 72.0);
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
    mut q: Query<(&ChromePart, &mut Transform, Option<&mut Sprite>), With<TouchChromeRoot>>,
) {
    if !layout.active {
        return;
    }
    // If play bounds have not yet applied chrome insets, synthesize them so we
    // never leave controls parked/stacked mid-field for a whole capture hold.
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
    let knob_r_world = stick_r_world * 0.48;
    // Knob offset is window-space (Y down). Convert with uniform world/window
    // scale from stick radius so we never need camera viewport mapping.
    let win_r = layout.stick_radius.max(1.0);
    let knob_world = stick_world
        + Vec2::new(
            controls.stick_knob_offset.x * (stick_r_world / win_r),
            -controls.stick_knob_offset.y * (stick_r_world / win_r),
        );
    let press = if controls.dash_held { 0.86 } else { 1.0 };

    for (part, mut tf, sprite) in &mut q {
        match *part {
            ChromePart::ShellPrimary => {
                if let Some(mut sprite) = sprite {
                    if layout.portrait {
                        // Game Boy bottom deck: fully outside the play rect.
                        let deck_top = place_bounds.bottom() - 2.0;
                        let deck_bot = -place_bounds.view_half.y;
                        let deck_cy = (deck_top + deck_bot) * 0.5;
                        let deck_h = (deck_top - deck_bot).max(24.0);
                        let deck_w = place_bounds.view_half.x * 2.0 + 4.0;
                        tf.translation = Vec3::new(0.0, deck_cy, 14.5);
                        sprite.custom_size = Some(Vec2::new(deck_w, deck_h + 4.0));
                        sprite.color = Color::srgb(0.07, 0.09, 0.14);
                    } else {
                        // PSP left grip: fill the chrome inset strip to the LEFT of
                        // the play border only (V-PLAY-NO-SIDE-DIM-SLABS /
                        // V-PLAY-SINGLE-BORDER). Never overlap the play rect.
                        let grip_right = place_bounds.left() - 2.0;
                        let grip_left = -place_bounds.view_half.x;
                        let world_w = (grip_right - grip_left).max(8.0);
                        let cx = grip_left + world_w * 0.5;
                        tf.translation = Vec3::new(cx, 0.0, 14.5);
                        sprite.custom_size =
                            Some(Vec2::new(world_w + 2.0, place_bounds.view_half.y * 2.08));
                        sprite.color = Color::srgb(0.07, 0.09, 0.14);
                    }
                }
            }
            ChromePart::ShellSecondary => {
                if let Some(mut sprite) = sprite {
                    if layout.portrait {
                        tf.translation =
                            Vec3::new(0.0, -place_bounds.view_half.y * 3.0, 14.5);
                        sprite.custom_size = Some(Vec2::splat(1.0));
                        sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
                    } else {
                        // PSP right grip: fill the strip to the RIGHT of the play border.
                        let grip_left = place_bounds.right() + 2.0;
                        let grip_right = place_bounds.view_half.x;
                        let world_w = (grip_right - grip_left).max(8.0);
                        let cx = grip_left + world_w * 0.5;
                        tf.translation = Vec3::new(cx, 0.0, 14.5);
                        sprite.custom_size =
                            Some(Vec2::new(world_w + 2.0, place_bounds.view_half.y * 2.08));
                        sprite.color = Color::srgb(0.07, 0.09, 0.14);
                    }
                }
            }
            ChromePart::StickBase => {
                tf.translation = stick_world.extend(16.0);
                tf.scale = Vec3::splat(stick_r_world);
            }
            ChromePart::StickKnob => {
                tf.translation = knob_world.extend(17.0);
                tf.scale = Vec3::splat(knob_r_world);
            }
            ChromePart::DashBtn => {
                tf.translation = dash_world.extend(16.0);
                tf.scale = Vec3::splat(dash_r_world * press);
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
        // Not collapsed to origin mid-field (the rodin landscape regression).
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
