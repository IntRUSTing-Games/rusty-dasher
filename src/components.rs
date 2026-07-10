use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub velocity: Vec2,
    pub dash_timer: f32,
    pub dash_cooldown: f32,
    pub invuln: f32,
    pub magnet: f32,
    pub shield: f32,
    pub speed_boost: f32,
}

#[derive(Component)]
pub struct Star;

#[derive(Component)]
pub struct Hazard {
    pub velocity: Vec2,
    pub spin: f32,
}

#[derive(Component)]
pub struct Powerup {
    pub kind: PowerupKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PowerupKind {
    Magnet,
    Shield,
    Speed,
}

#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub life: f32,
    pub max_life: f32,
}

#[derive(Component)]
pub struct Pulse {
    pub base_scale: f32,
    pub phase: f32,
    pub speed: f32,
}

#[derive(Component, Clone, Copy)]
pub struct MenuUi;

#[derive(Component, Clone, Copy)]
pub struct ModeUi;

#[derive(Component, Clone, Copy)]
pub struct GameOverUi;

#[derive(Component)]
pub struct PlayEntity;

#[derive(Component)]
pub struct FieldDecor;

#[derive(Component)]
pub struct HudScore;

#[derive(Component)]
pub struct HudLives;

/// One of the three life icons (0 = leftmost).
#[derive(Component)]
pub struct HudHeart {
    pub index: u32,
}

#[derive(Component)]
pub struct HudCombo;

#[derive(Component)]
pub struct HudStatus;

#[derive(Component)]
pub struct HudLevel;

#[derive(Component)]
pub struct LevelBanner {
    pub life: f32,
}

#[derive(Component)]
pub struct MainCamera;
