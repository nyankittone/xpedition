use core::f64;
use std::{fmt::Display, ops::*};
use x11rb::protocol::xproto::Point;

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
#[derive(Copy, Clone, Debug, Default)]
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

impl From<GameVector<i32>> for Point {
    fn from(value: GameVector<i32>) -> Self {
        Self {
            x: value.x as i16,
            y: value.y as i16,
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

macro_rules! two_overload_assign {
    ($trait:ident, $dep_trait:ident, $fn:ident, $op:tt) => {
    impl<T: $dep_trait<Output = T> + Clone> $trait for GameVector<T> {
        fn $fn(&mut self, rhs: Self) {
            let x = self.x.clone() $op rhs.x;
            let y = self.y.clone() $op rhs.y;
            *self = Self {
                x,
                y,
            }
        }
    }
    impl<T: $dep_trait<Output = T> + Clone> $trait<T> for GameVector<T> {
        fn $fn(&mut self, rhs: T) {
            let x = self.x.clone() $op rhs.clone();
            let y = self.y.clone() $op rhs.clone();
            *self = Self {
                x,
                y,
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
two_overload_assign!(AddAssign, Add, add_assign, +);
two_overload_assign!(SubAssign, Sub, sub_assign, -);
two_overload_assign!(MulAssign, Mul, mul_assign, *);
two_overload_assign!(DivAssign, Div, div_assign, /);

// Data type for storing Radians in a more intelligent way
#[derive(Copy, Clone)]
pub struct Radians(f64);

impl From<f64> for Radians {
    fn from(value: f64) -> Self {
        Radians(value)
    }
}

impl From<Radians> for f64 {
    fn from(value: Radians) -> Self {
        value.0
    }
}

impl Deref for Radians {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Radians {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Radians {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl Radians {
    pub const fn from_degrees(input: f64) -> Self {
        Self(input * f64::consts::PI / 180.0)
    }

    pub const fn to_degrees_float(radians: f64) -> f64 {
        radians * 180.0 / f64::consts::PI
    }

    pub const fn to_degrees(self) -> f64 {
        self.0 * 180.0 / f64::consts::PI
    }
}

