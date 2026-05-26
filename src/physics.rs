use crate::math_types::*; // TODO: This import is stupid and ugly and I need to organize my files
                           // better

pub fn update_velocity(velocity: &mut GameVector<i32>, apply: GameVector<i32>) {
    *velocity += apply;

    // I need to decelerate the figure in its direction of travel. Normalize a vector, then get the
    // sum of a version of that's divided by some constant, plus that normalized vector multiplied
    // by some small factor so that the deceleration grows as the speed increases. I need to make
    // sure I don't have so much deceleration that it feels like the figure travels through
    // molassas.
}

