//! Touch / pointer controls for browser + handheld.
//!
//! Playing (handheld — Game Boy / PSP chrome):
//!   - **Virtual joystick** in the control deck → movement
//!   - **Dash button** on the opposite side → dash
//!   - Sides can be swapped via main-menu toggle (`SaveData.swap_touch_controls`)
//! Playing (desktop mouse):
//!   - **Left drag** → point-to-move toward cursor
//!   - **Right-click** → dash
//! Mode select (handheld):
//!   - Mode list band: top half = previous mode, bottom half = next mode
//!   - Difficulty row: left half = easier, right half = harder
//!   - **START** button band → confirm
//!   - Two-finger tap → back
//! Menus (main / game over):
//!   - Tap → confirm (except swap-controls band on main menu)

use crate::components::{MainCamera, Player};
use crate::save::SaveData;
use crate::state::GameState;
use crate::ui_scale::UiScale;
use crate::viewport::PlayBounds;
use crate::web_pointer;
use bevy::prelude::*;

/// World-space distance at which point-to-move is considered "arrived" (desktop mouse).
const ARRIVE_RADIUS: f32 = 14.0;
/// Distance at which point-to-move uses full move speed (smooth ramp below this).
const FULL_SPEED_DIST: f32 = 90.0;

/// Screen-space layout for the virtual stick + dash button (logical window coords).
/// Y grows downward (Bevy window / browser convention).
#[derive(Resource, Debug, Clone, Copy)]
pub struct TouchChromeLayout {
    pub active: bool,
    pub portrait: bool,
    /// Stick base center (window logical px).
    pub stick_center: Vec2,
    /// Max stick travel radius (window logical px).
    pub stick_radius: f32,
    /// Hit radius around stick (slightly larger than visual).
    pub stick_hit_radius: f32,
    /// Dash button center.
    pub dash_center: Vec2,
    pub dash_radius: f32,
    pub dash_hit_radius: f32,
    /// Deck / grip rects for visual shell (min..max in window space).
    pub deck_min: Vec2,
    pub deck_max: Vec2,
    /// True when stick is on the right (user preference).
    pub swapped: bool,
}

