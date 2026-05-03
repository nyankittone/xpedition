use core::{default::Default, ops::{Deref, DerefMut}};

use crate::game_vector::GameVector;


pub struct Player {
    position: GameVector<i64>,
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
    position: GameVector<i64>,
    rotation: f32,
    velocity: GameVector<i32>,
    health_current: u16,
    health_max: u16,
    scale: f32,
    points_count: u8,
    points: [f32; 8],
}

pub struct Projectile {
    position: GameVector<i64>,
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

    pub fn allocate(&mut self, entity: Entity) -> Option<u32> {
        if self.0.blocks_free == 0 {
            return None;
        }

        let returned = self.0.fralloc_list[self.0.blocks_free as usize - 1];
        let new_entity = self.0.entities.get_mut(returned as usize).unwrap();
        *new_entity = Some(entity);

        self.0.blocks_free -= 1;
        Some(returned)
    }

    pub fn free(&mut self, index: u32) -> Result<(), ()> {
        if self.0.blocks_free as usize >= S {
            return Err(());
        }

        // find the index in the list of allocated indices.
        // Then move the element there to the front of the allocated indices.
        {
            let blocks_free_usize = self.0.blocks_free as usize;
            let allocated_indices = &mut self.0.fralloc_list[blocks_free_usize..];

            let Some((found_index, _)) = allocated_indices
                .iter()
                .enumerate()
                .find(|(_, e)| **e == index) else {
                    return Err(());
            };

            allocated_indices[..=found_index].rotate_right(1);
        }

        self.0.entities[index as usize] = None;
        self.0.blocks_free += 1;
        Ok(())
    }

    pub fn get(&self, id: u32) -> Option<&Entity> {
        self.0.entities.get(id as usize).map_or(None, |thing| thing.as_ref())
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut Entity> {
        self.0.entities.get_mut(id as usize).map_or(None, |thing| thing.as_mut())
    }
}

// How will iterating over the allocated objects work?
// The problem is that the entities allocated are free to allocate or free new entities as they
// please, and this would happen while iterating. This could mess with the iterator itself in a
// not-so-desireable way.
// Either we design our allocator and iterators so that allocations and deletions are iterator-aware
// and will update any active iterators when they occur so that they'll not iterate on inactive
// cells, etc... or we don't allocate and delete things right away, and instead stage them until
// iteration is complete.
// The latter strategy seems easier and more straight-forward to accompilish, at the cost of *maybe*
// being less flexiable. There is probably not a good reason to need to run a brand new entity
// immediately before a tick is finished.

