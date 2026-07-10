//! Lightweight mesh particles + trails + floating score pops.

use crate::components::{Particle, PlayEntity};
use crate::events::{PlayerDashed, StarCollected};
use crate::mesh_gfx::{self, set_material_color};
use crate::util::rand_f32;
use bevy::prelude::*;
use std::f32::consts::TAU;

#[derive(Component)]
pub struct FloatText {
    pub life: f32,
    pub max_life: f32,
    pub vel_y: f32,
}

#[derive(Component)]
pub struct Shockwave {
    pub life: f32,
    pub max_life: f32,
    pub grow: f32,
}

pub fn burst(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    origin: Vec2,
    color: Color,
    count: usize,
) {
    for _ in 0..count {
        let angle = rand_f32() * TAU;
        let speed = 90.0 + rand_f32() * 200.0;
        let vel = Vec2::new(angle.cos(), angle.sin()) * speed;
        let life = 0.28 + rand_f32() * 0.35;
        let radius = 2.0 + rand_f32() * 3.2;
        let c = color.to_srgba();
        let soft = Color::srgba(c.red, c.green, c.blue, 0.9);
        let (mesh, mat) = mesh_gfx::circle(meshes, materials, radius, soft);
        commands.spawn((
            PlayEntity,
            Particle {
                velocity: vel,
                life,
                max_life: life,
            },
            mesh,
            mat,
            Transform::from_xyz(origin.x, origin.y, 6.0),
        ));
    }
}

/// Expanding ring (collect / hit juice) — not a colour strobe.
pub fn shockwave(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    origin: Vec2,
    color: Color,
) {
    let c = color.to_srgba();
    let (mesh, mat) = mesh_gfx::circle(
        meshes,
        materials,
        8.0,
        Color::srgba(c.red, c.green, c.blue, 0.45),
    );
    commands.spawn((
        PlayEntity,
        Shockwave {
            life: 0.35,
            max_life: 0.35,
            grow: 9.0,
        },
        mesh,
        mat,
        Transform::from_xyz(origin.x, origin.y, 5.5),
    ));
}

pub fn float_score(commands: &mut Commands, origin: Vec2, points: u32, scale: f32) {
    let label = if points > 1 {
        format!("+{points}")
    } else {
        "+1".into()
    };
    commands.spawn((
        PlayEntity,
        FloatText {
            life: 0.7,
            max_life: 0.7,
            vel_y: 55.0,
        },
        Text2d::new(label),
        TextFont {
            font_size: FontSize::Px((22.0 * scale).clamp(14.0, 40.0)),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.92, 0.4)),
        Transform::from_xyz(origin.x, origin.y + 12.0, 12.0),
    ));
}

const PLAYER_TRAIL_R: f32 = 13.0;

fn trail_color(base: Color, alpha: f32) -> Color {
    let c = base.to_srgba();
    Color::srgba(c.red, c.green, c.blue, alpha)
}

/// Opening dash streak: largest bubble at the character, smaller ones trail behind,
/// all coasting **forward** with the dash so the effect propagates with motion.
fn dash_forward_burst(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    origin: Vec2,
    dir: Vec2,
    color: Color,
) {
    let dir = if dir.length_squared() > 0.01 {
        dir.normalize()
    } else {
        Vec2::X
    };
    for i in 0..6 {
        // i=0 at the character (big); higher i farther behind (smaller).
        let back = i as f32 * 11.0;
        let pos = origin - dir * back;
        let life = 0.12 + (5 - i) as f32 * 0.025;
        let r = (PLAYER_TRAIL_R - i as f32 * 1.35).max(3.5);
        let a = 0.48 - i as f32 * 0.055;
        // Forward velocity — near-character ghosts keep pace with the dash.
        let speed = crate::constants::DASH_SPEED * (0.88 - i as f32 * 0.06);
        let (mesh, mat) = mesh_gfx::circle(meshes, materials, r, trail_color(color, a));
        commands.spawn((
            PlayEntity,
            Particle {
                velocity: dir * speed.max(120.0),
                life,
                max_life: life,
            },
            mesh,
            mat,
            Transform::from_xyz(pos.x, pos.y, 1.7),
        ));
    }
}

/// Afterimage under the player (large near character) while the dash is active.
fn drop_dash_ghost(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    pos: Vec2,
    dir: Vec2,
    scale: f32,
    color: Color,
) {
    let dir = if dir.length_squared() > 0.01 {
        dir.normalize()
    } else {
        Vec2::ZERO
    };
    // Stay large next to the character.
    let r = (PLAYER_TRAIL_R * 0.95 * scale).max(5.0);
    let life = 0.14;
    let (mesh, mat) = mesh_gfx::circle(
        meshes,
        materials,
        r,
        trail_color(color, 0.4 * scale),
    );
    commands.spawn((
        PlayEntity,
        Particle {
            // Ride forward with the dash (slightly slower → soft trail).
            velocity: dir * (crate::constants::DASH_SPEED * 0.55),
            life,
            max_life: life,
        },
        mesh,
        mat,
        Transform::from_xyz(pos.x, pos.y, 1.65),
    ));
}

