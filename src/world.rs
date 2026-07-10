use crate::components::*;
use crate::constants::*;
use crate::events::*;
use crate::game_assets::GameAssets;
use crate::particles;
use crate::state::{GameMode, GameState, SelectedDifficulty, SelectedMode};
use crate::stats::{next_level_target, GameStats};
use crate::util::{rand_f32, rand_range, random_field_pos};
use crate::mesh_gfx;
use crate::ui_scale::UiScale;
use crate::viewport::{FieldPiece, PlayBounds};
use bevy::prelude::*;
use std::f32::consts::TAU;
use std::time::Duration;

#[derive(Resource)]
pub struct StarSpawnTimer(pub Timer);

#[derive(Resource)]
pub struct HazardSpawnTimer(pub Timer);

#[derive(Resource)]
pub struct PowerupSpawnTimer(pub Timer);

#[derive(Resource)]
pub struct HurtCooldown(pub Timer);

pub fn setup_field(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    bounds: Res<PlayBounds>,
) {
    crate::viewport::spawn_camera(&mut commands);

    // Solid color rects scale perfectly (1x1 white texture + tint, not pixel art).
    commands.spawn((
        FieldDecor,
        FieldPiece::Backdrop,
        Sprite::from_color(Color::srgb(0.035, 0.04, 0.07), bounds.view_half * 2.2),
        Transform::from_xyz(0.0, 0.0, -6.0),
    ));

    // Soft glow: vector circle mesh
    // Unit circle; scaled in sync_play_bounds for crisp resize.
    let (m, mat) = mesh_gfx::circle(
        &mut meshes,
        &mut materials,
        1.0,
        Color::srgba(0.2, 0.35, 0.75, 0.12),
    );
    let glow_r = bounds.half.length().max(200.0) * 0.55;
    commands.spawn((
        FieldDecor,
        FieldPiece::Glow,
        m,
        mat,
        Transform::from_xyz(bounds.center.x, bounds.center.y, -5.0).with_scale(Vec3::splat(glow_r)),
    ));

    // Sparse ambient dots — cheap, soft backdrop (keep count modest for perf).
    for _ in 0..40 {
        let p = Vec2::new(
            (rand_f32() * 2.0 - 1.0) * bounds.view_half.x * 0.98,
            (rand_f32() * 2.0 - 1.0) * bounds.view_half.y * 0.98,
        );
        let r = 1.0 + rand_f32() * 1.8;
        let bright = 0.35 + rand_f32() * 0.45;
        let (m, mat) = mesh_gfx::circle(
            &mut meshes,
            &mut materials,
            r,
            Color::srgba(0.8, 0.88, 1.0, bright),
        );
        commands.spawn((FieldDecor, m, mat, Transform::from_xyz(p.x, p.y, -5.2)));
    }

    commands.spawn((
        FieldDecor,
        FieldPiece::OuterBorder,
        Sprite::from_color(
            Color::srgb(0.35, 0.5, 0.9),
            Vec2::new(bounds.half.x * 2.0 + 10.0, bounds.half.y * 2.0 + 10.0),
        ),
        Transform::from_translation(bounds.center.extend(-4.0)),
    ));
    commands.spawn((
        FieldDecor,
        FieldPiece::InnerField,
        Sprite::from_color(Color::srgb(0.05, 0.055, 0.09), bounds.half * 2.0),
        Transform::from_translation(bounds.center.extend(-3.0)),
    ));
}

