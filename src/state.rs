use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    ModeSelect,
    Playing,
    GameOver,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    Classic,
    Zen,
    Survival,
    Timed,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Classic => "CLASSIC",
            Self::Zen => "ZEN",
            Self::Survival => "SURVIVAL",
            Self::Timed => "TIMED 60s",
        }
    }

    pub fn blurb(self) -> &'static str {
        self.blurb_short()
    }

    pub fn blurb_short(self) -> &'static str {
        match self {
            Self::Classic => "3 hearts, levels, power-ups",
            Self::Zen => "No hazards - chill collecting",
            Self::Survival => "1 heart - high stakes",
            Self::Timed => "60 seconds - score attack",
        }
    }

    pub const ALL: [GameMode; 4] = [
        GameMode::Classic,
        GameMode::Zen,
        GameMode::Survival,
        GameMode::Timed,
    ];
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Insane,
}

impl Difficulty {
    pub const ALL: [Difficulty; 4] = [
        Difficulty::Easy,
        Difficulty::Normal,
        Difficulty::Hard,
        Difficulty::Insane,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Easy => "EASY",
            Self::Normal => "NORMAL",
            Self::Hard => "HARD",
            Self::Insane => "INSANE",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Self::Easy => "Slower game, fewer points per star",
            Self::Normal => "Standard speed and score",
            Self::Hard => "Faster game, more points per star",
            Self::Insane => "Very fast, highest points per star",
        }
    }

    pub fn score_mult(self) -> f32 {
        match self {
            Self::Easy => 0.75,
            Self::Normal => 1.0,
            Self::Hard => 1.5,
            Self::Insane => 2.5,
        }
    }

    pub fn speed_mult(self) -> f32 {
        match self {
            Self::Easy => 0.85,
            Self::Normal => 1.0,
            Self::Hard => 1.3,
            Self::Insane => 1.65,
        }
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub struct SelectedMode(pub GameMode);

#[derive(Resource, Default, Clone, Copy)]
pub struct SelectedDifficulty(pub Difficulty);
