use core::{default::Default};
use std::cell::{RefCell, RefMut, Ref};
use std::mem::MaybeUninit;
use std::clone::Clone;
use std::time;
use std::thread;

use crate::game_vector::GameVector;

#[derive(Debug, Copy, Clone)]
pub struct Player {
    pub position: GameVector<i64>,
    pub rotation: f32,
    pub velocity: GameVector<i32>,
    pub health_current: u32,
    pub health_max: u32,
    pub ammo_current: u32,
    pub ammo_max: u32,
    pub oxygen_current: u32,
    pub oxygen_max: u32,
    pub fuel_current: u32,
    pub fuel_max: u32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: GameVector::new(0, 0),
            rotation: 0.0,
            velocity: GameVector::new(0, 0),
            health_current: 100,
            health_max: 100,
            ammo_current: 0,
            ammo_max: 0,
            oxygen_current: 1000,
            oxygen_max: 1000,
            fuel_current: 1000,
            fuel_max: 1000,
        }
    }
}

// Having GameVector derive default might be a really bad idea long-term, but this is temporary.
#[derive(Default, Debug, Copy, Clone)]
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

#[derive(Default, Debug, Copy, Clone)]
pub struct Projectile {
    position: GameVector<i64>,
    velocity: GameVector<i32>,
    color: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum Entity {
    Player(Player),
    Asteriod(Asteroid),
    Projectile(Projectile),
}

pub struct EntityPoolAllocation<const S: usize, const Q: usize> {
    blocks_free: u32,
    fralloc_list: [u32; S],
    entities: [Option<RefCell<Entity>>; S],
    alloc_queue: [MaybeUninit<Entity>; Q],
    alloc_queue_length: u32,
    free_queue: [MaybeUninit<u32>; Q],
    free_queue_length: u32,
}

impl<const S: usize, const Q: usize> EntityPoolAllocation<S, Q> {
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

    fn queue_allocate(&mut self, entity: Entity) -> Result<(), ()> {
        if self.alloc_queue_length as usize >= Q {
            return Err(());
        }

        unsafe {
            self.alloc_queue.get_unchecked_mut(self.alloc_queue_length as usize).write(entity);
        }
        self.alloc_queue_length += 1;

        Ok(())
    }

    fn queue_free(&mut self, id: EntityID) -> Result<(), ()> {
        if self.alloc_queue_length as usize >= Q {
            return Err(());
        }

        unsafe {
            self.free_queue.get_unchecked_mut(self.free_queue_length as usize).write(id);
        }
        self.free_queue_length += 1;

        Ok(())
    }

    fn flush_queue(&mut self) -> Result<(), ()> {
        // first free anything in the free queue
        for i in 0..(self.free_queue_length) {
            unsafe {
                self.free(self.free_queue.get_unchecked(i as usize).assume_init());
            }
        }
        self.free_queue_length = 0;

        // then allocate anything that needs allocation in the alloc queue
        for i in 0..(self.alloc_queue_length) {
            unsafe {
                self.allocate (
                    self.alloc_queue.get_unchecked(i as usize).assume_init()
                ).ok_or(())?;
            }
        }
        self.alloc_queue_length = 0;

        Ok(())
    }

    fn get(&self, id: u32) -> Option<Ref<'_, Entity>> {
        self.entities.get(id as usize)?.as_ref().map(|cell| cell.borrow())
    }

    fn get_mut(&self, id: u32) -> Option<RefMut<'_, Entity>> {
        self.entities.get(id as usize)?.as_ref().map(|cell| cell.borrow_mut())
    }
}

pub struct EntityDirectAllocHandle<'a, const S: usize, const Q: usize>(RefMut<'a, EntityPoolAllocation<S, Q>>);
impl<'a, const S: usize, const Q: usize> EntityDirectAllocHandle<'a, S, Q> {
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

pub struct EntityIndirectAllocHandle<'a, const S: usize, const Q: usize>(RefMut<'a, EntityPoolAllocation<S, Q>>);
impl<'a, const S: usize, const Q: usize> EntityIndirectAllocHandle<'a, S, Q> {
    fn allocate(&mut self, entity: Entity) -> Result<(), ()> {
        self.0.queue_allocate(entity)
    }

    fn free(&mut self, index: u32) -> Result<(), ()> {
        self.0.queue_free(index)
    }

    fn get(&self, id: u32) -> Option<Ref<'_, Entity>> {
        self.0.get(id)
    }

    fn get_mut(&self, id: u32) -> Option<RefMut<'_, Entity>> {
        self.0.get_mut(id)
    }

    pub fn sub_iter(&'a self, progress: u32) -> SubEntityIterator<'a, S, Q> {
        SubEntityIterator {
            handle: &self,
            i: self.0.blocks_free + progress + 1,
        }
    }
}

