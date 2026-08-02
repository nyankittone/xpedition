use crate::log;
use std::cell::{Ref, RefCell, RefMut};
use std::mem::MaybeUninit;
use std::sync;
use std::sync::mpsc;
use std::thread;
use std::time;

use crate::entities::*;
use crate::events;
use crate::math_types::*;
use crate::render_pipeline as rp;

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
                .find(|(_, e)| **e == index)
            else {
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
            self.alloc_queue
                .get_unchecked_mut(self.alloc_queue_length as usize)
                .write(entity);
        }
        self.alloc_queue_length += 1;

        Ok(())
    }

    fn queue_free(&mut self, id: EntityID) -> Result<(), ()> {
        if self.alloc_queue_length as usize >= Q {
            return Err(());
        }

        unsafe {
            self.free_queue
                .get_unchecked_mut(self.free_queue_length as usize)
                .write(id);
        }
        self.free_queue_length += 1;

        Ok(())
    }

    fn flush_queue(&mut self) -> Result<(), ()> {
        // first free anything in the free queue
        for i in 0..(self.free_queue_length) {
            unsafe {
                let _ = self.free(self.free_queue.get_unchecked(i as usize).assume_init());
            }
        }
        self.free_queue_length = 0;

        // then allocate anything that needs allocation in the alloc queue
        for i in 0..(self.alloc_queue_length) {
            unsafe {
                self.allocate(self.alloc_queue.get_unchecked(i as usize).assume_init())
                    .ok_or(())?;
            }
        }
        self.alloc_queue_length = 0;

        Ok(())
    }

    fn get(&self, id: u32) -> Option<Ref<'_, Entity>> {
        self.entities
            .get(id as usize)?
            .as_ref()
            .map(|cell| cell.borrow())
    }

    fn get_mut(&self, id: u32) -> Option<RefMut<'_, Entity>> {
        self.entities
            .get(id as usize)?
            .as_ref()
            .map(|cell| cell.borrow_mut())
    }
}

pub struct EntityDirectAllocHandle<'a, const S: usize, const Q: usize>(
    RefMut<'a, EntityPoolAllocation<S, Q>>,
);
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

pub struct EntityIndirectAllocHandle<'a, const S: usize, const Q: usize>(
    RefMut<'a, EntityPoolAllocation<S, Q>>,
);
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
                entities: [const { None }; S],
                alloc_queue: [const { MaybeUninit::uninit() }; Q],
                alloc_queue_length: 0,
                free_queue: [const { MaybeUninit::uninit() }; Q],
                free_queue_length: 0,
            })),
        }
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

pub struct EntityIterator<'a, const S: usize, const Q: usize> {
    pool: &'a RefCell<EntityPoolAllocation<S, Q>>,
    i: u32,
}

impl<'a, const S: usize, const Q: usize> Iterator for EntityIterator<'a, S, Q> {
    type Item = (
        EntityID,
        EntityIndirectAllocHandle<'a, S, Q>,
        EntityIterationCount,
    );

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

enum TickSignal {
    Normal,
    StopMe,
}

struct TickState<'a, const S: usize, const Q: usize, const A: usize, const P: usize> {
    pool: EntityPool<S, Q>,
    pipeline: &'a sync::Mutex<rp::ReadRenderInfo<A, P>>,
    physics_events: mpsc::Receiver<events::PhysicsEvent>,
    player: Player,
    player_control: events::PlayerControl,
    camera_position: GameVector<i64>,
}

impl<'a, const S: usize, const Q: usize, const A: usize, const P: usize> TickState<'a, S, Q, A, P> {
    fn new(
        pipeline: &'a sync::Mutex<rp::ReadRenderInfo<A, P>>,
        physics_events: mpsc::Receiver<events::PhysicsEvent>,
    ) -> Self {
        Self {
            pool: EntityPool::<S, Q>::new(),
            pipeline,
            physics_events,
            player: Player::new(),
            player_control: events::PlayerControl::new(),
            camera_position: GameVector::new(0, 0),
        }
    }

    fn tick(&mut self, delta_time: f64) -> TickSignal {
        for event in self.physics_events.try_iter() {
            use events::PhysicsEvent as pe;
            match event {
                pe::Stop => {
                    return TickSignal::StopMe;
                }
                pe::PlayerControl(control) => {
                    eprintln!("{:?}", control);
                    self.player_control = control;
                }
                _ => (),
            }
        }

        let mut proto_pipeline = rp::WriteRenderInfo::<A, P>::new();

        // iterate on each entity in the pool. Then flush the alloc and free buffers afterwards.
        for (id, handle, _) in self.pool.iter() {
            let mut entity = handle.get_mut(id).unwrap();
        }

        let _ = self.pool.flush_queue();

        // handle player-specific physics. This owuld make sense to move out to another function if
        // it grows in size.
        update_player(self, delta_time, &mut proto_pipeline);

        // push the worked-on pipeline so the renderer can now use it
        let mut pipeline_handle = self.pipeline.lock().expect("Pipeline fucking died!!!!!!");
        *pipeline_handle = proto_pipeline.commit();
        drop(pipeline_handle);

        TickSignal::Normal
    }
}

