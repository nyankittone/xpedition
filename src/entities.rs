use crate::game_vector::GameVector;

#[derive(Debug, Copy, Clone)]
pub struct Player {
    pub position: GameVector<i64>,
    pub rotation: f32,
    pub velocity: GameVector<i32>,
    pub rotation_velocity: f64,
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
            rotation_velocity: 0.0,
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

// Is this enum REALLY useful??? Like honestly
#[repr(u8)]
pub enum EntityType {
    Player,
    Asteroid,
    Projectile,
}

