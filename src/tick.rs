use core::default::Default;

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

pub enum EntityCell<'a> {
    Unallocated(Option<&'a EntityCell<'a>>),
    Allocated {
        next: Option<&'a EntityCell<'a>>,
        entity: Entity,
    },
}

impl<'a> Default for EntityCell<'a> {
    fn default() -> Self {
        Self::Unallocated(None)
    }
}

pub struct EntityManager<'a> {
    pool: &'a [EntityCell<'a>],
    free_head: Option<&'a EntityCell<'a>>,
    alloc_head: Option<&'a EntityCell<'a>>,
}

impl<'a> EntityManager<'a> {
    pub fn manage(cells: &'a mut [EntityCell<'a>]) -> Self {
        for i in 0..(cells.len() - 1) {
            cells[i] = EntityCell::Unallocated(Some(&cells[i + 1]));
        }
        cells[cells.len() - 1] = EntityCell::Unallocated(None);

        Self {
            pool: cells,
            free_head: Some(&cells[0]),
            alloc_head: None,
        }
    }
}

