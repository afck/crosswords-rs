use cw::{Crosswords, Dir, Range, Point};

enum RangesIterType {
    Word, Free,
}

pub struct RangesIter<'a> {
    point: Point,
    dir: Dir,
    ended: bool,
    cw: &'a Crosswords,
    ri_type: RangesIterType,
}

impl<'a> RangesIter<'a> {
    fn new(cw: &'a Crosswords, ri_type: RangesIterType) -> Self {
        RangesIter {
            point: Point::new(0, 0),
            dir: Dir::Right,
            ended: false,
            cw: cw,
            ri_type: ri_type,
        }
    }

    pub fn new_free(cw: &'a Crosswords) -> Self { RangesIter::new(cw, RangesIterType::Free) }

    pub fn new_words(cw: &'a Crosswords) -> Self { RangesIter::new(cw, RangesIterType::Word) }

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
            let range = match self.ri_type {
                RangesIterType::Word => self.cw.get_word_range_at(self.point, self.dir),
                RangesIterType::Free => self.cw.get_free_range_at(self.point, self.dir),
            };
            if range.len > 1 {
                self.advance(range.len); // TODO: If RIT::Free, advance len + 2?
                return Some(range);
            }
            self.advance(1);
        }
        None
    }
}

