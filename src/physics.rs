use crate::game_vector::*; // TODO: This import is stupid and ugly and I need to organize my files
                           // better

pub fn update_velocity(velocity: &mut GameVector<i32>, apply: GameVector<i32>) {
    *velocity += apply;
}

