//! On-screen Game Boy / PSP control chrome: shell, virtual stick, dash button.

use crate::components::PlayEntity;
use crate::mesh_gfx;
use crate::state::GameState;
use crate::touch_controls::{window_to_world_approx, TouchChromeLayout, TouchControls};
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
    // Primary shell (portrait deck / landscape left grip)
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::ShellPrimary,
        Sprite::from_color(Color::srgba(0.07, 0.09, 0.14, 0.94), Vec2::new(100.0, 100.0)),
        Transform::from_xyz(0.0, 0.0, 14.5),
    ));
    // Secondary shell (landscape right grip; hidden in portrait)
    commands.spawn((
        PlayEntity,
        TouchChromeRoot,
        ChromePart::ShellSecondary,
        Sprite::from_color(Color::srgba(0.07, 0.09, 0.14, 0.94), Vec2::new(100.0, 100.0)),
        Transform::from_xyz(0.0, 0.0, 14.5),
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
        Transform::from_xyz(0.0, 0.0, 16.0).with_scale(Vec3::splat(40.0)),
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
        Transform::from_xyz(0.0, 0.0, 17.0).with_scale(Vec3::splat(22.0)),
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
        Transform::from_xyz(0.0, 0.0, 16.0).with_scale(Vec3::splat(32.0)),
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
        Transform::from_xyz(0.0, 0.0, 18.0),
    ));
}

/// Reposition chrome sprites from the screen-space layout each frame.
pub fn update_touch_chrome_visuals(
    windows: Query<&Window>,
    layout: Res<TouchChromeLayout>,
    bounds: Res<PlayBounds>,
    controls: Res<TouchControls>,
    mut q: Query<(&ChromePart, &mut Transform, Option<&mut Sprite>), With<TouchChromeRoot>>,
) {
    if !layout.active {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let wh = window.height().max(1.0);
    let ww = window.width().max(1.0);

    let stick_world = window_to_world_approx(layout.stick_center, window, &bounds);
    let stick_r_world = (layout.stick_radius / wh) * bounds.view_half.y * 2.0;
    let knob_r_world = stick_r_world * 0.48;
    let knob_win = layout.stick_center + controls.stick_knob_offset;
    let knob_world = window_to_world_approx(knob_win, window, &bounds);
    let dash_world = window_to_world_approx(layout.dash_center, window, &bounds);
    let dash_r_world = (layout.dash_radius / wh) * bounds.view_half.y * 2.0;
    let press = if controls.dash_held { 0.86 } else { 1.0 };

    for (part, mut tf, sprite) in &mut q {
        match *part {
            ChromePart::ShellPrimary => {
                if let Some(mut sprite) = sprite {
                    if layout.portrait {
                        let mid = Vec2::new(
                            (layout.deck_min.x + layout.deck_max.x) * 0.5,
                            (layout.deck_min.y + layout.deck_max.y) * 0.5,
                        );
                        let world = window_to_world_approx(mid, window, &bounds);
                        let w_size = layout.deck_max - layout.deck_min;
                        let world_size = Vec2::new(
                            (w_size.x / ww) * bounds.view_half.x * 2.0,
                            (w_size.y / wh) * bounds.view_half.y * 2.0,
                        );
                        tf.translation = world.extend(14.5);
                        sprite.custom_size = Some(world_size + Vec2::splat(4.0));
                        sprite.color = Color::srgba(0.07, 0.09, 0.14, 0.94);
                    } else {
                        // Left grip
                        let left_mid = Vec2::new(layout.stick_center.x, wh * 0.5);
                        let world = window_to_world_approx(left_mid, window, &bounds);
                        let grip_w = layout.stick_hit_radius * 2.5;
                        let world_w = (grip_w / ww) * bounds.view_half.x * 2.0;
                        tf.translation = world.extend(14.5);
                        sprite.custom_size = Some(Vec2::new(world_w, bounds.view_half.y * 2.08));
                        sprite.color = Color::srgba(0.07, 0.09, 0.14, 0.9);
                    }
                }
            }
            ChromePart::ShellSecondary => {
                if let Some(mut sprite) = sprite {
                    if layout.portrait {
                        // Hide secondary in portrait (park off-screen tiny)
                        tf.translation = Vec3::new(0.0, -bounds.view_half.y * 3.0, 14.5);
                        sprite.custom_size = Some(Vec2::splat(1.0));
                        sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
                    } else {
                        let right_mid = Vec2::new(layout.dash_center.x, wh * 0.5);
                        let world = window_to_world_approx(right_mid, window, &bounds);
                        let grip_w = layout.dash_hit_radius * 2.5;
                        let world_w = (grip_w / ww) * bounds.view_half.x * 2.0;
                        tf.translation = world.extend(14.5);
                        sprite.custom_size = Some(Vec2::new(world_w, bounds.view_half.y * 2.08));
                        sprite.color = Color::srgba(0.07, 0.09, 0.14, 0.9);
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
