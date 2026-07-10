use crate::state::{Difficulty, GameMode};
use bevy::prelude::*;

#[derive(Resource, Default, Clone)]
pub struct GameStats {
    pub score: u32,
    pub lives: u32,
    pub combo: u32,
    pub best_combo: u32,
    pub stars_collected: u32,
    pub combo_timer: f32,
    pub elapsed: f32,
    /// Runtime intensity ramp (spawns/speeds), separate from chosen Difficulty.
    pub difficulty: f32,
    pub level: u32,
    pub level_target: u32,
    pub time_left: f32,
    pub mode: GameMode,
    pub chosen_difficulty: Difficulty,
    pub is_new_record: bool,
}

impl GameStats {
    pub fn for_mode(mode: GameMode, chosen: Difficulty) -> Self {
        let (lives, time_left) = match mode {
            GameMode::Classic => (3, 0.0),
            GameMode::Zen => (99, 0.0),
            GameMode::Survival => (1, 0.0),
            GameMode::Timed => (3, 60.0),
        };
        Self {
            lives,
            level: 1,
            level_target: 15,
            time_left,
            mode,
            chosen_difficulty: chosen,
            ..default()
        }
    }

    pub fn points_for_collect(&self) -> u32 {
        let base = (1 + self.combo / 3).min(10) as f32;
        let pts = (base * self.chosen_difficulty.score_mult()).round() as u32;
        pts.max(1)
    }

    pub fn speed_mult(&self) -> f32 {
        self.chosen_difficulty.speed_mult()
    }
}

/// Score thresholds for level-ups (classic/survival).
pub fn next_level_target(level: u32) -> u32 {
    15 + (level.saturating_sub(1)) * 20 + (level.saturating_sub(1)) * (level.saturating_sub(1)) * 3
}
