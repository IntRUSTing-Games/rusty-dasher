use crate::components::*;
use crate::save::SaveData;
use crate::state::{Difficulty, GameMode, GameState, SelectedDifficulty, SelectedMode};
use crate::stats::GameStats;
use crate::touch_controls::TouchControls;
use crate::ui_scale::{
    back_just_pressed, confirm_just_pressed, font, menu_down_just_pressed, menu_up_just_pressed,
    ScaledPanel, ScaledPos, ScaledText, UiScale, ViewportClass,
};
use crate::game_assets::GameAssets;
use crate::viewport::PlayBounds;
use bevy::prelude::*;
use bevy::sprite::Anchor;

fn panel_border(base: Vec2) -> Vec2 {
    base + Vec2::new(14.0, 14.0)
}

pub fn spawn_menu(mut commands: Commands, save: Res<SaveData>, scale: Res<UiScale>) {
    spawn_menu_with(&mut commands, &save, &scale);
}

fn spawn_menu_with(commands: &mut Commands, save: &SaveData, scale: &UiScale) {
    let best = save.high_scores.classic.max(
        save.high_scores
            .zen
            .max(save.high_scores.survival.max(save.high_scores.timed)),
    );
    let s = scale.panel;
    let compact = scale.class.is_compact() || scale.aspect < 0.85;
    // Fit inside design with margin so body text never kisses the border.
    let panel = if compact {
        Vec2::new(
            (scale.design.x * 0.96).clamp(320.0, 480.0),
            (scale.design.y * 0.90).clamp(300.0, 560.0),
        )
    } else {
        Vec2::new(
            (scale.design.x * 0.90).clamp(360.0, 780.0),
            (scale.design.y * 0.86).clamp(300.0, 520.0),
        )
    };

    spawn_panel_frame(commands, MenuUi, panel, s);

    let title_px = if compact { 34.0 } else { 46.0 };
    let body_px = if compact { 14.0 } else { 18.0 };
    let body = if scale.class.is_handheld() {
        format!(
            "Collect yellow stars. Avoid red hazards.\n\
             Hold 1 finger to move  -  2nd finger dashes\n\n\
             Best score: {best}\n\n\
             Tap  -  choose mode"
        )
    } else if compact {
        format!(
            "Collect yellow stars. Avoid red hazards.\n\
             WASD / arrows move  -  SPACE dash\n\n\
             Best score: {best}\n\n\
             ENTER / tap  -  choose mode"
        )
    } else {
        format!(
            "Collect yellow stars  -  dodge red hazards\n\
             WASD / arrows move  -  SPACE / right-click dash\n\
             Hold mouse to point-to-move  -  power-ups\n\n\
             Best score: {best}\n\n\
             ENTER / SPACE / click  -  choose mode\n\
             ESC  -  quit (desktop)"
        )
    };

    commands.spawn((
        MenuUi,
        ScaledText {
            base_px: title_px,
            menu: true,
        },
        ScaledPos {
            base: Vec2::new(0.0, panel.y * 0.30),
            menu: true,
        },
        Text2d::new("RUSTY DASHER"),
        font(title_px, s),
        TextColor(Color::srgb(1.0, 0.88, 0.35)),
        Transform::from_xyz(0.0, panel.y * 0.30 * s, 20.0),
    ));
    commands.spawn((
        MenuUi,
        ScaledText {
            base_px: body_px,
            menu: true,
        },
        ScaledPos {
            base: Vec2::new(0.0, panel.y * 0.18),
            menu: true,
        },
        Text2d::new("by IntRUSTing Games"),
        font(body_px, s),
        TextColor(Color::srgb(0.55, 0.75, 1.0)),
        Transform::from_xyz(0.0, panel.y * 0.18 * s, 20.0),
    ));
    commands.spawn((
        MenuUi,
        ScaledText {
            base_px: body_px,
            menu: true,
        },
        ScaledPos {
            base: Vec2::new(0.0, -panel.y * 0.08),
            menu: true,
        },
        Text2d::new(body),
        font(body_px, s),
        TextColor(Color::srgb(0.88, 0.91, 0.97)),
        TextLayout::justify(Justify::Center),
        Transform::from_xyz(0.0, -panel.y * 0.08 * s, 20.0),
    ));
}

