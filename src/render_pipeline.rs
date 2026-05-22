use crate::game_vector::*;
use crate::render_entities;
use std::mem::MaybeUninit;
use std::clone::Clone;

pub struct AsteroidRender {
    pub start_position: GameVector<i64>,
    pub end_position: GameVector<i64>,
    pub start_rotation: f32,
    pub end_rotation: f32,
    pub variation: u32,
}

#[derive(Clone)]
pub struct PlayerRender {
    pub start_position: GameVector<i64>,
    pub end_position: GameVector<i64>,
    pub start_rotation: f32,
    pub end_rotation: f32,
}

pub struct ProjectileRender {
    pub start_position: GameVector<i64>,
    pub end_position: GameVector<i64>,
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
    fn new() -> Self {
        Self {
            asteroid_layer_count: 0,
            projectile_layer_count: 0,
            start_time: 0, // idk if 0 makes sense on initialization...
            delta_time: 0, // idk if 0 makes sense on initialization...
            player: None,
            asteroids: [const { MaybeUninit::uninit() }; A],
            projectiles: [const { MaybeUninit::uninit() }; P],
        }
    }
}

impl<const A: usize, const P: usize> Clone for RenderInfo<A, P> {
    fn clone(&self) -> Self {
        let mut returned = MaybeUninit::<Self>::uninit();
        unsafe {
            std::ptr::copy(self as *const Self, returned.as_mut_ptr(), 1);
            returned.assume_init()
        }
    }
}

pub struct WriteRenderInfo<const A: usize, const P: usize>(RenderInfo<A, P>);
impl<const A: usize, const P: usize> WriteRenderInfo<A, P> {
    pub fn new() -> Self {
        Self(RenderInfo::new())
    }

    pub fn set_player(
        &mut self,
        start_position: GameVector<i64>,
        end_position: GameVector<i64>,
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
        start_position: GameVector<i64>,
        end_position: GameVector<i64>,
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
        start_position: GameVector<i64>,
        end_position: GameVector<i64>,
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

#[derive(Clone)]
pub struct ReadRenderInfo<const A: usize, const P: usize>(RenderInfo<A, P>);
pub struct AsteroidRenderIter<'a, const A: usize, const P: usize> {
    render_info: &'a ReadRenderInfo<A, P>,
    i: u32,
}

impl<'a, const A: usize, const P: usize> Iterator for AsteroidRenderIter<'a, A, P> {
    type Item = &'a AsteroidRender;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.render_info.0.asteroid_layer_count {
            return None;
        }

        let returned = unsafe {
            self.render_info.0.asteroids.get_unchecked(self.i as usize).assume_init_ref()
        };

        self.i += 1;
        Some(returned)
    }
}

pub struct ProjectileRenderIter<'a, const A: usize, const P: usize> {
    render_info: &'a ReadRenderInfo<A, P>,
    i: u32,
}

impl<'a, const A: usize, const P: usize> Iterator for ProjectileRenderIter<'a, A, P> {
    type Item = &'a ProjectileRender;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.render_info.0.projectile_layer_count {
            return None;
        }

        let returned = unsafe {
            self.render_info.0.projectiles.get_unchecked(self.i as usize).assume_init_ref()
        };

        self.i += 1;
        Some(returned)
    }
}

impl<const A: usize, const P: usize> ReadRenderInfo<A, P> {
    pub fn new() -> Self {
        Self(RenderInfo::new())
    }

    pub const fn iter_asteroids(&self) -> AsteroidRenderIter<'_, A, P> {
        AsteroidRenderIter {
            render_info: self,
            i: 0,
        }
    }

    pub const fn iter_projectiles(&self) -> ProjectileRenderIter<'_, A, P> {
        ProjectileRenderIter {
            render_info: self,
            i: 0,
        }
    }

    pub const fn get_player(&self) -> Option<&PlayerRender> {
        self.0.player.as_ref()
    }
}

