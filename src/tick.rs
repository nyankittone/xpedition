use core::{default::Default};
use std::cell::{RefCell, RefMut, Ref};

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

pub struct EntityPoolAllocation<const S: usize> {
    blocks_free: u32,
    fralloc_list: [u32; S],
    entities: [Option<RefCell<Entity>>; S],
}

impl<const S: usize> EntityPoolAllocation<S> {
    fn allocate(&mut self, entity: Entity) -> Option<u32> {
        if self.blocks_free == 0 {
            return None;
        }

        let returned = self.fralloc_list[self.blocks_free as usize - 1];
        let new_entity = self.entities.get_mut(returned as usize).unwrap();
        *new_entity = Some(RefCell::new(entity));

        self.blocks_free -= 1;
        Some(returned)
    }

    fn free(&mut self, index: u32) -> Result<(), ()> {
        if self.blocks_free as usize >= S {
            return Err(());
        }

        // find the index in the list of allocated indices.
        // Then move the element there to the front of the allocated indices.
        {
            let blocks_free_usize = self.blocks_free as usize;
            let allocated_indices = &mut self.fralloc_list[blocks_free_usize..];

            let Some((found_index, _)) = allocated_indices
                .iter()
                .enumerate()
                .find(|(_, e)| **e == index) else {
                    return Err(());
            };

            allocated_indices[..=found_index].rotate_right(1);
        }

        self.entities[index as usize] = None;
        self.blocks_free += 1;
        Ok(())
    }

    fn get(&self, id: u32) -> Option<Ref<'_, Entity>> {
        self.entities.get(id as usize)?.as_ref().map(|cell| cell.borrow())
    }

    fn get_mut(&self, id: u32) -> Option<RefMut<'_, Entity>> {
        self.entities.get(id as usize)?.as_ref().map(|cell| cell.borrow_mut())
    }
}

pub trait EntityPoolHandle {
    fn allocate(&mut self, entity: Entity) -> Option<u32>;
    fn free(&mut self, index: u32) -> Result<(), ()>;
    fn get(&self, id: u32) -> Option<Ref<'_, Entity>>;
    fn get_mut(&self, id: u32) -> Option<RefMut<'_, Entity>>;
}

pub struct EntityDirectAllocHandle<'a, const S: usize>(RefMut<'a, EntityPoolAllocation<S>>);
impl<'a, const S: usize> EntityPoolHandle for EntityDirectAllocHandle<'a, S> {
    fn allocate(&mut self, entity: Entity) -> Option<u32> {
        self.0.allocate(entity)
    }

    fn free(&mut self, index: u32) -> Result<(), ()> {
        self.0.free(index)
    }

    fn get(&self, id: u32) -> Option<Ref<'_, Entity>> {
        self.0.get(id)
    }

    fn get_mut(&self, id: u32) -> Option<RefMut<'_, Entity>> {
        self.0.get_mut(id)
    }
}

pub struct EntityIndirectAllocHandle<'a, const S: usize>(RefMut<'a, EntityPoolAllocation<S>>);
impl<'a, const S: usize> EntityPoolHandle for EntityIndirectAllocHandle<'a, S> {
    fn allocate(&mut self, entity: Entity) -> Option<u32> {
        None
    }

    fn free(&mut self, index: u32) -> Result<(), ()> {
        Err(())
    }

    fn get(&self, id: u32) -> Option<Ref<'_, Entity>> {
        self.0.get(id)
    }

    fn get_mut(&self, id: u32) -> Option<RefMut<'_, Entity>> {
        self.0.get_mut(id)
    }
}

pub struct EntityPool<const S: usize> {
    pool: Box<RefCell<EntityPoolAllocation<S>>>,
}

impl<const S: usize> EntityPool<S> {
    pub fn new() -> Self {
        Self {
            pool: Box::new(RefCell::new(EntityPoolAllocation {
                blocks_free: S as u32,
                fralloc_list: std::array::from_fn(|i| (S - i - 1) as u32),
                entities: [const{None}; S],
            },
        ))}
    }

    pub fn get_handle(&self) -> EntityDirectAllocHandle<'_, S> {
        EntityDirectAllocHandle(self.pool.borrow_mut())
    }

    pub fn iter(&self) -> EntityIterator<'_, S> {
        let i = self.pool.borrow().blocks_free;

        EntityIterator {
            i,
            pool: &self.pool,
        }
    }
}

pub struct EntityIterator<'a:, const S: usize> {
    pool: &'a RefCell<EntityPoolAllocation<S>>,
    i: u32,
}

impl<'a, const S: usize> Iterator for EntityIterator<'a, S> {
    type Item = (u32, EntityIndirectAllocHandle<'a, S>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i as usize >= S {
            return None;
        }

        let handle = self.pool.borrow_mut();
        let returned = handle.fralloc_list[self.i as usize];
        self.i += 1;

        Some((returned, EntityIndirectAllocHandle(handle)))
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
    let pool = EntityPool::<1024>::new();
    let mut pool_handle = pool.get_handle();
    pool_handle.allocate(Entity::Player(Player {
        velocity: GameVector {
            x: 1,
            y: 1,
        },
        ..Player::default()
    }));
    pool_handle.allocate(Entity::Asteriod(Asteroid::default()));

    drop(pool_handle);

    for (id, handle) in pool.iter() {
        eprintln!("I am ID {}.", id);

        let player = handle.get_mut(0);
        if let Some(mut player) = player {
            match &mut *player {
                Entity::Player(player) => {
                    player.velocity += 1;
                    eprintln!("Player velocity is now {:?}", player.velocity);
                },
                _ => (),
            }
        }
    }
}