fn spawn_panel_frame<M: Component + Copy>(
    commands: &mut Commands,
    marker: M,
    panel: Vec2,
    s: f32,
) {
    // Full-view dim so the playfield doesn't show through on tall/narrow screens.
    commands.spawn((
        marker,
        Sprite::from_color(Color::srgba(0.02, 0.03, 0.06, 0.72), Vec2::new(4000.0, 4000.0)),
        Transform::from_xyz(0.0, 0.0, 12.0),
    ));
    let border = panel_border(panel);
    commands.spawn((
        marker,
        ScaledPanel { base: border },
        Sprite::from_color(Color::srgb(0.35, 0.5, 0.9), border * s),
        Transform::from_xyz(0.0, 0.0, 14.0),
    ));
    commands.spawn((
        marker,
        ScaledPanel { base: panel },
        Sprite::from_color(Color::srgba(0.09, 0.11, 0.18, 0.96), panel * s),
        Transform::from_xyz(0.0, 0.0, 15.0),
    ));
}

/// Rebuild title / mode-select when the aspect class changes (phone rotate, first web resize).
pub fn rebuild_menus_on_layout_change(
    scale: Res<UiScale>,
    state: Res<State<GameState>>,
    mut commands: Commands,
    save: Res<SaveData>,
    selected: Res<SelectedMode>,
    difficulty: Res<SelectedDifficulty>,
    menu_q: Query<Entity, With<MenuUi>>,
    mode_q: Query<Entity, With<ModeUi>>,
    mut last: Local<(f32, Vec2)>,
    mut frames: Local<u32>,
) {
    *frames = frames.saturating_add(1);
    let (prev_aspect, prev_design) = *last;
    let initialized = prev_aspect != 0.0 || prev_design != Vec2::ZERO;
    let changed = (prev_aspect - scale.aspect).abs() > 0.12
        || (prev_design - scale.design).length_squared() > 80.0;

    // First real scale sample after a few frames (window size settles on web).
    if !initialized {
        if *frames < 4 {
            return;
        }
        *last = (scale.aspect, scale.design);
        // Always rebuild once after settle — OnEnter often used Default 16:9.
    } else if !changed {
        return;
    } else {
        *last = (scale.aspect, scale.design);
    }

    match *state.get() {
        GameState::Menu => {
            for e in &menu_q {
                commands.entity(e).despawn();
            }
            spawn_menu_with(&mut commands, &save, &scale);
        }
        GameState::ModeSelect => {
            for e in &mode_q {
                commands.entity(e).despawn();
            }
            spawn_mode_select_with(&mut commands, &save, selected.0, difficulty.0, &scale);
        }
        _ => {}
    }
}

pub fn menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    touch: Res<TouchControls>,
    mut next: ResMut<NextState<GameState>>,
    exit: MessageWriter<AppExit>,
) {
    if confirm_just_pressed(&keyboard) || touch.confirm_just {
        next.set(GameState::ModeSelect);
    }
    if back_just_pressed(&keyboard) || touch.back_just {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut exit = exit;
            exit.write(AppExit::Success);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = exit;
        }
    }
}

pub fn spawn_mode_select(
    mut commands: Commands,
    save: Res<SaveData>,
    selected: Res<SelectedMode>,
    difficulty: Res<SelectedDifficulty>,
    scale: Res<UiScale>,
) {
    spawn_mode_select_with(&mut commands, &save, selected.0, difficulty.0, &scale);
}

fn spawn_mode_select_with(
    commands: &mut Commands,
    save: &SaveData,
    selected: GameMode,
    difficulty: Difficulty,
    scale: &UiScale,
) {
    let s = scale.panel;
    let panel = scale.design;
    spawn_panel_frame(commands, ModeUi, panel, s);

    let compact = scale.class.is_compact() || scale.aspect < 0.85 || scale.design.y < 400.0;
    let title_px = if compact { 26.0 } else { 34.0 };
    commands.spawn((
        ModeUi,
        ScaledText {
            base_px: title_px,
            menu: true,
        },
        ScaledPos {
            base: Vec2::new(0.0, panel.y * 0.38),
            menu: true,
        },
        Text2d::new("SELECT MODE"),
        font(title_px, s),
        TextColor(Color::srgb(0.95, 0.95, 1.0)),
        Transform::from_xyz(0.0, panel.y * 0.38 * s, 20.0),
    ));

    refresh_mode_list(commands, save, selected, difficulty, scale);

    let help = if scale.class.is_handheld() {
        "Tap top/bottom = mode    sides = difficulty\nTap center = start    two-finger = back"
    } else if compact {
        "Up/Down = mode    Left/Right = difficulty\nENTER / tap = start    ESC = back"
    } else {
        "Up/Down = mode     Left/Right = difficulty\nENTER / SPACE / click = start     ESC = back"
    };
    commands.spawn((
        ModeUi,
        ScaledText {
            base_px: 13.0,
            menu: true,
        },
        ScaledPos {
            base: Vec2::new(0.0, -panel.y * 0.38),
            menu: true,
        },
        Text2d::new(help),
        font(13.0, s),
        TextColor(Color::srgb(0.6, 0.68, 0.85)),
        TextLayout::justify(Justify::Center),
        Transform::from_xyz(0.0, -panel.y * 0.38 * s, 20.0),
    ));
}

