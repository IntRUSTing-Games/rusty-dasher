use crate::components::{Player, PlayEntity, Pulse};
use crate::constants::*;
use crate::events::PlayerDashed;
use crate::mesh_gfx::{self, set_material_color};
use crate::stats::GameStats;
use crate::touch_controls::TouchControls;
use crate::util::clamp_to_field;
use crate::viewport::PlayBounds;
use bevy::prelude::*;

pub fn spawn_player(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) {
    // Flat solid disc (many-sided poly) — no center highlight artifact.
    let (mesh, mat) = mesh_gfx::poly(
        meshes,
        materials,
        PLAYER_RADIUS,
        28,
        Color::srgb(0.32, 0.80, 1.0),
    );
    commands.spawn((
        PlayEntity,
        Player {
            velocity: Vec2::ZERO,
            dash_timer: 0.0,
            dash_cooldown: 0.0,
            invuln: 1.25,
            magnet: 0.0,
            shield: 0.0,
            speed_boost: 0.0,
        },
        mesh,
        mat,
        Transform::from_xyz(0.0, 0.0, 2.0),
        Pulse {
            base_scale: 1.0,
            phase: 0.0,
            speed: 2.2,
        },
    ));
}

pub fn player_input(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    touch: Res<TouchControls>,
    bounds: Res<PlayBounds>,
    stats: Res<GameStats>,
    mut dash_w: MessageWriter<PlayerDashed>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    let Ok((mut player, mut transform)) = query.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    player.dash_cooldown = (player.dash_cooldown - dt).max(0.0);
    if player.dash_timer > 0.0 {
        player.dash_timer = (player.dash_timer - dt).max(0.0);
    }

    let mut dir = Vec2::ZERO;
    let mut touch_strength = 1.0;
    let mut using_touch_move = false;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }

    // Touch / mouse point-to-move overrides keyboard while a pointer is held.
    if touch.move_target.is_some() {
        using_touch_move = true;
        if touch.move_dir != Vec2::ZERO {
            dir = touch.move_dir;
            touch_strength = touch.move_strength;
        } else {
            dir = Vec2::ZERO;
            touch_strength = 0.0;
        }
    } else if touch.move_dir != Vec2::ZERO {
        dir = touch.move_dir;
        touch_strength = touch.move_strength.max(0.15);
        using_touch_move = true;
    }

    if dir != Vec2::ZERO {
        dir = dir.normalize();
    }

    let want_dash = keyboard.just_pressed(KeyCode::Space) || touch.dash_just;
    if want_dash && player.dash_cooldown <= 0.0 && player.dash_timer <= 0.0 {
        let dash_dir = if dir != Vec2::ZERO {
            dir
        } else if player.velocity.length_squared() > 1.0 {
            player.velocity.normalize()
        } else {
            Vec2::X
        };
        let spd = stats.speed_mult();
        player.velocity = dash_dir * DASH_SPEED * spd;
        player.dash_timer = DASH_DURATION;
        player.dash_cooldown = (DASH_COOLDOWN / spd).max(0.35);
        // Brief i-frames without a visual flash on the body (trail FX handles juice).
        player.invuln = player.invuln.max(DASH_DURATION + 0.05);
        dash_w.write(PlayerDashed {
            pos: transform.translation.truncate(),
            dir: dash_dir,
            color: status_color(&player),
        });
    } else if player.dash_timer <= 0.0 {
        let speed = PLAYER_SPEED
            * stats.speed_mult()
            * if player.speed_boost > 0.0 { 1.48 } else { 1.0 }
            * if using_touch_move { touch_strength } else { 1.0 };
        let target = dir * speed;
        // Arrive: ease out near the finger so we don't orbit the target.
        let lerp_rate = if using_touch_move && touch_strength < 0.45 {
            18.0
        } else {
            13.0
        };
        player.velocity = player.velocity.lerp(target, (lerp_rate * dt).min(1.0));
    }

    transform.translation += (player.velocity * dt).extend(0.0);
    clamp_to_field(&mut transform.translation, PLAYER_RADIUS, &bounds);
}

pub fn tick_player_fx(
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut player_q: Query<(
        &mut Player,
        &mut Transform,
        &mut Pulse,
        &MeshMaterial2d<ColorMaterial>,
    )>,
) {
    let Ok((mut player, mut transform, mut pulse, body_mat)) = player_q.single_mut() else {
        return;
    };
    let dt = time.delta_secs();

    player.invuln = (player.invuln - dt).max(0.0);
    player.magnet = (player.magnet - dt).max(0.0);
    player.shield = (player.shield - dt).max(0.0);
    player.speed_boost = (player.speed_boost - dt).max(0.0);

    // Gentle idle scale only — no dash pop, no flicker.
    pulse.phase += dt * pulse.speed;
    let breathe = 1.0 + pulse.phase.sin() * 0.04;
    transform.scale = Vec3::splat(pulse.base_scale * breathe);

    // Steady tints only. Invuln keeps normal colour (no ghost flash on dash).
    set_material_color(&mut materials, body_mat, status_color(&player));
}

/// Body / dash-trail colour from active power-ups (same palette everywhere).
pub fn status_color(player: &Player) -> Color {
    if player.shield > 0.0 {
        Color::srgb(0.35, 0.95, 0.7)
    } else if player.magnet > 0.0 {
        Color::srgb(0.75, 0.45, 1.0)
    } else if player.speed_boost > 0.0 {
        Color::srgb(1.0, 0.82, 0.4)
    } else {
        Color::srgb(0.35, 0.82, 1.0)
    }
}
