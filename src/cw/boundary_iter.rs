use cw::{BLOCK, Crosswords, Dir, Point, Range};

fn turn(point: Point) -> Point {
    Point {
        x: -point.y,
        y: point.x,
    }
}

/// An iterator of all pairs of empty and filled cells at the boundary of the given cluster.
/// It can be given an additional range that it will consider filled with letters.
pub struct BoundaryIter<'a> {
    last: (Point, Point),
    prev: Option<(Point, Point)>,
    filled_range: Option<Range>,
    cw: &'a Crosswords,
}

impl<'a> BoundaryIter<'a> {
    pub fn new(point: Point, filled_range: Option<Range>, cw: &'a Crosswords) -> BoundaryIter<'a> {
        (cw.get_char(point) == Some(BLOCK) && filled_range.iter().all(|r| !r.contains(point))) ||
        panic!("BoundaryIter must start with an empty cell.");
        let dp = Dir::Right.point();
        let mut p1 = point;
        while cw.get_char(p1) == Some(BLOCK) && filled_range.iter().all(|r| !r.contains(p1)) {
            p1.x += 1;
        }
        BoundaryIter {
            last: (p1 - dp, p1),
            prev: None,
            filled_range: filled_range,
            cw: cw,
        }
    }

    #[inline]
    fn is_free(&self, point: Point) -> bool {
        self.cw.get_char(point) == Some(BLOCK) &&
        self.filled_range.iter().all(|r| !r.contains(point))
    }

    fn advance(&mut self) -> bool {
        if self.prev == Some(self.last) {
            return false;
        }
        let (p0, p1) = self.prev.unwrap_or(self.last);
        let dp = p1 - p0;
        let odp = turn(dp);
        self.prev = Some(if !self.is_free(p0 + odp) {
            (p0, p0 + odp)
        } else if !self.is_free(p1 + odp) {
            (p0 + odp, p1 + odp)
        } else {
            (p1 + odp, p1)
        });
        true
    }
}

impl<'a> Iterator for BoundaryIter<'a> {
    type Item = (Point, Point);

    fn next(&mut self) -> Option<(Point, Point)> {
        if !self.advance() {
            return None;
        }
        while !self.cw.contains(self.prev.unwrap().1) {
            if !self.advance() {
                return None;
            }
        }
        Some(self.prev.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use cw::{Crosswords, Dir, Point, Range};
    use test_util::str_to_cvec;

    fn range_r(x: i32, y: i32) -> Option<(Point, Point)> {
        Some((Point::new(x, y), Point::new(x + 1, y)))
    }

    fn range_d(x: i32, y: i32) -> Option<(Point, Point)> {
        Some((Point::new(x, y), Point::new(x, y + 1)))
    }

    #[test]
    fn test() {
        // Create the following grid, tell the iterator to consider the + as letters.
        // ####A
        // ##++C
        // AB###
        let mut cw = Crosswords::new(5, 3);
        cw.try_word(Point::new(0, 2), Dir::Right, &str_to_cvec("AB"));
        cw.try_word(Point::new(4, 0), Dir::Down, &str_to_cvec("AC"));
        let range = Range {
            point: Point::new(2, 1),
            dir: Dir::Right,
            len: 2,
        };
        let mut iter = cw.get_boundary_iter_for(Point::new(0, 0), Some(range));
        assert_eq!(range_d(3, 0), iter.next());
        assert_eq!(range_d(2, 0), iter.next());
        assert_eq!(range_r(1, 1), iter.next());
        assert_eq!(range_d(1, 1), iter.next());
        assert_eq!(range_d(0, 1), iter.next());
        assert_eq!(range_r(3, 0), iter.next());
        assert_eq!(None, iter.next());
    }
}