#[derive(Component)]
pub struct ModeListText;

fn mode_list_body(save: &SaveData, selected: GameMode) -> String {
    let mut body = String::from("MODE\n");
    for mode in GameMode::ALL {
        let marker = if mode == selected { ">" } else { " " };
        let hs = save.high_scores.get(mode);
        // Name + score only — no "best", no blurbs.
        body.push_str(&format!("{marker} {:<10}  {:>4}\n", mode.label(), hs));
    }
    body.push_str("\nDIFFICULTY\n");
    body
}

const DIFF_ROW_Y: f32 = -55.0;

/// Horizontal difficulty slots scaled to the design panel (fits phone → 4K).
fn diff_slot_layout(scale: &UiScale) -> ([f32; 4], f32) {
    let half = (scale.design.x * 0.40).clamp(110.0, 210.0);
    let step = half / 1.5;
    let xs = [-1.5 * step, -0.5 * step, 0.5 * step, 1.5 * step];
    let bracket = (step * 0.55).clamp(28.0, 52.0);
    (xs, bracket)
}

fn refresh_mode_list(
    commands: &mut Commands,
    save: &SaveData,
    selected: GameMode,
    difficulty: Difficulty,
    scale: &UiScale,
) {
    let compact = scale.class.is_compact()
        || scale.aspect < 0.85
        || scale.design.y < 400.0
        || matches!(scale.class, ViewportClass::PhoneLandscape);
    let s = scale.panel;
    let px = if compact { 15.0 } else { 17.0 };
    let body = mode_list_body(save, selected);
    let mode_y = if compact { 42.0 } else { 50.0 };

    commands.spawn((
        ModeUi,
        ModeListText,
        ScaledText {
            base_px: px,
            menu: true,
        },
        ScaledPos {
            base: Vec2::new(0.0, mode_y),
            menu: true,
        },
        Text2d::new(body),
        font(px, s),
        TextColor(Color::srgb(0.9, 0.93, 1.0)),
        TextLayout::justify(Justify::Center),
        Anchor::CENTER,
        Transform::from_xyz(0.0, mode_y * s, 20.0),
    ));

    // Side-by-side difficulty: fixed design slots so Left/Right never reflows labels.
    let (slots, bracket_ox) = diff_slot_layout(scale);
    let diff_px = if compact { 13.0 } else { 16.0 };
    for (i, d) in Difficulty::ALL.iter().enumerate() {
        let x = slots[i];
        commands.spawn((
            ModeUi,
            ModeListText,
            ScaledText {
                base_px: diff_px,
                menu: true,
            },
            ScaledPos {
                base: Vec2::new(x, DIFF_ROW_Y),
                menu: true,
            },
            Text2d::new(d.label()),
            font(diff_px, s),
            TextColor(if *d == difficulty {
                Color::srgb(1.0, 0.92, 0.45)
            } else {
                Color::srgb(0.75, 0.8, 0.92)
            }),
            TextLayout::justify(Justify::Center),
            Anchor::CENTER,
            Transform::from_xyz(x * s, DIFF_ROW_Y * s, 20.0),
        ));
        if *d == difficulty {
            for (glyph, ox) in [("[", -bracket_ox), ("]", bracket_ox)] {
                commands.spawn((
                    ModeUi,
                    ModeListText,
                    ScaledText {
                        base_px: diff_px,
                        menu: true,
                    },
                    ScaledPos {
                        base: Vec2::new(x + ox, DIFF_ROW_Y),
                        menu: true,
                    },
                    Text2d::new(glyph),
                    font(diff_px, s),
                    TextColor(Color::srgb(1.0, 0.92, 0.45)),
                    Anchor::CENTER,
                    Transform::from_xyz((x + ox) * s, DIFF_ROW_Y * s, 20.0),
                ));
            }
        }
    }

    let stats_y = DIFF_ROW_Y - if compact { 30.0 } else { 36.0 };
    let stats = format!(
        "score x{:.1}   speed x{:.1}",
        difficulty.score_mult(),
        difficulty.speed_mult()
    );
    commands.spawn((
        ModeUi,
        ModeListText,
        ScaledText {
            base_px: if compact { 12.0 } else { 15.0 },
            menu: true,
        },
        ScaledPos {
            base: Vec2::new(0.0, stats_y),
            menu: true,
        },
        Text2d::new(stats),
        font(if compact { 12.0 } else { 15.0 }, s),
        TextColor(Color::srgb(0.7, 0.78, 0.9)),
        TextLayout::justify(Justify::Center),
        Anchor::CENTER,
        Transform::from_xyz(0.0, stats_y * s, 20.0),
    ));
}