fn update_player<const S: usize, const Q: usize, const A: usize, const P: usize>(
    state: &mut TickState<S, Q, A, P>,
    delta_time: f64,
    proto_pipeline: &mut rp::WriteRenderInfo<A, P>,
) {
    let old_position = state.player.position;
    let old_rotation = state.player.rotation;

    // Constants defined for specifying acceleration from player's boosters.
    // I might want to move these outside of this function and have them be global constants.
    const MAIN_BOOSTER_ACCELERATION: f64 = 80000.0;
    const TURBO_BOOSTER_ACCELERATION: f64 = 500000.0;

    // Constants defined for the rate at which the player slows down if not boosting.
    const STATIC_DECELERATION: f64 = 1800.0;
    const DYNAMIC_DECELERATION_FACTOR: f64 = 0.275;

    const ROTATION_TORQUE: f64 = 9.0;
    const STATIC_ANGULAR_DECELERATION: f64 = 0.5;
    const DYNAMIC_ANGULAR_DECELLERATION_FACTOR: f64 = 0.67;

    // Updating position
    // TODO: HOLY SHIT BRAH LEARN HOW TO DO INTEGRALS FOR APPLYING ACCELERATION PERFECTLY YOU
    // DAUGHTER OF A BITCH
    let mut speed_mod = 0.0f64;
    if state.player_control.big_forward {
        speed_mod += TURBO_BOOSTER_ACCELERATION;
    }

    if state.player_control.main_forward {
        speed_mod += MAIN_BOOSTER_ACCELERATION;
    }

    // Reading this code after not looking at it for 2.5 months is a great way to feel like a huge
    // idiot

    let deceleration_ratio = {
        let combined_velocity = (state.player.velocity.x.powf(2.0) + state.player.velocity.y.powf(2.0)).sqrt();
        if combined_velocity == 0.0 {
            0.0
        } else {
            let deceleration = STATIC_DECELERATION + combined_velocity * DYNAMIC_DECELERATION_FACTOR;
            deceleration / combined_velocity
        }
    };

    // I will be real asf, these calculations are probably not fully correct. I was really
    // sleep-deprived when writing this. TODO: Check accuracy of this BS.
    let deceleration_vector = GameVector {
        x: state.player.velocity.x * deceleration_ratio,
        y: state.player.velocity.y * deceleration_ratio,
    };

    let velocity_mod = GameVector {
        x: f64::sin(state.player.rotation.into()) * speed_mod,
        y: f64::cos(state.player.rotation.into()) * -speed_mod,
    };
    let velocity_mod = velocity_mod - deceleration_vector;

    state.player.velocity += velocity_mod / 2.0 * delta_time;

    let casted_velocity = GameVector {
        x: (state.player.velocity.x * delta_time) as i64,
        y: (state.player.velocity.y * delta_time) as i64,
    };
    state.player.position += casted_velocity;

    state.player.velocity += velocity_mod / 2.0 * delta_time;

    // Updating rotation
    // TODO: It would be fun to see if I could reduce the number of potential branches taken here...
    // In the future, if I need to optimize. I dislike how many if statements there are here.
    let mut rotation_mod = if state.player.rotation_velocity > 0.0 {
        -STATIC_ANGULAR_DECELERATION 
    } else {
        STATIC_ANGULAR_DECELERATION 
    };

    rotation_mod -= state.player.rotation_velocity * DYNAMIC_ANGULAR_DECELLERATION_FACTOR;

    rotation_mod += match state.player_control {
        events::PlayerControl {
            rotate_left: true,
            rotate_right: false,
            ..
        } => -ROTATION_TORQUE,
        events::PlayerControl {
            rotate_left: false,
            rotate_right: true,
            ..
        } => ROTATION_TORQUE,
        _ => 0.0,
    };

    state.player.rotation_velocity += rotation_mod / 2.0 * delta_time;
    state.player.rotation += state.player.rotation_velocity * delta_time;
    state.player.rotation_velocity += rotation_mod / 2.0 * delta_time;

    proto_pipeline.set_player(
        old_position,
        state.player.position,
        f64::from(old_rotation) as f32,
        f64::from(state.player.rotation) as f32,
        match state.player_control {
            events::PlayerControl {
                big_forward: true, ..
            } => rp::BoosterState::TurboBooster,
            events::PlayerControl {
                main_forward: true, ..
            } => rp::BoosterState::MainBooster,
            _ => rp::BoosterState::None,
        },
    );
    proto_pipeline.set_camera(state.camera_position, state.camera_position);
}

pub fn start_physics<const A: usize, const P: usize>(
    render_pipeline: &sync::Mutex<rp::ReadRenderInfo<A, P>>,
    physics_events: mpsc::Receiver<events::PhysicsEvent>,
) {
    // Allocate any initial objects needed for the game to work. For now, that will just be the
    // player. In the future, it'll also include planets or large asteroids with gravity.
    // Should this function handle events of some kind? Or is that the responsibility of the ticking
    // function?
    // Run the ticking function in a loop forever.
    let mut ticker = TickState::<1337, 67, _, _>::new(&render_pipeline, physics_events);

    // Run the ticking function, 240 per second if doable.

    let mut delta_time: f64 = 1.0 / 240.0;
    const TICK_RATE: time::Duration = time::Duration::from_micros(1000000 / 240);

    loop {
        let before_tick = time::Instant::now();
        if let TickSignal::StopMe = ticker.tick(delta_time) {
            break;
        }

        let elapsed = before_tick.elapsed();
        if elapsed > TICK_RATE {
            delta_time = elapsed.as_secs_f64();
            continue;
        }

        delta_time = TICK_RATE.as_secs_f64();
        thread::sleep(TICK_RATE - elapsed);
    }

    println!("Stopping physics.");
}

// TODO: Add unit tests for the allocator. HA!
