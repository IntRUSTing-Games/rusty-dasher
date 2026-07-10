use crate::viewport::PlayBounds;
use bevy::prelude::*;
use std::f32::consts::TAU;

pub fn clamp_to_field(pos: &mut Vec3, radius: f32, bounds: &PlayBounds) {
    *pos = bounds.clamp(*pos, radius);
}

pub fn random_field_pos(margin: f32, bounds: &PlayBounds) -> Vec2 {
    bounds.random_pos(margin)
}

pub fn despawn_with<T: Component>(mut commands: Commands, q: Query<Entity, With<T>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

/// xorshift PRNG — no `SystemTime` (panics on wasm32-unknown-unknown).
pub fn rand_f32() -> f32 {
    use std::cell::Cell;
    thread_local! {
        static STATE: Cell<u64> = const { Cell::new(0xC0FFEE_BAD5EED_u64 | 1) };
    }

    STATE.with(|s| {
        let mut x = s.get();
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        s.set(x);
        let z = x.wrapping_mul(0x2545F4914F6CDD1D);
        ((z >> 32) as f32) / (u32::MAX as f32)
    })
}

pub fn rand_range(min: f32, max: f32) -> f32 {
    min + rand_f32() * (max - min)
}

#[allow(dead_code)]
pub fn rand_angle() -> f32 {
    rand_f32() * TAU
}
