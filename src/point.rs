use noisy_float::{
    prelude::Float,
    types::{r32, R32},
};
use std::ops;

/// A [`Point`] in the [`QuadTree`] graph.
///
/// This contains only real values, taken from the [`noisy_float`] library. When compiled these values will panic when they are set to `NaN` or `Infinity`.
///
/// [`QuadTree`]: struct.QuadTree.html
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    ///
    pub x: R32,
    ///
    pub y: R32,
}

impl Point {
    /// Create a point at `0, 0`
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            x: R32::unchecked_new(0.0),
            y: R32::unchecked_new(0.0),
        }
    }
    /// Create a point at `x, y`. In debug mode this will panic when `x` or `y` are `NaN` or `Infinity`
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: r32(x),
            y: r32(y),
        }
    }

    /// Create a point at `x, y` with `noisy_float`'s [`R32`]
    #[must_use]
    pub const fn new_noisy_float(x: R32, y: R32) -> Self {
        Self { x, y }
    }

    /// Get the squared distance to another point
    #[must_use]
    pub fn distance_squared_to(&self, other: Point) -> R32 {
        let result =
            (self.x.raw() - other.x.raw()).powf(2.0) + (self.y.raw() - other.y.raw()).powf(2.0);
        if let Some(result) = R32::try_new(result) {
            result
        } else {
            R32::max_value()
        }
    }
}

impl ops::Add<R32> for Point {
    type Output = Point;

    fn add(self, rhs: R32) -> Self {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}
impl ops::Sub<R32> for Point {
    type Output = Point;

    fn sub(self, rhs: R32) -> Self {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}
impl ops::Neg for Point {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Rect {
    top: R32,
    left: R32,
    bottom: R32,
    right: R32,
}

impl Rect {
    pub const fn new(top_left: Point, bottom_right: Point) -> Self {
        Self {
            top: top_left.y,
            left: top_left.x,
            bottom: bottom_right.y,
            right: bottom_right.x,
        }
    }

    pub fn middle(self) -> Point {
        Point::new_noisy_float(
            (self.left + self.right) / 2.0,
            (self.top + self.bottom) / 2.0,
        )
    }

    pub fn contains(self, point: Point) -> bool {
        !(self.left > point.x
            || self.right < point.x
            || self.top > point.y
            || self.bottom < point.y)
    }

    pub fn get_child_at(self, quadrant: Quadrant) -> Rect {
        let middle = self.middle();
        match quadrant {
            Quadrant::TopLeft => Rect {
                top: self.top,
                left: self.left,
                right: middle.x,
                bottom: middle.y,
            },
            Quadrant::BottomLeft => Rect {
                top: middle.y,
                left: self.left,
                right: middle.x,
                bottom: self.bottom,
            },
            Quadrant::TopRight => Rect {
                top: self.top,
                left: middle.x,
                right: self.right,
                bottom: middle.y,
            },
            Quadrant::BottomRight => Rect {
                top: middle.y,
                left: middle.x,
                right: self.right,
                bottom: self.bottom,
            },
        }
    }

    pub fn get_quadrant(self, point: Point) -> (Rect, Quadrant) {
        let middle = self.middle();
        let quadrant = if point.x < middle.x {
            if point.y < middle.y {
                Quadrant::TopLeft
            } else {
                Quadrant::BottomLeft
            }
        } else if point.y < middle.y {
            Quadrant::TopRight
        } else {
            Quadrant::BottomRight
        };
        (self.get_child_at(quadrant), quadrant)
    }

    pub(crate) fn get_index_rect(self, index: crate::index::Index) -> Rect {
        index.iter_from_root().fold(self, Rect::get_child_at)
    }

    pub(crate) fn intersects(&self, rect: Rect) -> bool {
        !(self.left < rect.right
            && self.right > rect.left
            && self.top > rect.bottom
            && self.bottom < rect.top)
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Quadrant {
    TopLeft = 0b00,
    TopRight = 0b01,
    BottomLeft = 0b10,
    BottomRight = 0b11,
}

impl Quadrant {
    pub const fn all() -> [Quadrant; 4] {
        [
            Self::TopLeft,
            Self::TopRight,
            Self::BottomLeft,
            Self::BottomRight,
        ]
    }

    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0b00 => Self::TopLeft,
            0b01 => Self::TopRight,
            0b10 => Self::BottomLeft,
            0b11 => Self::BottomRight,
            _ => unreachable!(),
        }
    }
}