pub struct EntityPool<const S: usize, const Q: usize> {
    pool: Box<RefCell<EntityPoolAllocation<S, Q>>>,
}

impl<const S: usize, const Q: usize> EntityPool<S, Q> {
    pub fn new() -> Self {
        Self {
            pool: Box::new(RefCell::new(EntityPoolAllocation {
                blocks_free: S as u32,
                fralloc_list: std::array::from_fn(|i| (S - i - 1) as u32),
                entities: [const{None}; S],
                alloc_queue: [const{MaybeUninit::uninit()}; Q],
                alloc_queue_length: 0,
                free_queue: [const{MaybeUninit::uninit()}; Q],
                free_queue_length: 0,
            },
        ))}
    }

    pub fn get_handle(&self) -> EntityDirectAllocHandle<'_, S, Q> {
        EntityDirectAllocHandle(self.pool.borrow_mut())
    }

    pub fn flush_queue(&self) -> Result<(), ()> {
        self.pool.borrow_mut().flush_queue()
    }

    pub fn iter(&self) -> EntityIterator<'_, S, Q> {
        let i = self.pool.borrow().blocks_free;

        EntityIterator {
            i,
            pool: &self.pool,
        }
    }
}

pub type EntityID = u32;
pub type EntityIterationCount = u32;

pub struct EntityIterator<'a:, const S: usize, const Q: usize> {
    pool: &'a RefCell<EntityPoolAllocation<S, Q>>,
    i: u32,
}

impl<'a, const S: usize, const Q: usize> Iterator for EntityIterator<'a, S, Q> {
    type Item = (EntityID, EntityIndirectAllocHandle<'a, S, Q>, EntityIterationCount);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i as usize >= S {
            return None;
        }

        let handle = self.pool.borrow_mut();
        let returned = handle.fralloc_list[self.i as usize];
        let count = self.i - handle.blocks_free;
        self.i += 1;

        Some((returned, EntityIndirectAllocHandle(handle), count))
    }
}

pub struct SubEntityIterator<'a, const S: usize, const Q: usize> {
    handle: &'a EntityIndirectAllocHandle<'a, S, Q>,
    i: u32,
}

impl<'a, const S: usize, const Q: usize> Iterator for SubEntityIterator<'a, S, Q> {
    type Item = (EntityID, EntityIterationCount);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i as usize >= S {
            return None;
        }

        let id = self.handle.0.fralloc_list[self.i as usize];
        let count = self.i - self.handle.0.blocks_free;
        self.i += 1;

        Some((id, count))
    }
}

fn tick<const S: usize, const Q: usize>(pool: &EntityPool<S, Q>, delta_time: u64) {
    // iterate on each entity in the pool. Then flush the alloc and free buffers afterwards.
    for (id, handle, _) in pool.iter() {
        let mut entity = handle.get_mut(id).unwrap();
        if let Entity::Player(player) = &mut *entity {
            eprintln!("Player position is {:?}", player.position);
            let modification = player.velocity * (delta_time as i32);
            player.position += GameVector {x: modification.x as i64, y: modification.y as i64};
        }
    }
}

pub fn start_physics() {
    let pool = EntityPool::<1337, 67>::new();

    // Allocate any initial objects needed for the game to work. For now, that will just be the
    // player. In the future, it'll also include planets or large asteroids with gravity.
    // Should this function handle events of some kind? Or is that the responsibility of the ticking
    // function?
    // Run the ticking function in a loop forever.

    let mut handle = pool.get_handle();
    let player_id = handle.allocate(Entity::Player(Player::new())).unwrap();

    // Let's add a little bit of initial velocity to the player for testing purposes.
    let mut player = handle.get_mut(player_id).unwrap();
    if let Entity::Player(player) = &mut *player {
        player.velocity.x += 128;
        player.velocity.y += 16;
    }

    drop(player);
    drop(handle);

    // Run the ticking function, once every 60 ticks for now. Later I want to make it work well for
    // running at 240 ticks/second max.

    let mut delta_time = 1000000 / 240;
    const TICK_RATE: time::Duration = time::Duration::from_micros(1000000 / 60);

    loop {
        let before_tick = time::Instant::now();
        tick(&pool, delta_time);

        let elapsed = before_tick.elapsed();
        if elapsed > TICK_RATE {
            delta_time = elapsed.as_micros() as u64;
            continue;
        }

        delta_time = TICK_RATE.as_micros() as u64;
        thread::sleep(elapsed);
    }
}

// TODO: Add unit tests for the allocator. HA!
