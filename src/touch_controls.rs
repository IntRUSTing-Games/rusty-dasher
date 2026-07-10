//! Touch / pointer controls for browser + handheld.
//!
//! Playing:
//!   - **One finger** (or left mouse drag) → **point-to-move** toward that world point
//!   - **Second finger** just pressed while the first is still down → **dash**
//!   - Keyboard Space / right-click also dash (desktop)
//! Menus:
//!   - Tap center → confirm
//!   - Tap top / bottom thirds → previous / next mode
//!   - Side strips → difficulty
//!   - Two-finger tap or far-left edge → back (menus only)

use crate::components::{MainCamera, Player};
use crate::state::GameState;
use bevy::prelude::*;

/// World-space distance at which point-to-move is considered "arrived".
const ARRIVE_RADIUS: f32 = 14.0;
/// Distance at which point-to-move uses full move speed (smooth ramp below this).
const FULL_SPEED_DIST: f32 = 90.0;

#[derive(Resource, Default, Debug)]
pub struct TouchControls {
    /// Normalized movement intent (length 0..=1). Used by dash facing + keyboard merge.
    pub move_dir: Vec2,
    /// 0..=1 speed scale for touch point-to-move (distance-based).
    pub move_strength: f32,
    /// World-space point under the active move pointer (None = no touch move).
    pub move_target: Option<Vec2>,
    pub dash: bool,
    pub dash_just: bool,
    pub confirm_just: bool,
    pub back_just: bool,
    pub menu_up_just: bool,
    pub menu_down_just: bool,
    pub menu_diff_left: bool,
    pub menu_diff_right: bool,
}

pub fn update_touch_controls(
    touches: Res<Touches>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    player_q: Query<&Transform, With<Player>>,
    state: Res<State<GameState>>,
    mut controls: ResMut<TouchControls>,
) {
    // Clear edge-triggered flags each frame
    controls.dash_just = false;
    controls.confirm_just = false;
    controls.back_just = false;
    controls.menu_up_just = false;
    controls.menu_down_just = false;
    controls.menu_diff_left = false;
    controls.menu_diff_right = false;
    controls.move_dir = Vec2::ZERO;
    controls.move_strength = 0.0;
    controls.move_target = None;
    controls.dash = false;

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
            // --- Multi-touch: 1st finger moves, 2nd finger dashes ---
            let active: Vec<_> = touches.iter().collect();
            let just_pressed: Vec<_> = touches.iter_just_pressed().collect();
            let touch_count = active.len();
            // Touches that were already down before this frame (steering finger).
            let held_prior = touch_count.saturating_sub(just_pressed.len());

            // Dash only when a *new* finger lands while another is already steering.
            // Two fingers landing on the same frame do not dash (avoids accidental dash).
            if held_prior >= 1 && !just_pressed.is_empty() {
                controls.dash_just = true;
                controls.dash = true;
            } else if touch_count >= 2 {
                controls.dash = true;
            }

            // Desktop mouse: right-click = dash (no second finger).
            if mouse_right_just {
                controls.dash_just = true;
                controls.dash = true;
            }

            // Move finger = oldest active touch (not a just-pressed dash finger if
            // we can exclude it), else any remaining held touch, else left mouse.
            let move_screen = pick_move_pointer(&active, &just_pressed, touch_count)
                .or_else(|| {
                    if mouse_down {
                        window.cursor_position()
                    } else {
                        None
                    }
                });

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
        GameState::Menu | GameState::GameOver | GameState::ModeSelect => {
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
                });

            let Some(pos) = tap else {
                // Two fingers held on menus → back
                if touches.iter().count() >= 2 {
                    controls.back_just = true;
                }
                return;
            };

            let y_ratio = pos.y / size.y;
            let x_ratio = pos.x / size.x;

            if x_ratio < 0.08 && !matches!(*state.get(), GameState::ModeSelect) {
                controls.back_just = true;
                return;
            }

            if matches!(*state.get(), GameState::ModeSelect) {
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
            } else {
                controls.confirm_just = true;
            }
        }
    }
}

/// Prefer a finger that is held for steering, not the brand-new second finger.
fn pick_move_pointer(
    active: &[&bevy::input::touch::Touch],
    just_pressed: &[&bevy::input::touch::Touch],
    touch_count: usize,
) -> Option<Vec2> {
    if active.is_empty() {
        return None;
    }

    // If multi-touch and some fingers were already down, steer with a non-just-pressed one.
    if touch_count >= 2 && !just_pressed.is_empty() {
        let just_ids: Vec<_> = just_pressed.iter().map(|t| t.id()).collect();
        if let Some(held) = active.iter().find(|t| !just_ids.contains(&t.id())) {
            return Some(held.position());
        }
    }

    // Single finger, or all new: use first active (stable order from Bevy).
    Some(active[0].position())
}