pub fn start_run(
    mut commands: Commands,
    assets: Res<GameAssets>,
    selected: Res<SelectedMode>,
    chosen: Res<SelectedDifficulty>,
    mut stats: ResMut<GameStats>,
    mut bounds: ResMut<PlayBounds>,
    ui_scale: Res<UiScale>,
    windows: Query<&Window>,
    mut cam_q: Query<&mut Projection, With<crate::components::MainCamera>>,
    mut pieces: Query<
        (&crate::viewport::FieldPiece, &mut Sprite, &mut Transform),
        Without<Mesh2d>,
    >,
    mut glow: Query<
        &mut Transform,
        (With<crate::viewport::FieldPiece>, With<Mesh2d>),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    *stats = GameStats::for_mode(selected.0, chosen.0);
    let spd = stats.speed_mult();

    let star_iv = match selected.0 {
        GameMode::Zen => 0.85,
        GameMode::Survival => 1.0,
        GameMode::Timed => 0.75,
        GameMode::Classic => 1.05,
    } / spd;
    let haz_iv = match selected.0 {
        GameMode::Zen => 9999.0,
        GameMode::Survival => 1.1,
        GameMode::Timed => 1.35,
        GameMode::Classic => 1.75,
    } / spd;

    commands.insert_resource(StarSpawnTimer(Timer::new(
        Duration::from_secs_f32(star_iv),
        TimerMode::Repeating,
    )));
    commands.insert_resource(HazardSpawnTimer(Timer::new(
        Duration::from_secs_f32(haz_iv),
        TimerMode::Repeating,
    )));
    commands.insert_resource(PowerupSpawnTimer(Timer::new(
        Duration::from_secs_f32(7.0 / spd),
        TimerMode::Repeating,
    )));
    let mut hurt = Timer::from_seconds(0.85, TimerMode::Once);
    hurt.tick(Duration::from_secs_f32(10.0));
    commands.insert_resource(HurtCooldown(hurt));

    // Apply handheld chrome bounds + field geometry immediately so the player,
    // HUD, and blue play border land in the Game Boy / PSP screen (not under sticks).
    let class = ui_scale.class;
    let chrome = class.is_handheld();
    let aspect = windows
        .single()
        .map(|w| w.width().max(1.0) / w.height().max(1.0))
        .unwrap_or(16.0 / 9.0);
    *bounds = PlayBounds::compute(aspect, class, chrome);
    crate::viewport::apply_bounds_geometry(&bounds, &mut cam_q, &mut pieces, &mut glow);
    let play_bounds = *bounds;

    crate::player::spawn_player(&mut commands, &mut meshes, &mut materials, &play_bounds);
    crate::ui::spawn_hud(&mut commands, &stats, &play_bounds, &ui_scale, &assets);

    let starter = if selected.0 == GameMode::Zen { 6 } else { 4 };
    for _ in 0..starter {
        spawn_star(
            &mut commands,
            &mut meshes,
            &mut materials,
            random_field_pos(STAR_RADIUS * 3.0, &bounds),
        );
    }
}

pub fn cleanup_play(
    mut commands: Commands,
    // Roots only — children (glow/highlight) despawn with their parent.
    q: Query<Entity, (With<PlayEntity>, Without<ChildOf>)>,
    mut save: ResMut<crate::save::SaveData>,
    mut stats: ResMut<GameStats>,
) {
    for e in &q {
        commands.entity(e).despawn();
    }
    stats.is_new_record = save.high_scores.set_if_better(stats.mode, stats.score);
    save.persist();
}

fn spawn_star(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    pos: Vec2,
) {
    let (mesh, mat) = mesh_gfx::star(meshes, materials, STAR_RADIUS, Color::srgb(1.0, 0.86, 0.22));
    commands.spawn((
        PlayEntity,
        Star,
        mesh,
        mat,
        Transform::from_xyz(pos.x, pos.y, 1.0),
        Pulse {
            base_scale: 1.0,
            phase: rand_f32() * TAU,
            speed: 3.0 + rand_f32() * 1.5,
        },
    ));
}

pub fn spawn_stars(
    time: Res<Time>,
    mut timer: ResMut<StarSpawnTimer>,
    mut stats: ResMut<GameStats>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    bounds: Res<PlayBounds>,
    stars: Query<Entity, With<Star>>,
) {
    stats.elapsed += time.delta_secs();
    if stats.mode == GameMode::Timed {
        stats.time_left = (stats.time_left - time.delta_secs()).max(0.0);
    }

    stats.difficulty = match stats.mode {
        GameMode::Zen => 0.6 + stats.elapsed / 60.0,
        GameMode::Survival => (1.4 + stats.elapsed / 22.0 + stats.score as f32 / 30.0).min(4.5),
        GameMode::Timed => (1.2 + (60.0 - stats.time_left) / 25.0).min(3.2),
        GameMode::Classic => (1.0 + stats.elapsed / 35.0 + stats.score as f32 / 45.0 + (stats.level as f32 - 1.0) * 0.2)
            .min(3.8),
    };

    let spd = stats.speed_mult();
    let interval = match stats.mode {
        GameMode::Zen => (0.9 / (0.9 + stats.difficulty * 0.2)).max(0.35),
        _ => (1.1 / (0.85 + stats.difficulty * 0.28)).max(0.32),
    } / spd;
    timer.0.set_duration(Duration::from_secs_f32(interval));
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    let cap = match stats.mode {
        GameMode::Zen => 14,
        _ => (6 + (stats.difficulty * 2.0) as usize).min(14),
    };
    if stars.iter().count() >= cap {
        return;
    }
    spawn_star(
        &mut commands,
        &mut meshes,
        &mut materials,
        random_field_pos(STAR_RADIUS * 2.5, &bounds),
    );
}

