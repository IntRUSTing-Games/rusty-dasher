use crate::events::{LevelUp, PlayerDashed, PlayerHit, PowerupCollected, StarCollected};
use crate::game_assets::{play_sfx, GameAssets};
use crate::state::GameState;
use bevy::prelude::*;

pub fn sfx_on_star(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut events: MessageReader<StarCollected>,
) {
    for _ in events.read() {
        play_sfx(&mut commands, assets.sfx_collect.clone());
    }
}

pub fn sfx_on_hit(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut events: MessageReader<PlayerHit>,
) {
    for _ in events.read() {
        play_sfx(&mut commands, assets.sfx_hit.clone());
    }
}

pub fn sfx_on_dash(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut events: MessageReader<PlayerDashed>,
) {
    for _ in events.read() {
        play_sfx(&mut commands, assets.sfx_dash.clone());
    }
}

pub fn sfx_on_powerup(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut events: MessageReader<PowerupCollected>,
) {
    for _ in events.read() {
        play_sfx(&mut commands, assets.sfx_powerup.clone());
    }
}

pub fn sfx_on_levelup(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut events: MessageReader<LevelUp>,
) {
    for _ in events.read() {
        play_sfx(&mut commands, assets.sfx_levelup.clone());
    }
}

pub fn sfx_menu_enter(mut commands: Commands, assets: Res<GameAssets>) {
    play_sfx(&mut commands, assets.sfx_menu.clone());
}

pub fn sfx_gameover_enter(mut commands: Commands, assets: Res<GameAssets>) {
    play_sfx(&mut commands, assets.sfx_gameover.clone());
}

pub fn _state_marker(_: Res<State<GameState>>) {}