pub fn mode_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    touch: Res<TouchControls>,
    mut selected: ResMut<SelectedMode>,
    mut difficulty: ResMut<SelectedDifficulty>,
    mut next: ResMut<NextState<GameState>>,
    mut commands: Commands,
    save: Res<SaveData>,
    scale: Res<UiScale>,
    list: Query<Entity, With<ModeListText>>,
) {
    let mut changed = false;
    if menu_up_just_pressed(&keyboard) || touch.menu_up_just {
        let idx = GameMode::ALL
            .iter()
            .position(|m| *m == selected.0)
            .unwrap_or(0);
        let new_idx = if idx == 0 {
            GameMode::ALL.len() - 1
        } else {
            idx - 1
        };
        selected.0 = GameMode::ALL[new_idx];
        changed = true;
    }
    if menu_down_just_pressed(&keyboard) || touch.menu_down_just {
        let idx = GameMode::ALL
            .iter()
            .position(|m| *m == selected.0)
            .unwrap_or(0);
        selected.0 = GameMode::ALL[(idx + 1) % GameMode::ALL.len()];
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::KeyA)
        || touch.menu_diff_left
    {
        let idx = Difficulty::ALL
            .iter()
            .position(|d| *d == difficulty.0)
            .unwrap_or(1);
        let new_idx = if idx == 0 {
            Difficulty::ALL.len() - 1
        } else {
            idx - 1
        };
        difficulty.0 = Difficulty::ALL[new_idx];
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight)
        || keyboard.just_pressed(KeyCode::KeyD)
        || touch.menu_diff_right
    {
        let idx = Difficulty::ALL
            .iter()
            .position(|d| *d == difficulty.0)
            .unwrap_or(1);
        difficulty.0 = Difficulty::ALL[(idx + 1) % Difficulty::ALL.len()];
        changed = true;
    }
    if changed {
        for e in &list {
            commands.entity(e).despawn();
        }
        refresh_mode_list(&mut commands, &save, selected.0, difficulty.0, &scale);
    }
    if confirm_just_pressed(&keyboard) || touch.confirm_just {
        next.set(GameState::Playing);
    }
    if back_just_pressed(&keyboard) || touch.back_just {
        next.set(GameState::Menu);
    }
}

