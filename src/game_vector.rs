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
        Self { x, y }
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

// time to re-implement arithmetic for a float!!!!!! wowie zowie!
macro_rules! radians_overload {
    ($trait:ident, $fn:ident, $op:tt) => {
        impl $trait<f64> for Radians {
        type Output = Self;

        fn $fn(self, rhs: f64) -> Self::Output {
            Self((self.0 $op rhs) % f64::consts::TAU)
        }
    }

    impl $trait for Radians {
        type Output = Self;

        fn $fn(self, rhs: Self) -> Self::Output {
            Self((self.0 $op rhs.0) % f64::consts::TAU)
        }
    }

    impl $trait<Radians> for f64 {
        type Output = f64;

        fn $fn(self, rhs: Radians) -> Self::Output {
            self $op rhs.0
        }
    }
    };
}

macro_rules! radians_assign_overload {
    ($trait:ident, $fn:ident, $op:tt) => {
        impl $trait<f64> for Radians {
            fn $fn(&mut self, rhs: f64) {
                self.0 $op rhs;
                self.0 %= f64::consts::TAU;
            }
        }

        impl $trait for Radians {
            fn $fn(&mut self, rhs: Self) {
                self.0 $op rhs.0;
                self.0 %= f64::consts::TAU;
            }
        }

        impl $trait<Radians> for f64 {
            fn $fn(&mut self, rhs: Radians) {
                *self $op rhs.0;
            }
        }
    };
}

radians_overload!(Add, add, +);
radians_overload!(Sub, sub, -);
radians_overload!(Mul, mul, *);
radians_overload!(Div, div, /);
radians_assign_overload!(AddAssign, add_assign, +=);
radians_assign_overload!(SubAssign, sub_assign, -=);
radians_assign_overload!(MulAssign, mul_assign, *=);
radians_assign_overload!(DivAssign, div_assign, /=);
// radians_assign_overload!(RemAssign, rem_assign, %=);

impl Rem<f64> for Radians {
    type Output = Self;

    fn rem(self, rhs: f64) -> Self::Output {
        Self(self.0 % rhs)
    }
}

impl Rem for Radians {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl Rem<Radians> for f64 {
    type Output = f64;

    fn rem(self, rhs: Radians) -> Self::Output {
        self % rhs.0
    }
}

impl RemAssign<f64> for Radians {
    fn rem_assign(&mut self, rhs: f64) {
        self.0 %= rhs;
    }
}

impl RemAssign for Radians {
    fn rem_assign(&mut self, rhs: Self) {
        self.0 %= rhs.0;
    }
}

impl RemAssign<Radians> for f64 {
    fn rem_assign(&mut self, rhs: Radians) {
        *self %= rhs.0;
    }
}

impl Neg for Radians {
    type Output = Radians;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Radians {
    pub const fn from_degrees(input: f64) -> Self {
        Self((input * f64::consts::PI / 180.0) % f64::consts::TAU)
    }

    pub const fn to_degrees(self) -> f64 {
        self.0 * 180.0 / f64::consts::PI
    }
}
