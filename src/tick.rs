use core::{default::Default, ops::{Deref, DerefMut}};

use crate::game_vector::GameVector;


pub struct Player {
    rotation: f32,
    velocity: GameVector<i32>,
    health_current: u32,
    health_max: u32,
    ammo_current: u32,
    ammo_max: u32,
    oxygen_current: u32,
    oxygen_max: u32,
    fuel_current: u32,
    fuel_max: u32,
}

pub struct Asteroid {
    rotation: f32,
    velocity: GameVector<i32>,
    health_current: u16,
    health_max: u16,
    scale: f32,
    points_count: u8,
    points: [f32; 8],
}

pub struct Projectile {
    velocity: GameVector<i32>,
    color: u32,
}

// TODO: We're going to need some memory allocator for all of these kind of objects. How fun.
// A simple memory pool with a free list is probably the simplest approach, but will waste a lot
// of memory. Especially if I have a lot of projectiles on-screen. However, I think this is the best
// approach for my needs here.

pub enum Entity {
    Player(Player),
    Asteriod(Asteroid),
    Projectile(Projectile),
}

struct EntityPoolAllocation<const S: usize> {
    blocks_free: u32,
    fralloc_list: [u32; S],
    entities: [Option<Entity>; S],
}

pub struct EntityPool<const S: usize>(Box<EntityPoolAllocation<S>>);

impl<const S: usize> EntityPool<S> {
    pub fn new() -> Self {
        Self(Box::new(EntityPoolAllocation {
            blocks_free: S as u32,
            fralloc_list: std::array::from_fn(|i| (S - i - 1) as u32),
            entities: [const{None}; S],
        }))
    }

    pub fn allocate(&mut self) -> Option<&mut Option<Entity>> {
        if self.0.blocks_free == 0 {
            return None;
        }

        let returned = self.0.entities.get_mut(self.0.fralloc_list[self.0.blocks_free as usize - 1] as usize).unwrap();
        Some(returned)
    }
}