pub fn spawn_game_over(
    mut commands: Commands,
    stats: Res<GameStats>,
    save: Res<SaveData>,
    scale: Res<UiScale>,
) {
    let hs = save.high_scores.get(stats.mode);
    let headline = if stats.is_new_record {
        "NEW HIGH SCORE!"
    } else {
        "GAME OVER"
    };
    let color = if stats.is_new_record {
        Color::srgb(1.0, 0.85, 0.25)
    } else {
        Color::srgb(1.0, 0.42, 0.48)
    };
    let s = scale.panel;
    let panel = Vec2::new(
        (scale.design.x * 0.88).clamp(340.0, 720.0),
        (scale.design.y * 0.78).clamp(260.0, 420.0),
    );
    spawn_panel_frame(&mut commands, GameOverUi, panel, s);

    commands.spawn((
        GameOverUi,
        ScaledText {
            base_px: 44.0,
            menu: true,
        },
        ScaledPos {
            base: Vec2::new(0.0, panel.y * 0.22),
            menu: true,
        },
        Text2d::new(headline),
        font(44.0, s),
        TextColor(color),
        Transform::from_xyz(0.0, panel.y * 0.22 * s, 30.0),
    ));
    commands.spawn((
        GameOverUi,
        ScaledText {
            base_px: 20.0,
            menu: true,
        },
        ScaledPos {
            base: Vec2::new(0.0, -panel.y * 0.06),
            menu: true,
        },
        Text2d::new(format!(
            "{}  -  {}  -  Score {}\n\
             Level {}  -  Best combo x{}  -  Stars {}\n\
             High score {}\n\n\
             ENTER / tap  -  play again     ESC  -  menu",
            stats.mode.label(),
            stats.chosen_difficulty.label(),
            stats.score,
            stats.level,
            stats.best_combo.max(1),
            stats.stars_collected,
            hs
        )),
        font(20.0, s),
        TextColor(Color::srgb(0.9, 0.92, 0.98)),
        TextLayout::justify(Justify::Center),
        Transform::from_xyz(0.0, -panel.y * 0.06 * s, 30.0),
    ));
}

pub fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    touch: Res<TouchControls>,
    mut next: ResMut<NextState<GameState>>,
) {
    if confirm_just_pressed(&keyboard) || touch.confirm_just {
        next.set(GameState::Playing);
    }
    if back_just_pressed(&keyboard) || touch.back_just {
        next.set(GameState::Menu);
    }
}

pub fn playing_escape(
    keyboard: Res<ButtonInput<KeyCode>>,
    touch: Res<TouchControls>,
    mut next: ResMut<NextState<GameState>>,
) {
    if back_just_pressed(&keyboard) || touch.back_just {
        next.set(GameState::Menu);
    }
}

pub fn spawn_hud(
    commands: &mut Commands,
    stats: &GameStats,
    bounds: &PlayBounds,
    scale: f32,
    assets: &GameAssets,
) {
    let top = bounds.hud_top_y;
    let bot = bounds.hud_bottom_y;
    let left = -bounds.view_half.x + bounds.margin * 0.55;
    let right = bounds.view_half.x - bounds.margin * 0.55;

    commands.spawn((
        PlayEntity,
        HudScore,
        ScaledText {
            base_px: 26.0,
            menu: false,
        },
        Text2d::new(format!("Score  {}", stats.score)),
        font(26.0, scale),
        TextColor(Color::srgb(0.95, 0.96, 1.0)),
        Transform::from_xyz(left + 80.0, top, 20.0),
    ));
    // Real heart icons (not "1 heart" text — default font has no ♥ glyph).
    let heart_size = 28.0 * scale.clamp(0.75, 1.8);
    let spacing = heart_size + 6.0;
    let max_hearts = 3u32;
    for i in 0..max_hearts {
        let filled = stats.mode == GameMode::Zen || i < stats.lives;
        let x = right - 14.0 - (max_hearts - 1 - i) as f32 * spacing;
        commands.spawn((
            PlayEntity,
            HudLives,
            HudHeart { index: i },
            Sprite {
                image: assets.tex_heart.clone(),
                custom_size: Some(Vec2::splat(heart_size)),
                color: if filled {
                    Color::WHITE
                } else {
                    Color::srgba(0.35, 0.35, 0.4, 0.35)
                },
                ..default()
            },
            Transform::from_xyz(x, top, 20.0),
        ));
    }
    commands.spawn((
        PlayEntity,
        HudCombo,
        ScaledText {
            base_px: 26.0,
            menu: false,
        },
        Text2d::new(""),
        font(26.0, scale),
        TextColor(Color::srgb(1.0, 0.85, 0.3)),
        Transform::from_xyz(0.0, top, 20.0),
    ));
    commands.spawn((
        PlayEntity,
        HudLevel,
        ScaledText {
            base_px: 17.0,
            menu: false,
        },
        Text2d::new(format!(
            "{}  |  {}",
            stats.mode.label(),
            stats.chosen_difficulty.label()
        )),
        font(17.0, scale),
        TextColor(Color::srgb(0.65, 0.75, 0.95)),
        Transform::from_xyz(0.0, top - (bounds.margin * 0.28).clamp(16.0, 30.0), 20.0),
    ));
    commands.spawn((
        PlayEntity,
        HudStatus,
        ScaledText {
            base_px: 15.0,
            menu: false,
        },
        Text2d::new("WASD move  -  SPACE / 2nd finger dash  -  hold to move"),
        font(15.0, scale),
        TextColor(Color::srgb(0.55, 0.62, 0.78)),
        Transform::from_xyz(0.0, bot, 20.0),
    ));
}

