use crate::game_vector;

#[derive(Debug, Copy, Clone)]
pub struct Player {
    pub position: game_vector::GameVector<i64>,
    pub rotation: game_vector::Radians,
    pub velocity: game_vector::GameVector<f64>,
    pub rotation_velocity: game_vector::UncappedRadians,
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
            position: game_vector::GameVector::new(0, 0),
            rotation: game_vector::Radians::from(0.0),
            velocity: game_vector::GameVector::new(0.0, 0.0),
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
    position: game_vector::GameVector<i64>,
    rotation: f32,
    velocity: game_vector::GameVector<i32>,
    health_current: u16,
    health_max: u16,
    scale: f32,
    points_count: u8,
    points: [f32; 8],
}

#[derive(Default, Debug, Copy, Clone)]
pub struct Projectile {
    position: game_vector::GameVector<i64>,
    velocity: game_vector::GameVector<i32>,
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

