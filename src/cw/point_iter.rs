use cw::{Dir, Point};

/// An iterator over a range of points.
pub struct PointIter {
    point: Point,
    dp: Point,
    len: usize,
}

impl PointIter {
    /// Returns an iterator starting at the given point and proceeding in the given direction,
    /// which will be exhausted after it has returned `len` points.
    pub fn new(point: Point, dir: Dir, len: usize) -> PointIter {
        PointIter { point: point, dp: dir.point(), len: len }
    }
}

impl Iterator for PointIter {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.len == 0 {
            None
        } else {
            let point = self.point;
            self.point = self.point + self.dp;
            self.len -= 1;
            Some(point)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (self.len, Some(self.len)) }
}

