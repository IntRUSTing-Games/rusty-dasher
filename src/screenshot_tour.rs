//! Automated visual capture of every game screen.
//! Run: `cargo run -- --screenshots`

use crate::components::PlayEntity;
use crate::state::{GameMode, GameState, SelectedMode};
use crate::stats::GameStats;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Resource)]
pub struct ScreenshotTour {
    pub step: u32,
    pub timer: Timer,
    pub waiting_capture: bool,
    pub shots_done: u32,
}

impl Default for ScreenshotTour {
    fn default() -> Self {
        Self {
            step: 0,
            timer: Timer::new(Duration::from_secs_f32(0.9), TimerMode::Once),
            waiting_capture: false,
            shots_done: 0,
        }
    }
}

pub fn tour_enabled() -> bool {
    std::env::args().any(|a| a == "--screenshots")
}

pub fn take_shot(commands: &mut Commands, name: &str) {
    let dir = PathBuf::from("screenshots");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("{name}.png"));
    info!("Capturing screenshot → {}", path.display());
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}

pub fn run_tour(
    mut commands: Commands,
    time: Res<Time>,
    mut tour: ResMut<ScreenshotTour>,
    mut next: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
    mut selected: ResMut<SelectedMode>,
    mut stats: ResMut<GameStats>,
    mut exit: MessageWriter<AppExit>,
    play: Query<Entity, With<PlayEntity>>,
) {
    if tour.waiting_capture {
        // give capture a couple frames
        tour.timer.tick(time.delta());
        if tour.timer.is_finished() {
            tour.waiting_capture = false;
            tour.timer = Timer::new(Duration::from_secs_f32(0.55), TimerMode::Once);
            tour.step += 1;
        }
        return;
    }

    tour.timer.tick(time.delta());
    if !tour.timer.is_finished() {
        return;
    }

    match tour.step {
        0 => {
            // Ensure menu
            if *state.get() != GameState::Menu {
                next.set(GameState::Menu);
                tour.timer = Timer::new(Duration::from_secs_f32(0.6), TimerMode::Once);
                return;
            }
            take_shot(&mut commands, "01_menu");
            tour.shots_done += 1;
            tour.waiting_capture = true;
            tour.timer = Timer::new(Duration::from_secs_f32(0.45), TimerMode::Once);
        }
        1 => {
            next.set(GameState::ModeSelect);
            tour.timer = Timer::new(Duration::from_secs_f32(0.7), TimerMode::Once);
            tour.step += 1;
        }
        2 => {
            if *state.get() != GameState::ModeSelect {
                tour.timer = Timer::new(Duration::from_secs_f32(0.3), TimerMode::Once);
                return;
            }
            take_shot(&mut commands, "02_mode_select");
            tour.shots_done += 1;
            tour.waiting_capture = true;
            tour.timer = Timer::new(Duration::from_secs_f32(0.45), TimerMode::Once);
        }
        3 => {
            selected.0 = GameMode::Classic;
            next.set(GameState::Playing);
            tour.timer = Timer::new(Duration::from_secs_f32(1.4), TimerMode::Once);
            tour.step += 1;
        }
        4 => {
            if *state.get() != GameState::Playing {
                tour.timer = Timer::new(Duration::from_secs_f32(0.3), TimerMode::Once);
                return;
            }
            // let a few entities exist (hazards need a moment to spawn)
            take_shot(&mut commands, "03_playing_classic");
            tour.shots_done += 1;
            tour.waiting_capture = true;
            tour.timer = Timer::new(Duration::from_secs_f32(0.45), TimerMode::Once);
        }
        5 => {
            // Bump stats first; capture next step so HUD systems apply
            stats.score = 42;
            stats.combo = 9;
            stats.combo_timer = 2.0;
            stats.level = 2;
            stats.level_target = 35;
            stats.lives = 2;
            stats.difficulty = 1.8;
            tour.timer = Timer::new(Duration::from_secs_f32(0.35), TimerMode::Once);
            tour.step += 1;
        }
        6 => {
            take_shot(&mut commands, "04_playing_combo");
            tour.shots_done += 1;
            tour.waiting_capture = true;
            tour.timer = Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once);
        }
        7 => {
            stats.is_new_record = true;
            stats.score = 42;
            stats.best_combo = 9;
            stats.stars_collected = 20;
            stats.level = 2;
            next.set(GameState::GameOver);
            tour.timer = Timer::new(Duration::from_secs_f32(0.8), TimerMode::Once);
            tour.step += 1;
        }
        8 => {
            if *state.get() != GameState::GameOver {
                tour.timer = Timer::new(Duration::from_secs_f32(0.3), TimerMode::Once);
                return;
            }
            take_shot(&mut commands, "05_game_over");
            tour.shots_done += 1;
            tour.waiting_capture = true;
            tour.timer = Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once);
        }
        9 => {
            selected.0 = GameMode::Zen;
            next.set(GameState::Playing);
            tour.timer = Timer::new(Duration::from_secs_f32(1.0), TimerMode::Once);
            tour.step += 1;
        }
        10 => {
            if *state.get() != GameState::Playing {
                tour.timer = Timer::new(Duration::from_secs_f32(0.3), TimerMode::Once);
                return;
            }
            take_shot(&mut commands, "06_playing_zen");
            tour.shots_done += 1;
            tour.waiting_capture = true;
            tour.timer = Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once);
        }
        _ => {
            info!(
                "Screenshot tour complete ({} shots). Entities still live: {}",
                tour.shots_done,
                play.iter().count()
            );
            exit.write(AppExit::Success);
        }
    }
}
