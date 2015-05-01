use cw::{BLOCK, Crosswords, Dir, Point, Range};

fn turn(point: Point) -> Point {
    Point {
        x: -point.y,
        y: point.x,
    }
}

fn to_range((point0, point1): (Point, Point)) -> Range {
    Range {
        point: if point0.x < point1.x || point0.y < point1.y { point0 } else { point1 },
        dir: if point0.x == point1.x { Dir::Down } else { Dir::Right },
        len: 2,
    }
}

/// An iterator of all length 2 ranges with one empty cell of the given cluster, and one letter.
/// It can be given an additional range that it will consider filled with letters.
pub struct BoundaryIter<'a> {
   last: (Point, Point),
   prev: Option<(Point, Point)>,
   filled_range: Range,
   cw: &'a Crosswords,
}

impl<'a> BoundaryIter<'a> {
    pub fn new(point: Point, filled_range: Range, cw: &'a Crosswords) -> BoundaryIter<'a> {
        let dp = Dir::Right.point();
        let mut p1 = point;
        while cw.get_char(p1) == Some(BLOCK) {
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
        self.cw.get_char(point) == Some(BLOCK) && !self.filled_range.contains(point)
    }

    fn advance(&mut self) -> bool {
        self.prev != Some(self.last) || return false;
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

impl <'a> Iterator for BoundaryIter<'a> {
    type Item = Range;

    fn next(&mut self) -> Option<Range> {
        self.advance() || return None;
        while !self.cw.contains(self.prev.unwrap().1) {
            self.advance() || return None;
        }
        Some(to_range(self.prev.unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use cw::{Crosswords, Dir, Point, Range};

    fn range_r(x: i32, y: i32) -> Option<Range> {
        Some(Range { point: Point::new(x, y), dir: Dir::Right, len: 2 })
    }

    fn range_d(x: i32, y: i32) -> Option<Range> {
        Some(Range { point: Point::new(x, y), dir: Dir::Down, len: 2 })
    }

    #[test]
    fn test() {
        // Create the following grid, tell the iterator to consider the + as letters.
        // ####A
        // ##++C
        // AB###
        let mut cw = Crosswords::new(5, 3);
        cw.try_word(Point::new(0, 2), Dir::Right, &"AB".chars().collect());
        cw.try_word(Point::new(4, 0), Dir::Down, &"AC".chars().collect());
        let mut iter = cw.get_boundary_iter_for(Point::new(0, 0), range_r(2, 1).unwrap());
        assert_eq!(range_d(3, 0), iter.next());
        assert_eq!(range_d(2, 0), iter.next());
        assert_eq!(range_r(1, 1), iter.next());
        assert_eq!(range_d(1, 1), iter.next());
        assert_eq!(range_d(0, 1), iter.next());
        assert_eq!(range_r(3, 0), iter.next());
        assert_eq!(None, iter.next());
    }
}
