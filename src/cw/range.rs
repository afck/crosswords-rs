use cw::{Dir, Point, PointIter};

/// A horizontal or vertical range of consecutive points in a grid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Range {
    /// The first point in the range.
    pub point: Point,
    /// The direction.
    pub dir: Dir,
    /// The number of points in the range.
    pub len: usize,
}

impl Range {
    /// Returns a (possibly empty) range containing all points satisfying the given predicate `f`,
    /// starting from `point` and proceeding in the given direction until the predicate is false.
    pub fn cells_with<F>(point: Point, dir: Dir, mut f: F) -> Range
        where F: FnMut(Point) -> bool
    {
        let dp = dir.point();
        let mut p = point;
        let mut len = 0;
        while f(p) {
            len += 1;
            p = p + dp;
        }
        Range {
            point: point,
            dir: dir,
            len: len,
        }
    }

    /// Returns an iterator over the points in the range.
    pub fn points(&self) -> PointIter {
        PointIter::new(self.point, self.dir, self.len)
    }

    /// Returns a range containing the given two points.
    /// Assumes (but does not check!) that the points are adjacent.
    pub fn with_points(point0: Point, point1: Point) -> Range {
        Range {
            point: if point0.x < point1.x || point0.y < point1.y {
                point0
            } else {
                point1
            },
            dir: if point0.x == point1.x {
                Dir::Down
            } else {
                Dir::Right
            },
            len: 2,
        }
    }

    /// Returns `true` if the point belongs to the range.
    pub fn contains(&self, point: Point) -> bool {
        match self.dir {
            Dir::Right => {
                self.point.y == point.y && self.point.x <= point.x &&
                point.x < self.point.x + self.len as i32
            }
            Dir::Down => {
                self.point.x == point.x && self.point.y <= point.y &&
                point.y < self.point.y + self.len as i32
            }
        }
    }

    /// Returns `true` if the ranges have at least one point in common.
    pub fn intersects(&self, other: &Range) -> bool {
        let (s0, s1) = (self.point, self.point + self.dir.point() * (self.len - 1));
        let (o0, o1) = (other.point, other.point + other.dir.point() * (other.len - 1));
        s0.x <= o1.x && o0.x <= s1.x && s0.y <= o1.y && o0.y <= s1.y
    }

    /// Returns `true` if the ranges are adjacent to each other, i. e. they are disjoint and their
    /// union would be a range again.
    pub fn is_adjacent_to(&self, other: &Range) -> bool {
        self.dir == other.dir || return false;
        let dp = self.dir.point();
        self.point + dp * self.len == other.point || other.point + dp * other.len == self.point
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cw::{Dir, Point};

    #[test]
    fn test_intersects() {
        let v_range0 = Range {
            point: Point::new(1, 1),
            len: 3,
            dir: Dir::Right,
        };
        let v_range1 = Range {
            point: Point::new(1, 2),
            len: 3,
            dir: Dir::Right,
        };
        let h_range0 = Range {
            point: Point::new(0, 0),
            len: 5,
            dir: Dir::Down,
        };
        let h_range1 = Range {
            point: Point::new(1, 0),
            len: 2,
            dir: Dir::Down,
        };
        let h_range2 = Range {
            point: Point::new(2, 2),
            len: 3,
            dir: Dir::Down,
        };
        assert_eq!(false, v_range0.intersects(&v_range1));
        assert_eq!(false, v_range0.intersects(&h_range0));
        assert_eq!(true, v_range0.intersects(&h_range1));
        assert_eq!(false, v_range0.intersects(&h_range2));
        assert_eq!(false, v_range1.intersects(&h_range0));
        assert_eq!(false, v_range1.intersects(&h_range1));
        assert_eq!(true, v_range1.intersects(&h_range2));
    }

    #[test]
    fn test_is_adjacent_to() {
        let v_range0 = Range {
            point: Point::new(3, 1),
            len: 3,
            dir: Dir::Right,
        };
        let v_range1 = Range {
            point: Point::new(1, 1),
            len: 2,
            dir: Dir::Right,
        };
        let v_range2 = Range {
            point: Point::new(1, 2),
            len: 2,
            dir: Dir::Right,
        };
        let v_range3 = Range {
            point: Point::new(0, 1),
            len: 2,
            dir: Dir::Right,
        };
        let v_range4 = Range {
            point: Point::new(2, 1),
            len: 2,
            dir: Dir::Right,
        };
        let h_range0 = Range {
            point: Point::new(3, 0),
            len: 2,
            dir: Dir::Down,
        };
        assert_eq!(true, v_range0.is_adjacent_to(&v_range1));
        assert_eq!(true, v_range1.is_adjacent_to(&v_range0));
        assert_eq!(false, v_range0.is_adjacent_to(&v_range2));
        assert_eq!(false, v_range2.is_adjacent_to(&v_range0));
        assert_eq!(false, v_range0.is_adjacent_to(&v_range3));
        assert_eq!(false, v_range0.is_adjacent_to(&v_range4));
        assert_eq!(false, v_range0.is_adjacent_to(&h_range0));
    }
}