impl Default for TouchChromeLayout {
    fn default() -> Self {
        Self {
            active: false,
            portrait: true,
            stick_center: Vec2::ZERO,
            stick_radius: 48.0,
            stick_hit_radius: 72.0,
            dash_center: Vec2::ZERO,
            dash_radius: 36.0,
            dash_hit_radius: 52.0,
            deck_min: Vec2::ZERO,
            deck_max: Vec2::ZERO,
            swapped: false,
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct TouchControls {
    /// Normalized movement intent (length 0..=1). Used by dash facing + keyboard merge.
    pub move_dir: Vec2,
    /// 0..=1 speed scale (stick magnitude or point-to-move distance ramp).
    pub move_strength: f32,
    /// World-space point under the active move pointer (desktop mouse only).
    pub move_target: Option<Vec2>,
    pub dash: bool,
    pub dash_just: bool,
    pub confirm_just: bool,
    pub back_just: bool,
    pub menu_up_just: bool,
    pub menu_down_just: bool,
    pub menu_diff_left: bool,
    pub menu_diff_right: bool,
    /// Main menu: toggle stick/DASH side preference.
    pub swap_controls_just: bool,
    /// Which touch/mouse is owning the stick (for multi-touch).
    stick_pointer: Option<u64>,
    /// Live stick knob offset in window space (for rendering).
    pub stick_knob_offset: Vec2,
    /// Dash button currently held.
    pub dash_held: bool,
}

/// Recompute screen-space stick/dash hit targets.
///
/// When [`PlayBounds`] chrome insets are live, map grip/deck fractions from
/// world → window so hits match bounds-driven visuals (critical on high-DPR
/// landscape where raw 20%-of-width clamps drifted from chrome_insets).
pub fn sync_touch_chrome_layout(
    windows: Query<&Window>,
    ui: Res<UiScale>,
    state: Res<State<GameState>>,
    save: Res<SaveData>,
    bounds: Res<PlayBounds>,
    mut layout: ResMut<TouchChromeLayout>,
) {
    let Ok(window) = windows.single() else {
        layout.active = false;
        return;
    };
    let w = window.width().max(1.0);
    let h = window.height().max(1.0);
    let handheld = ui.class.is_handheld();
    let playing = *state.get() == GameState::Playing;
    let active = handheld && playing;
    layout.active = active;
    layout.swapped = save.swap_touch_controls;
    if !active {
        return;
    }

    let portrait = ui.class.is_portrait() || h >= w;
    layout.portrait = portrait;

    let (mut stick_c, mut dash_c, stick_r, dash_r, deck_min, deck_max) =
        if bounds.chrome && bounds.view_half.x > 1.0 && bounds.view_half.y > 1.0 {
            // Mirror PlayBounds chrome strips into window space (Y down).
            let view_w = bounds.view_half.x * 2.0;
            let to_win = |world: Vec2| -> Vec2 {
                let nx = (world.x / bounds.view_half.x) * 0.5 + 0.5;
                let ny = 0.5 - (world.y / bounds.view_half.y) * 0.5;
                Vec2::new(nx * w, ny * h)
            };
            let world_to_win_r = |r: f32| (r / view_w) * w;

            if portrait {
                let deck_top_w = bounds.bottom();
                let deck_bot_w = -bounds.view_half.y;
                let deck_cy_w = (deck_top_w + deck_bot_w) * 0.5;
                let deck_h_w = (deck_top_w - deck_bot_w).max(24.0);
                let stick_r_w = (deck_h_w * 0.34).clamp(28.0, 72.0);
                let dash_r_w = (deck_h_w * 0.28).clamp(22.0, 56.0);
                let left_w = Vec2::new(-bounds.view_half.x + view_w * 0.28, deck_cy_w);
                let right_w = Vec2::new(-bounds.view_half.x + view_w * 0.75, deck_cy_w);
                let deck_min = Vec2::new(0.0, to_win(Vec2::new(0.0, deck_top_w)).y);
                let deck_max = Vec2::new(w, h);
                (
                    to_win(left_w),
                    to_win(right_w),
                    world_to_win_r(stick_r_w).clamp(36.0, 72.0),
                    world_to_win_r(dash_r_w).clamp(28.0, 56.0),
                    deck_min,
                    deck_max,
                )
            } else {
                let left_l = -bounds.view_half.x;
                let left_r = bounds.left() - 2.0;
                let left_w = (left_r - left_l).max(8.0);
                let left_cx = left_l + left_w * 0.5;
                let right_l = bounds.right() + 2.0;
                let right_r = bounds.view_half.x;
                let right_w = (right_r - right_l).max(8.0);
                let right_cx = right_l + right_w * 0.5;
                let cy = -bounds.view_half.y * 0.04;
                let stick_r_w = (left_w.min(right_w) * 0.38).clamp(28.0, 90.0);
                let dash_r_w = (left_w.min(right_w) * 0.32).clamp(22.0, 72.0);
                (
                    to_win(Vec2::new(left_cx, cy)),
                    to_win(Vec2::new(right_cx, cy)),
                    world_to_win_r(stick_r_w).clamp(32.0, 70.0),
                    world_to_win_r(dash_r_w).clamp(26.0, 56.0),
                    Vec2::new(0.0, 0.0),
                    Vec2::new(w, h),
                )
            }
        } else if portrait {
            // Fallback before chrome bounds settle: Game Boy bottom deck.
            let deck_top = h * 0.66;
            let deck_min = Vec2::new(0.0, deck_top);
            let deck_max = Vec2::new(w, h);
            let deck_cy = (deck_top + h) * 0.5;
            let stick_r = (w.min(h) * 0.11).clamp(40.0, 64.0);
            let dash_r = (w.min(h) * 0.08).clamp(30.0, 48.0);
            let stick_c = Vec2::new(w * 0.28, deck_cy);
            let dash_c = Vec2::new(w * 0.75, deck_cy);
            (stick_c, dash_c, stick_r, dash_r, deck_min, deck_max)
        } else {
            // Fallback: PSP side grips as a fraction of window (matches chrome_insets ~20%).
            let grip_w = w * 0.20;
            let deck_min = Vec2::new(0.0, 0.0);
            let deck_max = Vec2::new(w, h);
            let stick_r = (h * 0.16).clamp(36.0, 58.0);
            let dash_r = (h * 0.13).clamp(28.0, 44.0);
            let stick_c = Vec2::new(grip_w * 0.5, h * 0.52);
            let dash_c = Vec2::new(w - grip_w * 0.5, h * 0.52);
            (stick_c, dash_c, stick_r, dash_r, deck_min, deck_max)
        };

    if save.swap_touch_controls {
        std::mem::swap(&mut stick_c, &mut dash_c);
    }

    layout.deck_min = deck_min;
    layout.deck_max = deck_max;
    layout.stick_center = stick_c;
    layout.stick_radius = stick_r;
    layout.stick_hit_radius = stick_r * 1.55;
    layout.dash_center = dash_c;
    layout.dash_radius = dash_r;
    layout.dash_hit_radius = dash_r * 1.45;
}

pub fn update_touch_controls(
    touches: Res<Touches>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    player_q: Query<&Transform, With<Player>>,
    state: Res<State<GameState>>,
    layout: Res<TouchChromeLayout>,
    ui: Res<UiScale>,
    mut controls: ResMut<TouchControls>,
) {
    controls.dash_just = false;
    controls.confirm_just = false;
    controls.back_just = false;
    controls.menu_up_just = false;
    controls.menu_down_just = false;
    controls.menu_diff_left = false;
    controls.menu_diff_right = false;
    controls.swap_controls_just = false;
    controls.move_dir = Vec2::ZERO;
    controls.move_strength = 0.0;
    controls.move_target = None;
    controls.dash = false;
    controls.dash_held = false;

    let Ok(window) = windows.single() else {
        return;
    };
    let size = window.size();
    if size.x <= 1.0 || size.y <= 1.0 {
        return;
    }

    let mouse_just = mouse.just_pressed(MouseButton::Left);
    let mouse_down = mouse.pressed(MouseButton::Left);
    let mouse_right_just = mouse.just_pressed(MouseButton::Right);

    match *state.get() {
        GameState::Playing => {
            if layout.active {
                update_playing_chrome(
                    &touches,
                    window,
                    &layout,
                    &mut controls,
                    mouse_just,
                    mouse_down,
                    mouse_right_just,
                );
            } else {
                update_playing_desktop(
                    &touches,
                    window,
                    &camera_q,
                    &player_q,
                    &mut controls,
                    mouse_down,
                    mouse_right_just,
                );
            }
        }
        GameState::Menu | GameState::GameOver | GameState::ModeSelect => {
            controls.stick_pointer = None;
            controls.stick_knob_offset = Vec2::ZERO;
            let tap = touches
                .iter_just_pressed()
                .next()
                .map(|t| t.position())
                .or_else(|| {
                    if mouse_just {
                        window.cursor_position()
                    } else {
                        None
                    }
                })
                .map(|p| web_pointer::remap_to_window(p, window));

            let Some(pos) = tap else {
                if touches.iter().count() >= 2 {
                    controls.back_just = true;
                }
                return;
            };

            let y_ratio = (pos.y / size.y).clamp(0.0, 1.0);
            let x_ratio = (pos.x / size.x).clamp(0.0, 1.0);
            let handheld = ui.class.is_handheld();

            match *state.get() {
                GameState::ModeSelect if handheld => {
                    // Ordered hit bands matching the on-screen layout (Y down).
                    // START first so it never fights mode/difficulty.
                    if y_ratio >= 0.58 && y_ratio <= 0.78 && x_ratio >= 0.18 && x_ratio <= 0.82 {
                        controls.confirm_just = true;
                    } else if y_ratio >= 0.46 && y_ratio < 0.58 {
                        // Difficulty row: whole left half = easier, right = harder
                        if x_ratio < 0.50 {
                            controls.menu_diff_left = true;
                        } else {
                            controls.menu_diff_right = true;
                        }
                    } else if y_ratio >= 0.20 && y_ratio < 0.46 {
                        // Mode list: upper half previous, lower half next
                        let mid = 0.33;
                        if y_ratio < mid {
                            controls.menu_up_just = true;
                        } else {
                            controls.menu_down_just = true;
                        }
                    } else if y_ratio > 0.82 || (x_ratio < 0.08 && y_ratio > 0.5) {
                        // Bottom strip / far-left: treat as back on handheld
                        controls.back_just = true;
                    }
                    // Two-finger still works via the empty-tap path above
                }
                GameState::ModeSelect => {
                    // Desktop/tablet-wide: keep side strips + thirds
                    if x_ratio < 0.20 {
                        controls.menu_diff_left = true;
                    } else if x_ratio > 0.80 {
                        controls.menu_diff_right = true;
                    } else if y_ratio < 0.28 {
                        controls.menu_up_just = true;
                    } else if y_ratio > 0.72 {
                        controls.menu_down_just = true;
                    } else {
                        controls.confirm_just = true;
                    }
                }
                GameState::Menu if handheld => {
                    // Bottom band: swap stick/DASH preference
                    if y_ratio >= 0.72 && y_ratio <= 0.92 && x_ratio >= 0.12 && x_ratio <= 0.88 {
                        controls.swap_controls_just = true;
                    } else if x_ratio < 0.08 {
                        controls.back_just = true;
                    } else {
                        controls.confirm_just = true;
                    }
                }
                GameState::Menu | GameState::GameOver => {
                    if x_ratio < 0.08 && matches!(*state.get(), GameState::GameOver) {
                        controls.back_just = true;
                    } else {
                        controls.confirm_just = true;
                    }
                }
                _ => {}
            }
        }
    }
}

fn update_playing_chrome(
    touches: &Touches,
    window: &Window,
    layout: &TouchChromeLayout,
    controls: &mut TouchControls,
    mouse_just: bool,
    mouse_down: bool,
    mouse_right_just: bool,
) {
    let mut stick_pos: Option<Vec2> = None;
    let mut dash_just = false;
    let mut dash_held = false;

    let active: Vec<_> = touches.iter().collect();
    let just: Vec<_> = touches.iter_just_pressed().collect();

    if let Some(id) = controls.stick_pointer {
        if !active.iter().any(|t| t.id() as u64 == id) {
            controls.stick_pointer = None;
            controls.stick_knob_offset = Vec2::ZERO;
        }
    }

    for t in &just {
        let p = web_pointer::remap_to_window(t.position(), window);
        let id = t.id() as u64;
        if in_circle(p, layout.dash_center, layout.dash_hit_radius) {
            dash_just = true;
            dash_held = true;
        } else if in_circle(p, layout.stick_center, layout.stick_hit_radius)
            || in_deck_stick_half(p, layout)
        {
            controls.stick_pointer = Some(id);
            stick_pos = Some(p);
        }
    }

    for t in &active {
        let p = web_pointer::remap_to_window(t.position(), window);
        let id = t.id() as u64;
        if Some(id) == controls.stick_pointer {
            stick_pos = Some(p);
        } else if in_circle(p, layout.dash_center, layout.dash_hit_radius) {
            dash_held = true;
        } else if controls.stick_pointer.is_none()
            && (in_circle(p, layout.stick_center, layout.stick_hit_radius)
                || in_deck_stick_half(p, layout))
        {
            controls.stick_pointer = Some(id);
            stick_pos = Some(p);
        }
    }

    if mouse_down {
        if let Some(raw) = window.cursor_position() {
            let p = web_pointer::remap_to_window(raw, window);
            if controls.stick_pointer.is_none() || controls.stick_pointer == Some(u64::MAX) {
                if mouse_just
                    && (in_circle(p, layout.stick_center, layout.stick_hit_radius)
                        || in_deck_stick_half(p, layout))
                {
                    controls.stick_pointer = Some(u64::MAX);
                }
                if controls.stick_pointer == Some(u64::MAX) {
                    stick_pos = Some(p);
                }
            }
            if in_circle(p, layout.dash_center, layout.dash_hit_radius) {
                dash_held = true;
                if mouse_just {
                    dash_just = true;
                }
            }
        }
    } else if controls.stick_pointer == Some(u64::MAX) {
        controls.stick_pointer = None;
        controls.stick_knob_offset = Vec2::ZERO;
    }

    if mouse_right_just {
        dash_just = true;
    }

    if let Some(p) = stick_pos {
        let delta = p - layout.stick_center;
        // Window Y is down; world / movement Y is up → flip Y.
        let move_delta = Vec2::new(delta.x, -delta.y);
        let len = move_delta.length();
        let max_r = layout.stick_radius;
        if len > 1.0 {
            let strength = (len / max_r).clamp(0.0, 1.0);
            let strength = if strength < 0.12 {
                0.0
            } else {
                ((strength - 0.12) / 0.88).clamp(0.0, 1.0)
            };
            if strength > 0.0 {
                controls.move_dir = move_delta / len;
                controls.move_strength = strength.max(0.2);
            }
            let visual = if len > max_r {
                delta * (max_r / len)
            } else {
                delta
            };
            controls.stick_knob_offset = visual;
        } else {
            controls.stick_knob_offset = Vec2::ZERO;
        }
    } else {
        controls.stick_knob_offset = Vec2::ZERO;
    }

    controls.dash_just = dash_just;
    controls.dash = dash_held || dash_just;
    controls.dash_held = dash_held;
}

fn in_deck_stick_half(p: Vec2, layout: &TouchChromeLayout) -> bool {
    // Grab zone around the stick side of the deck (respects swap).
    if layout.portrait {
        let mid_x = (layout.deck_min.x + layout.deck_max.x) * 0.5;
        let on_stick_side = if layout.swapped {
            p.x >= mid_x
        } else {
            p.x < mid_x
        };
        p.y >= layout.deck_min.y && p.y <= layout.deck_max.y && on_stick_side
    } else {
        p.x < layout.stick_center.x + layout.stick_hit_radius * 1.2
            && p.x > layout.stick_center.x - layout.stick_hit_radius * 1.6
            && (p.y - layout.stick_center.y).abs() < layout.stick_hit_radius * 1.8
    }
}

fn in_circle(p: Vec2, c: Vec2, r: f32) -> bool {
    p.distance_squared(c) <= r * r
}

fn update_playing_desktop(
    touches: &Touches,
    window: &Window,
    camera_q: &Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    player_q: &Query<&Transform, With<Player>>,
    controls: &mut TouchControls,
    mouse_down: bool,
    mouse_right_just: bool,
) {
    controls.stick_pointer = None;
    controls.stick_knob_offset = Vec2::ZERO;

    let active: Vec<_> = touches.iter().collect();
    let just_pressed: Vec<_> = touches.iter_just_pressed().collect();
    let touch_count = active.len();
    let held_prior = touch_count.saturating_sub(just_pressed.len());

    if held_prior >= 1 && !just_pressed.is_empty() {
        controls.dash_just = true;
        controls.dash = true;
    } else if touch_count >= 2 {
        controls.dash = true;
    }

    if mouse_right_just {
        controls.dash_just = true;
        controls.dash = true;
    }

    let move_screen = pick_move_pointer(&active, &just_pressed, touch_count)
        .or_else(|| {
            if mouse_down {
                window.cursor_position()
            } else {
                None
            }
        })
        .map(|p| web_pointer::remap_to_window(p, window));

    if let Some(screen) = move_screen {
        if let Ok((camera, cam_tf)) = camera_q.single() {
            if let Ok(world) = camera.viewport_to_world_2d(cam_tf, screen) {
                controls.move_target = Some(world);
                if let Ok(player_tf) = player_q.single() {
                    let delta = world - player_tf.translation.truncate();
                    let dist = delta.length();
                    if dist > ARRIVE_RADIUS {
                        controls.move_dir = delta / dist;
                        controls.move_strength =
                            ((dist - ARRIVE_RADIUS) / (FULL_SPEED_DIST - ARRIVE_RADIUS))
                                .clamp(0.15, 1.0);
                    }
                }
            }
        }
    }
}

fn pick_move_pointer(
    active: &[&bevy::input::touch::Touch],
    just_pressed: &[&bevy::input::touch::Touch],
    touch_count: usize,
) -> Option<Vec2> {
    if active.is_empty() {
        return None;
    }

    if touch_count >= 2 && !just_pressed.is_empty() {
        let just_ids: Vec<_> = just_pressed.iter().map(|t| t.id()).collect();
        if let Some(held) = active.iter().find(|t| !just_ids.contains(&t.id())) {
            return Some(held.position());
        }
    }

    Some(active[0].position())
}

/// Convert window logical position → world using the live camera (matches playfield).
#[allow(dead_code)] // kept for input/debug paths; chrome visuals use PlayBounds
pub fn window_to_world(
    window_pos: Vec2,
    camera: &Camera,
    cam_tf: &GlobalTransform,
) -> Option<Vec2> {
    camera.viewport_to_world_2d(cam_tf, window_pos).ok()
}

/// Fallback when camera is unavailable (should be rare).
#[allow(dead_code)]
pub fn window_to_world_approx(
    window_pos: Vec2,
    window: &Window,
    bounds: &PlayBounds,
) -> Vec2 {
    let w = window.width().max(1.0);
    let h = window.height().max(1.0);
    let nx = (window_pos.x / w) * 2.0 - 1.0;
    let ny = 1.0 - (window_pos.y / h) * 2.0;
    Vec2::new(nx * bounds.view_half.x, ny * bounds.view_half.y)
}
