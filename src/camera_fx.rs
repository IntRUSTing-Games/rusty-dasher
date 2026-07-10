use crate::components::MainCamera;
use crate::events::{PlayerDashed, PlayerHit, StarCollected};
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ScreenShake {
    pub trauma: f32,
}

impl ScreenShake {
    pub fn add(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }
}

pub fn shake_on_events(
    mut shake: ResMut<ScreenShake>,
    mut hits: MessageReader<PlayerHit>,
    mut collects: MessageReader<StarCollected>,
    mut dashes: MessageReader<PlayerDashed>,
) {
    for hit in hits.read() {
        shake.add(if hit.fatal { 0.85 } else { 0.55 });
    }
    for c in collects.read() {
        if c.combo >= 6 {
            shake.add(0.12);
        }
    }
    for _ in dashes.read() {
        shake.add(0.18);
    }
}

pub fn apply_screen_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut cam: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut tf) = cam.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    shake.trauma = (shake.trauma - dt * 1.6).max(0.0);
    if shake.trauma <= 0.001 {
        tf.translation.x = 0.0;
        tf.translation.y = 0.0;
        return;
    }
    let t = shake.trauma * shake.trauma;
    let max_offset = 14.0 * t;
    // cheap deterministic shake from time
    let seed = time.elapsed_secs() * 40.0;
    tf.translation.x = seed.sin() * max_offset;
    tf.translation.y = (seed * 1.7).cos() * max_offset;
}
