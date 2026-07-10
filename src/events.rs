#![allow(dead_code)]
use bevy::prelude::*;

#[derive(Message, Clone, Copy, Debug)]
pub struct StarCollected {
    pub pos: Vec2,
    pub combo: u32,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct PlayerHit {
    pub pos: Vec2,
    pub fatal: bool,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct PlayerDashed {
    pub pos: Vec2,
    /// Unit direction of the dash (where the character is going).
    pub dir: Vec2,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct PowerupCollected {
    pub pos: Vec2,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct LevelUp {
    pub level: u32,
}