pub fn spawn_hazards(
    time: Res<Time>,
    mut timer: ResMut<HazardSpawnTimer>,
    stats: Res<GameStats>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    bounds: Res<PlayBounds>,
    hazards: Query<Entity, With<Hazard>>,
    player: Query<&Transform, With<Player>>,
) {
    if stats.mode == GameMode::Zen {
        return;
    }
    let interval = ((1.85 / stats.difficulty) / stats.speed_mult()).clamp(0.28, 2.2);
    timer.0.set_duration(Duration::from_secs_f32(interval));
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    let cap = (4 + stats.difficulty as usize * 2).min(16);
    if hazards.iter().count() >= cap {
        return;
    }

    let player_pos = player
        .single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    let side = (rand_f32() * 4.0).floor() as i32;
    let base_speed = (95.0 + stats.difficulty * 60.0 + rand_f32() * 45.0) * stats.speed_mult();
    let m = (bounds.margin * 0.72).clamp(36.0, 100.0);
    let (pos, mut vel) = match side {
        0 => (
            Vec2::new(bounds.left() - m, rand_range(bounds.bottom(), bounds.top())),
            Vec2::new(base_speed, rand_range(-55.0, 55.0)),
        ),
        1 => (
            Vec2::new(bounds.right() + m, rand_range(bounds.bottom(), bounds.top())),
            Vec2::new(-base_speed, rand_range(-55.0, 55.0)),
        ),
        2 => (
            Vec2::new(rand_range(bounds.left(), bounds.right()), bounds.bottom() - m),
            Vec2::new(rand_range(-55.0, 55.0), base_speed),
        ),
        _ => (
            Vec2::new(rand_range(bounds.left(), bounds.right()), bounds.top() + m),
            Vec2::new(rand_range(-55.0, 55.0), -base_speed),
        ),
    };

    let to_player = (player_pos - pos).normalize_or_zero();
    let aim = match stats.mode {
        GameMode::Survival => (0.25 + stats.difficulty * 0.14).min(0.78),
        _ => (0.14 + stats.difficulty * 0.11).min(0.65),
    };
    vel = vel.lerp(to_player * base_speed, aim);

    let (mesh, mat) = mesh_gfx::poly(
        &mut meshes,
        &mut materials,
        HAZARD_RADIUS,
        3,
        Color::srgb(0.95, 0.28, 0.32),
    );
    commands.spawn((
        PlayEntity,
        Hazard {
            velocity: vel,
            spin: 2.2 + rand_f32() * 3.2,
        },
        mesh,
        mat,
        Transform::from_xyz(pos.x, pos.y, 1.5),
    ));
}

pub fn spawn_powerups(
    time: Res<Time>,
    mut timer: ResMut<PowerupSpawnTimer>,
    stats: Res<GameStats>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    bounds: Res<PlayBounds>,
    existing: Query<Entity, With<Powerup>>,
) {
    if stats.mode == GameMode::Zen {
        // rarer powerups in zen — mostly magnet/speed
        timer.0.set_duration(Duration::from_secs_f32(10.0 / stats.speed_mult()));
    }
    timer.0.tick(time.delta());
    if !timer.0.just_finished() || existing.iter().count() >= 2 {
        return;
    }

    let kind = match (rand_f32() * 3.0).floor() as i32 {
        0 => PowerupKind::Magnet,
        1 => PowerupKind::Shield,
        _ => PowerupKind::Speed,
    };
    let (sides, color) = match kind {
        PowerupKind::Magnet => (6, Color::srgb(0.72, 0.42, 1.0)),
        PowerupKind::Shield => (8, Color::srgb(0.35, 0.95, 0.68)),
        PowerupKind::Speed => (4, Color::srgb(1.0, 0.62, 0.28)),
    };
    let pos = random_field_pos(POWERUP_RADIUS * 3.0, &bounds);
    let (mesh, mat) = mesh_gfx::poly(&mut meshes, &mut materials, POWERUP_RADIUS, sides, color);
    commands.spawn((
        PlayEntity,
        Powerup { kind },
        mesh,
        mat,
        Transform::from_xyz(pos.x, pos.y, 1.2),
        Pulse {
            base_scale: 1.0,
            phase: rand_f32() * TAU,
            speed: 3.8,
        },
    ));
}

