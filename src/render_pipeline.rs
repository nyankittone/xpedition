use crate::game_vector::*;
use crate::render_entities;
use std::mem::MaybeUninit;

pub struct AsteroidRender {
    pub start_position: GameVector<u64>,
    pub end_position: GameVector<u64>,
    pub start_rotation: f32,
    pub end_rotation: f32,
    pub variation: u32,
}

pub struct PlayerRender {
    pub start_position: GameVector<u64>,
    pub end_position: GameVector<u64>,
    pub start_rotation: f32,
    pub end_rotation: f32,
}

const ASTEROID_LAYER_MAX: usize = 196;
const PROJECTILE_LAYER_MAX: usize = 420;

// Ordering of layers (from most in-front to most behind):
// * Projectile
// * Player
// * Asteroid
// * Gravity Body

pub struct RenderInfo<const A: usize> {
    asteroid_layer_count: u32,
    projectile_layer_count: u32,
    start_time: u64,
    delta_time: u64,

    player: Option<PlayerRender>,
    asteroids: [MaybeUninit<AsteroidRender>; A],
}

impl<const A: usize> RenderInfo<A> {
    pub fn new() -> Self {
        Self {
            asteroid_layer_count: 0,
            projectile_layer_count: 0,
            start_time: 0, // idk if 0 makes sense on initialization...
            delta_time: 0, // idk if 0 makes sense on initialization...
            player: None,
            asteroids: [const { MaybeUninit::uninit() }; A],
        }
    }

    pub fn set_player(
        &mut self,
        start_position: GameVector<u64>,
        end_position: GameVector<u64>,
        start_rotation: f32,
        end_rotation: f32,
    ) {
        self.player = Some(PlayerRender {
            start_position,
            end_position,
            start_rotation,
            end_rotation,
        });
    }

    pub fn unset_player(&mut self) {
        self.player = None;
    }

    pub fn add_asteroid(
        &mut self,
        start_position: GameVector<u64>,
        end_position: GameVector<u64>,
        start_rotation: f32,
        end_rotation: f32,
        variation: u32,
    ) -> Result<(), ()> {
        if self.asteroid_layer_count as usize >= A {
            return Err(());
        }

        self.asteroids[self.asteroid_layer_count as usize].write(AsteroidRender {
            start_position,
            end_position,
            start_rotation,
            end_rotation,
            variation,
        });
        self.asteroid_layer_count += 1;
        Ok(())
    }
}