pub fn update_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(
        Entity,
        &mut Particle,
        &mut Transform,
        &MeshMaterial2d<ColorMaterial>,
    )>,
) {
    let dt = time.delta_secs();
    for (entity, mut p, mut transform, mat) in &mut query {
        p.life -= dt;
        if p.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation += (p.velocity * dt).extend(0.0);
        // Dash streaks (fast) coast more; small bursts still slow quickly.
        let drag = if p.velocity.length_squared() > 200_000.0 {
            1.0 - 1.4 * dt
        } else {
            1.0 - 2.8 * dt
        };
        p.velocity *= drag;
        let t = (p.life / p.max_life).clamp(0.0, 1.0);
        transform.scale = Vec3::splat(t);
        if let Some(mat_ref) = materials.get(&mat.0) {
            let c = mat_ref.color.to_srgba();
            set_material_color(
                &mut materials,
                mat,
                Color::srgba(c.red, c.green, c.blue, t * c.alpha),
            );
        }
    }
}

pub fn update_shockwaves(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut q: Query<(
        Entity,
        &mut Shockwave,
        &mut Transform,
        &MeshMaterial2d<ColorMaterial>,
    )>,
) {
    let dt = time.delta_secs();
    for (e, mut wave, mut tf, mat) in &mut q {
        wave.life -= dt;
        if wave.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        let t = (wave.life / wave.max_life).clamp(0.0, 1.0);
        let grown = 1.0 + (1.0 - t) * wave.grow;
        tf.scale = Vec3::splat(grown);
        if let Some(m) = materials.get(&mat.0) {
            let c = m.color.to_srgba();
            set_material_color(
                &mut materials,
                mat,
                Color::srgba(c.red, c.green, c.blue, t * 0.45),
            );
        }
    }
}

pub fn update_float_text(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut FloatText, &mut Transform, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    for (e, mut ft, mut tf, mut color) in &mut q {
        ft.life -= dt;
        if ft.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        tf.translation.y += ft.vel_y * dt;
        let t = (ft.life / ft.max_life).clamp(0.0, 1.0);
        let c = color.0.to_srgba();
        color.0 = Color::srgba(c.red, c.green, c.blue, t);
        tf.scale = Vec3::splat(0.85 + 0.25 * t);
    }
}

/// On dash start: fire a forward-propagating streak along the dash vector.
pub fn on_dash_trail(
    mut dashes: MessageReader<PlayerDashed>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for d in dashes.read() {
        let dir = if d.dir.length_squared() > 0.01 {
            d.dir.normalize()
        } else {
            Vec2::X
        };
        dash_forward_burst(
            &mut commands,
            &mut meshes,
            &mut materials,
            d.pos,
            dir,
            d.color,
        );
    }
}

#[derive(Resource, Default)]
pub struct DashTrailAcc(f32);

/// While dashing: keep dropping ghosts at the live player position so the
/// trail advances with the character instead of freezing at the start point.
pub fn dash_trail_while_moving(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut acc: ResMut<DashTrailAcc>,
    time: Res<Time>,
    player: Query<(&Transform, &crate::components::Player)>,
) {
    let Ok((tf, p)) = player.single() else {
        acc.0 = 0.0;
        return;
    };
    if p.dash_timer <= 0.0 {
        acc.0 = 0.0;
        return;
    }

    acc.0 += time.delta_secs();
    // Drop an afterimage under the player as they fly forward.
    const STEP: f32 = 0.018;
    while acc.0 >= STEP {
        acc.0 -= STEP;
        let dir = if p.velocity.length_squared() > 1.0 {
            p.velocity.normalize()
        } else {
            Vec2::ZERO
        };
        // Fade scale slightly as the dash winds down.
        let t = (p.dash_timer / crate::constants::DASH_DURATION).clamp(0.35, 1.0);
        drop_dash_ghost(
            &mut commands,
            &mut meshes,
            &mut materials,
            tf.translation.truncate(),
            dir,
            t,
            crate::player::status_color(p),
        );
    }
}


/// Shockwave + float text on star collect.
pub fn on_star_fx(
    mut stars: MessageReader<StarCollected>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    scale: Res<crate::ui_scale::UiScale>,
) {
    for s in stars.read() {
        // combo-scaled points shown by caller via combo; show combo tier pop
        let pts = (1 + s.combo / 3).min(10);
        shockwave(
            &mut commands,
            &mut meshes,
            &mut materials,
            s.pos,
            Color::srgb(1.0, 0.9, 0.35),
        );
        float_score(&mut commands, s.pos, pts, scale.text);
    }
}