pub fn move_hazards(
    time: Res<Time>,
    bounds: Res<PlayBounds>,
    mut commands: Commands,
    mut query: Query<(Entity, &Hazard, &mut Transform)>,
) {
    let dt = time.delta_secs();
    // Allow travel through the equal margin; only despawn past the view edge.
    let limit = bounds.view_half + Vec2::splat(40.0);
    for (entity, hazard, mut transform) in &mut query {
        transform.translation += (hazard.velocity * dt).extend(0.0);
        transform.rotate_z(hazard.spin * dt);
        let p = transform.translation.truncate();
        if p.x.abs() > limit.x || p.y.abs() > limit.y {
            commands.entity(entity).despawn();
        }
    }
}

pub fn animate_pickups(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Pulse), Without<Player>>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut pulse) in &mut query {
        pulse.phase += dt * pulse.speed;
        // Gentle scale breathe — no harsh flashing.
        let s = pulse.base_scale * (1.0 + pulse.phase.sin() * 0.07);
        transform.scale = Vec3::splat(s);
        transform.rotate_z(dt * 0.7);
    }
}

pub fn magnet_pull(
    time: Res<Time>,
    player_q: Query<(&Transform, &Player)>,
    mut stars: Query<&mut Transform, (With<Star>, Without<Player>)>,
) {
    let Ok((player_tf, player)) = player_q.single() else {
        return;
    };
    if player.magnet <= 0.0 {
        return;
    }
    let player_pos = player_tf.translation.truncate();
    let dt = time.delta_secs();
    for mut star_tf in &mut stars {
        let pos = star_tf.translation.truncate();
        let dist = player_pos.distance(pos);
        if dist < 165.0 && dist > 1.0 {
            let dir = (player_pos - pos).normalize();
            let pull = 240.0 * (1.0 - dist / 165.0);
            star_tf.translation += (dir * pull * dt).extend(0.0);
        }
    }
}

pub fn collect_stars(
    mut commands: Commands,
    _assets: Res<GameAssets>,
    mut stats: ResMut<GameStats>,
    time: Res<Time>,
    ui_scale: Res<UiScale>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut star_w: MessageWriter<StarCollected>,
    mut level_w: MessageWriter<LevelUp>,
    player_q: Query<(&Transform, &Player)>,
    stars: Query<(Entity, &Transform), With<Star>>,
) {
    let Ok((player_tf, player)) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    if stats.combo > 0 {
        stats.combo_timer -= time.delta_secs();
        if stats.combo_timer <= 0.0 {
            stats.combo = 0;
        }
    }

    for (entity, star_tf) in &stars {
        let star_pos = star_tf.translation.truncate();
        let dist = player_pos.distance(star_pos);
        let grab = PLAYER_RADIUS + STAR_RADIUS
            + if player.magnet > 0.0 && dist < 55.0 {
                36.0
            } else {
                0.0
            };

        if dist < grab {
            commands.entity(entity).despawn();
            stats.combo += 1;
            stats.combo_timer = COMBO_WINDOW;
            stats.best_combo = stats.best_combo.max(stats.combo);
            stats.stars_collected += 1;
            let pts = stats.points_for_collect();
            stats.score += pts;

            star_w.write(StarCollected {
                pos: star_pos,
                combo: stats.combo,
            });
            particles::burst(
                &mut commands,
                &mut meshes,
                &mut materials,
                star_pos,
                Color::srgb(1.0, 0.9, 0.35),
                8,
            );

            // Level progression (classic/survival)
            if matches!(stats.mode, GameMode::Classic | GameMode::Survival)
                && stats.score >= stats.level_target
            {
                stats.level += 1;
                stats.level_target = next_level_target(stats.level);
                level_w.write(LevelUp { level: stats.level });
                crate::ui::spawn_level_banner(&mut commands, stats.level, ui_scale.text);
            }
        }
    }
}

