//! This contains entity definitions used in the rendering pipeline, rather than versions actually
//! used in the game's state.

use crate::entities;
use crate::game_vector::*;

#[repr(u16)]
#[derive(Copy, Clone)]
pub enum LayerID {
    Player,
    Projectile,
    Asteriod,
    Other,
}

#[derive(Copy, Clone)]
pub struct RenderEntity {
    layer: LayerID,
    start_position: GameVector<u64>,
    end_position: GameVector<u64>,
    start_rotation: f64, // Rotation fields are not always used, e.g. projectiles don't really need
                         // a rotation to be drawn.
    end_rotation: f64,
    sprite_id: u32, // This could change to be something more smart, like a function pointer, later.
}

impl RenderEntity {
    pub fn new_rotatable(
        start_position: GameVector<u64>,
        end_position: GameVector<u64>,
        start_rotation: f64,
        end_rotation: f64,
        layer: LayerID,
        sprite_id: u32,
    ) -> Self {
        Self {
            layer,
            start_position,
            end_position,
            start_rotation,
            end_rotation,
            sprite_id,
        }
    }

    pub fn new_unrotatable(
        start_position: GameVector<u64>,
        end_position: GameVector<u64>,
        layer: LayerID,
        sprite_id: u32,
    )  -> Self {
        Self {
            layer,
            start_position,
            end_position,
            start_rotation: 0.0,
            end_rotation: 0.0,
            sprite_id,
        }
    }
}
