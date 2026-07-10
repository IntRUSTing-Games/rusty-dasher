use crate::constants::SAVE_PATH;
use crate::state::GameMode;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub high_scores: HighScores,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HighScores {
    pub classic: u32,
    pub zen: u32,
    pub survival: u32,
    pub timed: u32,
}

impl HighScores {
    pub fn get(&self, mode: GameMode) -> u32 {
        match mode {
            GameMode::Classic => self.classic,
            GameMode::Zen => self.zen,
            GameMode::Survival => self.survival,
            GameMode::Timed => self.timed,
        }
    }

    pub fn set_if_better(&mut self, mode: GameMode, score: u32) -> bool {
        let slot = match mode {
            GameMode::Classic => &mut self.classic,
            GameMode::Zen => &mut self.zen,
            GameMode::Survival => &mut self.survival,
            GameMode::Timed => &mut self.timed,
        };
        if score > *slot {
            *slot = score;
            true
        } else {
            false
        }
    }
}

impl SaveData {
    pub fn load() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            return load_web();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            load_native()
        }
    }

    pub fn persist(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            persist_web(self);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            persist_native(self);
        }
    }
}

impl FromWorld for SaveData {
    fn from_world(_world: &mut World) -> Self {
        Self::load()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_native() -> SaveData {
    use std::fs;
    use std::path::Path;
    let path = Path::new(SAVE_PATH);
    if path.exists() {
        match fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|_| SaveData {
                high_scores: HighScores::default(),
            }),
            Err(_) => SaveData {
                high_scores: HighScores::default(),
            },
        }
    } else {
        SaveData {
            high_scores: HighScores::default(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_native(data: &SaveData) {
    use std::fs;
    if let Ok(text) = serde_json::to_string_pretty(data) {
        let _ = fs::write(SAVE_PATH, text);
    }
}

#[cfg(target_arch = "wasm32")]
fn load_web() -> SaveData {
    let Some(window) = web_sys::window() else {
        return SaveData {
            high_scores: HighScores::default(),
        };
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return SaveData {
            high_scores: HighScores::default(),
        };
    };
    match storage.get_item(SAVE_PATH) {
        Ok(Some(text)) => serde_json::from_str(&text).unwrap_or_else(|_| SaveData {
            high_scores: HighScores::default(),
        }),
        _ => SaveData {
            high_scores: HighScores::default(),
        },
    }
}

#[cfg(target_arch = "wasm32")]
fn persist_web(data: &SaveData) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    if let Ok(text) = serde_json::to_string(data) {
        let _ = storage.set_item(SAVE_PATH, &text);
    }
}