pub fn collect_powerups(
    mut commands: Commands,
    _assets: Res<GameAssets>,
    mut stats: ResMut<GameStats>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut power_w: MessageWriter<PowerupCollected>,
    mut player_q: Query<(&Transform, &mut Player)>,
    powerups: Query<(Entity, &Transform, &Powerup)>,
) {
    let Ok((player_tf, mut player)) = player_q.single_mut() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    for (entity, tf, powerup) in &powerups {
        if player_pos.distance(tf.translation.truncate()) < PLAYER_RADIUS + POWERUP_RADIUS {
            match powerup.kind {
                PowerupKind::Magnet => player.magnet = 6.0,
                PowerupKind::Shield => player.shield = 5.5,
                PowerupKind::Speed => {
                    player.speed_boost = 4.2;
                    stats.combo = stats.combo.max(6);
                    stats.combo_timer = COMBO_WINDOW + 1.0;
                    stats.best_combo = stats.best_combo.max(stats.combo);
                }
            }
            power_w.write(PowerupCollected {
                pos: tf.translation.truncate(),
            });
            particles::burst(
                &mut commands,
                &mut meshes,
                &mut materials,
                tf.translation.truncate(),
                Color::srgb(0.7, 1.0, 0.9),
                7,
            );
            commands.entity(entity).despawn();
        }
    }
}

pub fn hit_hazards(
    time: Res<Time>,
    mut commands: Commands,
    _assets: Res<GameAssets>,
    mut stats: ResMut<GameStats>,
    mut cooldown: ResMut<HurtCooldown>,
    mut next: ResMut<NextState<GameState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hit_w: MessageWriter<PlayerHit>,
    mut player_q: Query<(&Transform, &mut Player)>,
    hazards: Query<&Transform, With<Hazard>>,
) {
    if stats.mode == GameMode::Zen {
        return;
    }
    cooldown.0.tick(time.delta());
    let Ok((player_tf, mut player)) = player_q.single_mut() else {
        return;
    };
    if player.invuln > 0.0 || !cooldown.0.is_finished() {
        return;
    }

    let player_pos = player_tf.translation.truncate();
    for hazard_tf in &hazards {
        if player_pos.distance(hazard_tf.translation.truncate()) < PLAYER_RADIUS + HAZARD_RADIUS {
            if player.shield > 0.0 {
                player.shield = 0.0;
                player.invuln = 0.85;
                cooldown.0.reset();
                particles::burst(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    player_pos,
                    Color::srgb(0.4, 1.0, 0.7),
                    6,
                );
                break;
            }

            stats.lives = stats.lives.saturating_sub(1);
            stats.combo = 0;
            player.invuln = 1.45;
            cooldown.0.reset();
            let fatal = stats.lives == 0;
            hit_w.write(PlayerHit {
                pos: player_pos,
                fatal,
            });
            particles::burst(
                &mut commands,
                &mut meshes,
                &mut materials,
                player_pos,
                Color::srgb(1.0, 0.3, 0.35),
                8,
            );
            if fatal {
                next.set(GameState::GameOver);
            }
            break;
        }
    }
}

pub fn check_timed_end(stats: Res<GameStats>, mut next: ResMut<NextState<GameState>>) {
    if stats.mode == GameMode::Timed && stats.time_left <= 0.0 {
        next.set(GameState::GameOver);
    }
}

/// When the page URL contains `?qa_matrix=1` (or `&qa_matrix=1`), force Game Over
/// shortly after play starts so the visual QA matrix can capture that screen
/// reliably. No effect in normal play.
pub fn qa_matrix_force_gameover(
    time: Res<Time>,
    mut elapsed: Local<f32>,
    mut done: Local<bool>,
    mut next: ResMut<NextState<GameState>>,
) {
    if *done || !qa_matrix_query_enabled() {
        return;
    }
    *elapsed += time.delta_secs();
    // Long enough for HUD/playfield to paint; short enough for CI.
    if *elapsed >= 2.2 {
        *done = true;
        next.set(GameState::GameOver);
    }
}

fn qa_matrix_query_enabled() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.location().search().ok())
            .map(|s| s.contains("qa_matrix=1"))
            .unwrap_or(false)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}
