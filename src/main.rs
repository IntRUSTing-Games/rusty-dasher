//! RustyDasher
//!
//! ```text
//! cargo run                         # native desktop
//! cargo run -- --screenshots        # capture screens
//! cargo build --release             # native ship build
//! trunk serve                       # browser (WASM) at http://127.0.0.1:8080
//! trunk build --release             # static web dist/ for hosting
//! ```

mod audio_fx;
mod camera_fx;
mod components;
mod constants;
mod events;
mod game_assets;
mod mesh_gfx;
mod particles;
mod player;
mod save;
mod screenshot_tour;
mod state;
mod stats;
mod touch_controls;
mod ui;
mod ui_scale;
mod util;
mod viewport;
mod web_pointer;
mod world;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use camera_fx::ScreenShake;
use components::{GameOverUi, MenuUi, ModeUi};
use constants::{WINDOW_H, WINDOW_W};
use events::*;
use game_assets::GameAssets;
use state::{GameState, SelectedDifficulty, SelectedMode};
use stats::GameStats;
use touch_controls::TouchControls;
use ui_scale::UiScale;
use viewport::PlayBounds;

fn main() {
    // Avoid screenshot tour on wasm (no filesystem path that matches desktop QA flow)
    let screenshots = screenshot_tour::tour_enabled() && !cfg!(target_arch = "wasm32");

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "RustyDasher".into(),
                    resolution: WindowResolution::new(WINDOW_W, WINDOW_H),
                    resizable: true,
                    // CSS 100% fill of the page (resolution is fixed by viewport::sync_resolution)
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            })
            // Skip .meta files (we don't ship them; avoids 404 spam on web)
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::srgb(0.035, 0.04, 0.07)))
    .init_state::<GameState>()
    .init_resource::<GameStats>()
    .init_resource::<SelectedMode>()
    .init_resource::<SelectedDifficulty>()
    .init_resource::<ScreenShake>()
    .init_resource::<GameAssets>()
    .init_resource::<save::SaveData>()
    .init_resource::<TouchControls>()
    .init_resource::<PlayBounds>()
    .init_resource::<UiScale>()
    .init_resource::<particles::DashTrailAcc>()
    .add_message::<StarCollected>()
    .add_message::<PlayerHit>()
    .add_message::<PlayerDashed>()
    .add_message::<PowerupCollected>()
    .add_message::<LevelUp>()
    .add_systems(Startup, world::setup_field)
    // Menu
    .add_systems(
        OnEnter(GameState::Menu),
        (
            ui::spawn_menu,
            // Web browsers block autoplay; native can jingle immediately.
            audio_fx::sfx_menu_enter.run_if(|| !cfg!(target_arch = "wasm32")),
        ),
    )
    .add_systems(OnExit(GameState::Menu), util::despawn_with::<MenuUi>)
    // Mode select
    .add_systems(OnEnter(GameState::ModeSelect), ui::spawn_mode_select)
    .add_systems(OnExit(GameState::ModeSelect), util::despawn_with::<ModeUi>)
    // Playing
    .add_systems(OnEnter(GameState::Playing), world::start_run)
    .add_systems(OnExit(GameState::Playing), world::cleanup_play)
    // Game over
    .add_systems(
        OnEnter(GameState::GameOver),
        (ui::spawn_game_over, audio_fx::sfx_gameover_enter),
    )
    .add_systems(OnExit(GameState::GameOver), util::despawn_with::<GameOverUi>)
    .add_systems(
        Update,
        (
            ui_scale::sync_ui_scale,
            ui_scale::apply_scaled_text,
            ui_scale::apply_scaled_panels,
            ui_scale::apply_scaled_pos,
            ui::rebuild_menus_on_layout_change,
            viewport::sync_resolution,
            viewport::sync_play_bounds,
            viewport::sync_hud_layout,
            // Touch/mouse always first so the same frame can act on it
            touch_controls::update_touch_controls,
            ui::menu_input.run_if(in_state(GameState::Menu)),
            ui::mode_select_input.run_if(in_state(GameState::ModeSelect)),
            ui::game_over_input.run_if(in_state(GameState::GameOver)),
            (
                (
                    player::player_input,
                    player::tick_player_fx,
                    world::spawn_stars,
                    world::spawn_hazards,
                    world::spawn_powerups,
                    world::move_hazards,
                    world::animate_pickups,
                    world::magnet_pull,
                    world::collect_stars,
                    world::collect_powerups,
                    world::hit_hazards,
                    world::check_timed_end,
                    world::qa_matrix_force_gameover,
                ),
                (
                    particles::update_particles,
                    particles::update_shockwaves,
                    particles::update_float_text,
                    particles::on_dash_trail,
                    particles::dash_trail_while_moving,
                    particles::on_star_fx,
                    ui::update_hud,
                    ui::tick_level_banners,
                    ui::playing_escape,
                ),
            )
                .run_if(in_state(GameState::Playing)),
            (
                camera_fx::shake_on_events,
                camera_fx::apply_screen_shake,
                audio_fx::sfx_on_star,
                audio_fx::sfx_on_hit,
                audio_fx::sfx_on_dash,
                audio_fx::sfx_on_powerup,
                audio_fx::sfx_on_levelup,
            ),
        ),
    );

    if screenshots {
        app.init_resource::<screenshot_tour::ScreenshotTour>()
            .add_systems(Update, screenshot_tour::run_tour);
        info!("Screenshot tour mode — writing to ./screenshots/");
    }

    app.run();
}
