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
        Visibility::Inherited,
    ));

    // Soft center glow: stays *inside* the play rect (short-axis scale in
    // apply_bounds_geometry) so it never reads as half-disks outside the border.
    let (m, mat) = mesh_gfx::circle(
        &mut meshes,
        &mut materials,
        1.0,
        Color::srgba(0.22, 0.38, 0.78, 0.10),
    );
    let glow_r = bounds.half.x.min(bounds.half.y) * 0.92;
    commands.spawn((
        FieldDecor,
        FieldPiece::Glow,
        m,
        mat,
        // Between OuterBorder (-4) and InnerField (-3): visible as a soft tint
        // under the opaque field only if InnerField is slightly translucent —
        // keep under InnerField and very subtle so border edges stay clean.
        Transform::from_xyz(bounds.center.x, bounds.center.y, -3.5)
            .with_scale(Vec3::splat(glow_r.max(40.0))),
        Visibility::Hidden, // shown only while Playing (sync_field_overlay_visibility)
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
            Vec2::new(bounds.half.x * 2.0 + 12.0, bounds.half.y * 2.0 + 12.0),
        ),
        Transform::from_translation(bounds.center.extend(-4.0)),
        // Hidden under menus; revealed while Playing (V-GHOST-FIELD).
        Visibility::Hidden,
    ));
    commands.spawn((
        FieldDecor,
        FieldPiece::InnerField,
        Sprite::from_color(Color::srgb(0.05, 0.055, 0.09), bounds.half * 2.0),
        Transform::from_translation(bounds.center.extend(-3.0)),
        Visibility::Hidden,
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
    // Spawn *inside* the play border (not past it into Game Boy deck / PSP grips).
    // Prior OOB spawns made hazards free-float over chrome (V-PLAY-ENTITIES-IN-BOUNDS).
    // Reserve a top band so hazards don't birth on the in-field level HUD line
    // (V-PLAY-HUD-CLEAR / V-PLAY-HAZARD-NOT-ON-HUD). Triangle circumradius =
    // HAZARD_RADIUS — inset by full radius so vertices start inside the blue rect.
    let hud_band = if bounds.chrome { 42.0 } else { 22.0 };
    let inset = HAZARD_RADIUS + 4.0;
    let y_lo = bounds.bottom() + inset;
    let y_hi = (bounds.top() - inset - hud_band).max(y_lo + 8.0);
    let x_lo = bounds.left() + inset;
    let x_hi = bounds.right() - inset;
    let (pos, mut vel) = match side {
        0 => (
            Vec2::new(x_lo, rand_range(y_lo, y_hi)),
            Vec2::new(base_speed, rand_range(-55.0, 55.0)),
        ),
        1 => (
            Vec2::new(x_hi, rand_range(y_lo, y_hi)),
            Vec2::new(-base_speed, rand_range(-55.0, 55.0)),
        ),
        2 => (
            Vec2::new(rand_range(x_lo, x_hi), y_lo),
            Vec2::new(rand_range(-55.0, 55.0), base_speed),
        ),
        _ => (
            // Enter from below the HUD band, not on top of SURVIVAL|DIFF text.
            Vec2::new(rand_range(x_lo, x_hi), y_hi),
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
    // Despawn when the *body* would leave the play rect. Mesh is a regular polygon
    // with circumradius HAZARD_RADIUS. OuterBorder is a 6wu blue ring *outside*
    // play half-extents; keep an extra 4wu so tips stay clearly inside the dark
    // field (not on/through the blue stroke — V-PLAY-ENTITIES-IN-BOUNDS).
    // Prior positive "grace" let centers go outside, so tips floated over HUD/chrome.
    let pad = HAZARD_RADIUS + 4.0;
    let left = bounds.left() + pad;
    let right = bounds.right() - pad;
    let bottom = bounds.bottom() + pad;
    let top = bounds.top() - pad;
    for (entity, hazard, mut transform) in &mut query {
        transform.translation += (hazard.velocity * dt).extend(0.0);
        transform.rotate_z(hazard.spin * dt);
        let p = transform.translation.truncate();
        if p.x < left || p.x > right || p.y < bottom || p.y > top {
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
    bounds: Res<PlayBounds>,
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
            // Keep stars inside the play border (V-PLAY-ENTITIES-IN-BOUNDS).
            star_tf.translation = bounds.clamp(star_tf.translation, STAR_RADIUS);
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

/// Per-play timer for `?qa_matrix=1` forced Game Over (reset each enter Playing).
///
/// Uses **wall-clock** time (not Bevy virtual `Time`) so force-GO still fires when
/// parallel WASM tabs are rAF-throttled / delta-clamped under e2e concurrency.
#[derive(Resource, Default)]
pub struct QaMatrixForceTimer {
    /// Accumulated virtual time (debug / fallback when wall clock unavailable).
    pub elapsed: f32,
    pub done: bool,
    /// `performance.now()` at enter Playing (ms); `None` until first tick.
    /// WASM-only reads; kept on all targets so the resource shape is stable.
    #[allow(dead_code)]
    pub wall_start_ms: Option<f64>,
}

/// Reset force timer when a run starts.
///
/// Sticky wall-clock: if the player dies mid-run and Space restarts *before*
/// force-GO fired, keep the original `wall_start_ms` so force still lands at
/// the intended wall deadline (e2e was seeing go=timeout after accidental
/// restart reset the clock to zero). Only fully clear after a completed force
/// (`done`) so a deliberate post-GO "play again" gets a fresh window.
pub fn qa_matrix_reset_on_play(mut timer: ResMut<QaMatrixForceTimer>) {
    if timer.done || timer.wall_start_ms.is_none() {
        *timer = QaMatrixForceTimer::default();
        // Stamp wall start at Enter Playing (not first Update tick) so load lag
        // before the first Playing frame doesn't push force past e2e waits.
        #[cfg(target_arch = "wasm32")]
        {
            timer.wall_start_ms = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now());
        }
        return;
    }
    // Accidental restart before force: keep wall_start, clear virtual elapsed only.
    timer.elapsed = 0.0;
    timer.done = false;
}

/// When the page URL contains `?qa_matrix=1` (or `&qa_matrix=1`), force Game Over
/// after a short play so the visual QA matrix can capture that screen reliably.
///
/// Delay (seconds of *wall* time after entering Playing):
/// - `qa_go_ms=N` query override if present
/// - else `e2e=1` → **22.5s** (one continuous ≥20s play for e2e video)
/// - else **2.2s** (fast matrix-only / viewport_shots path)
///
/// No effect in normal play without `qa_matrix=1`.
pub fn qa_matrix_force_gameover(
    time: Res<Time>,
    mut timer: ResMut<QaMatrixForceTimer>,
    mut next: ResMut<NextState<GameState>>,
) {
    let Some(force_secs) = qa_matrix_force_secs() else {
        return;
    };
    if timer.done {
        return;
    }
    // Virtual delta as a floor (native + wasm fallback).
    timer.elapsed += time.delta_secs();

    let force_due = {
        #[cfg(target_arch = "wasm32")]
        {
            let now_ms = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now());
            if let Some(now) = now_ms {
                if timer.wall_start_ms.is_none() {
                    timer.wall_start_ms = Some(now);
                }
                let start = timer.wall_start_ms.unwrap_or(now);
                (now - start) >= f64::from(force_secs) * 1000.0
            } else {
                timer.elapsed >= force_secs
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            timer.elapsed >= force_secs
        }
    };

    if force_due {
        timer.done = true;
        next.set(GameState::GameOver);
    }
}

/// Publish `document.documentElement[data-rd-state]` for QA scripts
/// (`menu` | `mode_select` | `playing` | `game_over`).
/// Also `data-rd-dash-cd` = player dash cooldown seconds while Playing (else `"0"`)
/// so e2e can assert free multi-touch never fires a dash (I-NO-TWO-FINGER-GESTURE).
/// No-op on native.
pub fn publish_qa_state(
    state: Res<State<GameState>>,
    player_q: Query<&Player>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        let label = match state.get() {
            GameState::Menu => "menu",
            GameState::ModeSelect => "mode_select",
            GameState::Playing => "playing",
            GameState::GameOver => "game_over",
        };
        let dash_cd = if matches!(state.get(), GameState::Playing) {
            player_q
                .iter()
                .next()
                .map(|p| p.dash_cooldown)
                .unwrap_or(0.0)
        } else {
            0.0
        };
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
        {
            let _ = el.set_attribute("data-rd-state", label);
            // One decimal is enough for "was 0, stayed 0" e2e checks.
            let _ = el.set_attribute("data-rd-dash-cd", &format!("{dash_cd:.2}"));
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (state, player_q);
    }
}

/// Force delay in seconds when `qa_matrix=1` is present; `None` if disabled.
fn qa_matrix_force_secs() -> Option<f32> {
    #[cfg(target_arch = "wasm32")]
    {
        let search = web_sys::window()
            .and_then(|w| w.location().search().ok())
            .unwrap_or_default();
        if !search.contains("qa_matrix=1") {
            return None;
        }
        if let Some(ms) = query_param_u32(&search, "qa_go_ms") {
            // Clamp to a sane range (0.5s … 120s).
            return Some((ms as f32 / 1000.0).clamp(0.5, 120.0));
        }
        if search.contains("e2e=1") {
            // Continuous play long enough for ≥20s e2e video, then auto GO.
            return Some(22.5);
        }
        // Fast matrix PNG path (viewport_shots / short holds).
        Some(2.2)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

#[cfg(target_arch = "wasm32")]
fn query_param_u32(search: &str, key: &str) -> Option<u32> {
    let needle = format!("{key}=");
    let rest = search.split(&needle).nth(1)?;
    let raw = rest.split('&').next()?.trim();
    if raw.is_empty() {
        return None;
    }
    raw.parse().ok()
}
