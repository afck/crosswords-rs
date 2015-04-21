use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }

    #[inline]
    pub fn coord(&self, w: usize, h: usize) -> Option<usize> {
        if self.x < 0 || self.y < 0 || self.x as usize >= w || self.y as usize >= h {
            None
        } else {
            Some((self.x as usize) + w * (self.y as usize))
        }
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<i32> for Point {
    type Output = Point;

    fn mul(self, rhs: i32) -> Point {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<usize> for Point {
    type Output = Point;

    fn mul(self, rhs: usize) -> Point {
        Point {
            x: self.x * (rhs as i32),
            y: self.y * (rhs as i32),
        }
    }
}
