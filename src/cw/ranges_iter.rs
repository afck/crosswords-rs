use cw::{Crosswords, Dir, Range, Point};

pub struct RangesIter<'a> {
    point: Point,
    dir: Dir,
    ended: bool,
    cw: &'a Crosswords,
}

impl<'a> RangesIter<'a> {
    pub fn new(cw: &'a Crosswords) -> RangesIter<'a> {
        RangesIter {
            point: Point::new(0, 0),
            dir: Dir::Right,
            ended: false,
            cw: cw,
        }
    }

    fn advance(&mut self, len: usize) {
        if self.ended { return; }
        match self.dir {
            Dir::Right => {
                self.point.x += len as i32;
                if self.point.x >= self.cw.width as i32 {
                    self.point.y += 1;
                    self.point.x = 0;
                    if self.point.y >= self.cw.height as i32 {
                        self.point.y = 0;
                        self.dir = Dir::Down;
                    }
                }
            },
            Dir::Down => {
                self.point.y += len as i32;
                if self.point.y >= self.cw.height as i32 {
                    self.point.x += 1;
                    self.point.y = 0;
                    if self.point.x >= self.cw.width as i32 {
                        self.ended = true;
                    }
                }
            }
        }
    }
}

impl<'a> Iterator for RangesIter<'a> {
    type Item = Range;
    fn next(&mut self) -> Option<Range> {
        while !self.ended {
            let range = self.cw.get_word_range_at(self.point, self.dir);
            if range.len > 1 {
                self.advance(range.len); // TODO: If RIT::Free, advance len + 2?
                return Some(range);
            }
            self.advance(1);
        }
        None
    }
}