pub fn spawn_level_banner(commands: &mut Commands, level: u32, scale: f32) {
    commands.spawn((
        PlayEntity,
        LevelBanner { life: 1.8 },
        ScaledText {
            base_px: 48.0,
            menu: false,
        },
        Text2d::new(format!("LEVEL {level}")),
        font(48.0, scale),
        TextColor(Color::srgb(1.0, 0.9, 0.4)),
        Transform::from_xyz(0.0, 40.0, 40.0),
    ));
}

pub fn tick_level_banners(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut LevelBanner, &mut TextColor, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (e, mut ban, mut color, mut tf) in &mut q {
        ban.life -= dt;
        let t = (ban.life / 1.8).clamp(0.0, 1.0);
        let c = color.0.to_srgba();
        color.0 = Color::srgba(c.red, c.green, c.blue, t);
        tf.translation.y += 20.0 * dt;
        if ban.life <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

pub fn update_hud(
    stats: Res<GameStats>,
    player: Query<&Player>,
    mut score_q: Query<
        &mut Text2d,
        (
            With<HudScore>,
            Without<HudLives>,
            Without<HudCombo>,
            Without<HudStatus>,
            Without<HudLevel>,
        ),
    >,
    mut hearts_q: Query<(&HudHeart, &mut Sprite), With<HudLives>>,
    mut combo_q: Query<
        &mut Text2d,
        (
            With<HudCombo>,
            Without<HudScore>,
            Without<HudLives>,
            Without<HudStatus>,
            Without<HudLevel>,
        ),
    >,
    mut level_q: Query<
        &mut Text2d,
        (
            With<HudLevel>,
            Without<HudScore>,
            Without<HudLives>,
            Without<HudCombo>,
            Without<HudStatus>,
        ),
    >,
    mut status_q: Query<
        &mut Text2d,
        (
            With<HudStatus>,
            Without<HudScore>,
            Without<HudLives>,
            Without<HudCombo>,
            Without<HudLevel>,
        ),
    >,
) {
    if let Ok(mut text) = score_q.single_mut() {
        **text = format!("Score  {}", stats.score);
    }
    for (heart, mut sprite) in &mut hearts_q {
        let filled = stats.mode == GameMode::Zen || heart.index < stats.lives;
        sprite.color = if filled {
            Color::WHITE
        } else {
            Color::srgba(0.35, 0.35, 0.4, 0.35)
        };
    }
    if let Ok(mut text) = combo_q.single_mut() {
        **text = if stats.combo >= 2 {
            format!("COMBO x{}", 1 + stats.combo / 3)
        } else {
            String::new()
        };
    }
    if let Ok(mut text) = level_q.single_mut() {
        **text = if stats.mode == GameMode::Timed {
            format!(
                "{}  |  {}  |  {:.0}s left",
                stats.mode.label(),
                stats.chosen_difficulty.label(),
                stats.time_left.ceil()
            )
        } else if matches!(stats.mode, GameMode::Classic | GameMode::Survival) {
            format!(
                "{}  |  {}  |  Lv {}  |  next {}",
                stats.mode.label(),
                stats.chosen_difficulty.label(),
                stats.level,
                stats.level_target
            )
        } else {
            format!(
                "{}  |  {}  |  chill",
                stats.mode.label(),
                stats.chosen_difficulty.label()
            )
        };
    }
    if let Ok(mut text) = status_q.single_mut() {
        let Ok(p) = player.single() else {
            return;
        };
        let mut bits = Vec::new();
        if p.dash_cooldown > 0.0 {
            bits.push(format!("Dash {:.1}s", p.dash_cooldown));
        } else {
            bits.push("Dash READY".into());
        }
        if p.magnet > 0.0 {
            bits.push(format!("Magnet {:.0}s", p.magnet));
        }
        if p.shield > 0.0 {
            bits.push(format!("Shield {:.0}s", p.shield));
        }
        if p.speed_boost > 0.0 {
            bits.push(format!("Speed {:.0}s", p.speed_boost));
        }
        **text = bits.join("  -  ");
    }
}
