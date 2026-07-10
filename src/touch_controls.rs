//! Touch / pointer controls for browser + phone.
//!
//! Playing:
//!   - Drag on left ~70% of screen → move toward that world point
//!   - Tap / hold right edge → dash
//! Menus:
//!   - Tap center → confirm
//!   - Tap top third → previous mode
//!   - Tap bottom third → next mode
//!   - Two-finger tap or far-left edge → back

use crate::components::{MainCamera, Player};
use crate::state::GameState;
use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct TouchControls {
    /// Normalized movement intent (length 0..=1)
    pub move_dir: Vec2,
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
    controls.dash = false;

    let Ok(window) = windows.single() else {
        return;
    };
    let size = window.size();
    if size.x <= 1.0 || size.y <= 1.0 {
        return;
    }

    // Collect active pointer positions (touches + mouse drag)
    let mut points: Vec<Vec2> = touches.iter().map(|t| t.position()).collect();
    let mouse_just = mouse.just_pressed(MouseButton::Left);
    let mouse_down = mouse.pressed(MouseButton::Left);
    if mouse_down {
        if let Some(pos) = window.cursor_position() {
            points.push(pos);
        }
    }

    if points.is_empty() {
        return;
    }

    match *state.get() {
        GameState::Playing => {
            // Right strip = dash zone (~18% of width)
            let dash_x = size.x * 0.82;
            let mut move_points = Vec::new();
            for p in &points {
                if p.x >= dash_x {
                    controls.dash = true;
                } else {
                    move_points.push(*p);
                }
            }
            // Just-pressed dash: new touch in dash zone or mouse click there
            for t in touches.iter_just_pressed() {
                if t.position().x >= dash_x {
                    controls.dash_just = true;
                }
            }
            if mouse_just {
                if let Some(pos) = window.cursor_position() {
                    if pos.x >= dash_x {
                        controls.dash_just = true;
                    }
                }
            }

            // Move toward world position of primary move pointer
            if let Some(screen) = move_points.first().copied() {
                if let Ok((camera, cam_tf)) = camera_q.single() {
                    if let Ok(world) = camera.viewport_to_world_2d(cam_tf, screen) {
                        if let Ok(player_tf) = player_q.single() {
                            let delta = world - player_tf.translation.truncate();
                            if delta.length_squared() > 16.0 {
                                controls.move_dir = delta.normalize();
                            }
                        }
                    }
                }
            }
        }
        GameState::Menu | GameState::GameOver | GameState::ModeSelect => {
            // Use first just-pressed touch or mouse click
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
                // Two fingers held → back
                if touches.iter().count() >= 2 {
                    controls.back_just = true;
                }
                return;
            };

            let y_ratio = pos.y / size.y;
            let x_ratio = pos.x / size.x;

            // Far left edge = back (except mode select uses left/right for difficulty)
            if x_ratio < 0.08 && !matches!(*state.get(), GameState::ModeSelect) {
                controls.back_just = true;
                return;
            }

            if matches!(*state.get(), GameState::ModeSelect) {
                // Side strips change difficulty; vertical thirds change mode; center starts.
                if x_ratio < 0.18 {
                    controls.menu_diff_left = true;
                } else if x_ratio > 0.82 {
                    controls.menu_diff_right = true;
                } else if y_ratio < 0.30 {
                    controls.menu_up_just = true;
                } else if y_ratio > 0.70 {
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
