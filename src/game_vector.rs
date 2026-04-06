use std::ops::*;

// things to overload:
// add
// subtract
// multiply
// divide
// +=
// -=
// *-
// /=
// We also would want overloads for single scalar values when appropriate.
pub struct GameVector<T> {
    pub x: T,
    pub y: T,
}

impl<T> GameVector<T> {
    pub fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
        }
    }
}

// now for all the traits...
macro_rules! two_overload {
    ($trait:ident, $fn:ident, $op:tt) => {impl<T: $trait<Output = T>> $trait for GameVector<T> {
        type Output = Self;

        fn $fn(self, rhs: Self) -> Self::Output {
            Self {
                x: self.x $op rhs.x,
                y: self.y $op rhs.y,
            }
        }
    }
    impl<T: $trait<Output = T> + Clone> $trait<T> for GameVector<T> {
        type Output = Self;

        fn $fn(self, rhs: T) -> Self::Output {
            Self {
                x: self.x $op rhs.clone(),
                y: self.y $op rhs.clone(),
            }
        }
    }
    };
}

two_overload!(Add, add, +);
two_overload!(Sub, sub, -);
two_overload!(Mul, mul, *);
two_overload!(Div, div, /); // TODO: is it a good idea to have a special case just for division to
                            // support dividing by 0?

