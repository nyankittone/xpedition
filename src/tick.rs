use core::{default::Default, ops::{Deref, DerefMut}};

use crate::game_vector::GameVector;

#[derive(Default, Debug)]
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

// Having GameVector derive default might be a really bad idea long-term, but this is temporary.
#[derive(Default, Debug)]
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

#[derive(Debug)]
pub struct Projectile {
    position: GameVector<i64>,
    velocity: GameVector<i32>,
    color: u32,
}

#[derive(Debug)]
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

    pub fn iter(&mut self) -> EntityIterator<'_, S> {
        EntityIterator {
            i: self.0.blocks_free,
            pool: self,
        }
    }
}

pub struct EntityIterator<'a:, const S: usize> {
    pool: &'a mut EntityPool<S>,
    i: u32,
}

impl<'a, const S: usize> Iterator for EntityIterator<'a, S> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i as usize >= S {
            return None;
        }

        let returned = self.pool.0.fralloc_list[self.i as usize];
        self.i += 1;

        Some(returned)
    }
}

impl<'a, const S: usize> EntityIterator<'a, S> {
    pub fn next_ref(&mut self) -> Option<(u32, &mut Entity)> {
        if self.i as usize >= S {
            return None;
        }

        let index = self.pool.0.fralloc_list[self.i as usize];
        self.i += 1;

        Some((index, self.pool.get_mut(index).unwrap()))
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

pub fn start_physics() {
    let mut pool = EntityPool::<1024>::new();
    pool.allocate(Entity::Player(Player {
        velocity: GameVector {
            x: 1,
            y: 1,
        },
        ..Player::default()
    }));

    pool.allocate(Entity::Asteriod(Asteroid::default()));

    for id in pool.iter() {
        let entity = pool.get_mut(id).unwrap();
        eprintln!("{id:#?}");
    }
}

