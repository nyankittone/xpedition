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

pub struct ProjectileRender {
    pub start_position: GameVector<u64>,
    pub end_position: GameVector<u64>,
}

// Ordering of layers (from most in-front to most behind):
// * Projectile
// * Player
// * Asteroid
// * Gravity Body

pub struct RenderInfo<const A: usize, const P: usize> {
    asteroid_layer_count: u32,
    projectile_layer_count: u32,
    start_time: u64,
    delta_time: u64,

    player: Option<PlayerRender>,
    asteroids: [MaybeUninit<AsteroidRender>; A],
    projectiles: [MaybeUninit<ProjectileRender>; P],
}

impl<const A: usize, const P: usize> RenderInfo<A, P> {
    pub fn new() -> WriteRenderInfo<A> {
        WriteRenderInfo(Self {
            asteroid_layer_count: 0,
            projectile_layer_count: 0,
            start_time: 0, // idk if 0 makes sense on initialization...
            delta_time: 0, // idk if 0 makes sense on initialization...
            player: None,
            asteroids: [const { MaybeUninit::uninit() }; A],
            projectiles: [const { MaybeUninit::uninit() }; P],
        })
    }
}

pub struct WriteRenderInfo<const A: usize, const P: usize>(RenderInfo<A, P>);
impl<const A: usize, const P: usize> WriteRenderInfo<A, P> {
    pub fn set_player(
        &mut self,
        start_position: GameVector<u64>,
        end_position: GameVector<u64>,
        start_rotation: f32,
        end_rotation: f32,
    ) {
        self.0.player = Some(PlayerRender {
            start_position,
            end_position,
            start_rotation,
            end_rotation,
        });
    }

    pub fn unset_player(&mut self) {
        self.0.player = None;
    }

    pub fn add_asteroid(
        &mut self,
        start_position: GameVector<u64>,
        end_position: GameVector<u64>,
        start_rotation: f32,
        end_rotation: f32,
        variation: u32,
    ) -> Result<(), ()> {
        if self.0.asteroid_layer_count as usize >= A {
            return Err(());
        }

        self.0.asteroids[self.0.asteroid_layer_count as usize].write(AsteroidRender {
            start_position,
            end_position,
            start_rotation,
            end_rotation,
            variation,
        });
        self.0.asteroid_layer_count += 1;
        Ok(())
    }

    pub fn add_projectile(
        &mut self,
        start_position: GameVector<u64>,
        end_position: GameVector<u64>,
    ) -> Result<(), ()> {
        if self.0.projectile_layer_count as usize >= A {
            return Err(());
        }

        self.0.projectiles[self.0.projectile_layer_count as usize].write(ProjectileRender {
            start_position,
            end_position,
        });
        self.0.projectile_layer_count += 1;
        Ok(())
    }

    pub fn commit(self) -> ReadRenderInfo<A, P> {
        ReadRenderInfo(self.0)
    }
}

pub struct ReadRenderInfo<const A: usize, const P: usize>(RenderInfo<A, P>);

