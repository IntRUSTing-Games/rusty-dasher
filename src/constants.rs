use bevy::prelude::*;

pub const PLAYER_SPEED: f32 = 310.0;
pub const DASH_SPEED: f32 = 820.0;
pub const DASH_DURATION: f32 = 0.12;
pub const DASH_COOLDOWN: f32 = 0.8;
pub const PLAYER_RADIUS: f32 = 18.0;
pub const STAR_RADIUS: f32 = 14.0;
pub const HAZARD_RADIUS: f32 = 16.0;
pub const POWERUP_RADIUS: f32 = 15.0;
// Half-extents of the playfield (camera AutoMin keeps this fully visible)
#[allow(dead_code)]
pub const PLAY_AREA: Vec2 = Vec2::new(430.0, 250.0);
#[allow(dead_code)]
pub const MAX_LIVES: u32 = 3;
pub const COMBO_WINDOW: f32 = 1.65;
pub const WINDOW_W: u32 = 1920;
pub const WINDOW_H: u32 = 1080;
// HUD sits between playfield and window edge
#[allow(dead_code)]
pub const HUD_TOP_Y: f32 = 292.0;
#[allow(dead_code)]
pub const HUD_BOTTOM_Y: f32 = -298.0;
pub const SAVE_PATH: &str = "save_data.json";
