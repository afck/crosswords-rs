use cw::{Dir, Point, PointIter};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Range {
    pub point: Point,
    pub dir: Dir,
    pub len: usize,
}

impl Range {
    pub fn cells_with<F>(point: Point, dir: Dir, mut f: F) -> Range where F: FnMut(Point) -> bool {
        let dp = dir.point();
        let mut p = point;
        let mut len = 0;
        while f(p) {
            len += 1;
            p = p + dp;
        }
        Range { point: point, dir: dir, len: len }
    }

    pub fn points(&self) -> PointIter {
        PointIter::new(self.point, self.dir, self.len)
    }

    pub fn intersects(&self, other: &Range) -> bool {
        let (s0, s1) = (self.point, self.point + self.dir.point() * (self.len - 1));
        let (o0, o1) = (other.point, other.point + other.dir.point() * (other.len - 1));
        s0.x <= o1.x && o0.x <= s1.x && s0.y <= o1.y && o0.y <= s1.y
    }
}

