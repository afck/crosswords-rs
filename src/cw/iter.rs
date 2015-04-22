use cw::{BLOCK, Crosswords, Dir, Range};
use point::Point;

pub struct RangeIter<'a> {
    range: Range,
    cw: &'a Crosswords,
}

impl<'a> RangeIter<'a> {
    pub fn new(range: Range, cw: &'a Crosswords) -> RangeIter<'a> {
        RangeIter { range: range, cw: cw }
    }
}

impl<'a> Iterator for RangeIter<'a> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        if self.range.len == 0 {
            None
        } else {
            let point = self.range.point;
            self.range.point = self.range.point + self.range.dir.point();
            self.range.len -= 1;
            self.cw.get_char(point)
        }
    }
}

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
            let len = match self.ri_type {
                RangesIterType::Word => self.cw.get_word_len_at(self.point, self.dir),
                RangesIterType::Free => self.cw.get_free_range_at(self.point, self.dir),
            };
            if len > 1 {
                let range = Range { point: self.point, dir: self.dir, len: len };
                self.advance(len); // TODO: If RIT::Free, advance len + 2?
                return Some(range);
            }
            self.advance(1);
        }
        None
    }
}

enum PrintIterType {
    Solution,
    Puzzle,
}

pub enum PrintItem {
    VertBorder(bool),
    HorizBorder(bool),
    Cross(bool),
    Block,
    Character(char),
    Number(u32),
    LineBreak,
}

pub struct PrintIter<'a> {
    point: Point,
    between_lines: bool,
    between_chars: bool,
    cw: &'a Crosswords,
    pi_type: PrintIterType,
}

impl<'a> PrintIter<'a> {
    fn new(cw: &'a Crosswords, pi_type: PrintIterType) -> Self {
        PrintIter {
            point: Point::new(-1, -1),
            between_lines: true,
            between_chars: true,
            cw: cw,
            pi_type: pi_type,
        }
    }

    pub fn new_solution(cw: &'a Crosswords) -> Self { PrintIter::new(cw, PrintIterType::Solution) }
}

impl<'a> Iterator for PrintIter<'a> {
    type Item = PrintItem;
    fn next(&mut self) -> Option<PrintItem> {
        if self.point.y >= self.cw.height as i32 {
            return None;
        }
        let result;
        if self.point.x >= self.cw.width as i32 {
            result = PrintItem::LineBreak;
            self.point.x = -1;
            if self.between_lines {
                self.point.y += 1;
            }
            self.between_chars = true;
            self.between_lines = !self.between_lines;
        } else if self.between_chars {
            if self.between_lines {
                let mut count = 0;
                if self.cw.get_border(self.point, Dir::Down) { count += 1 }
                if self.cw.get_border(self.point, Dir::Right) { count += 1 }
                if self.cw.get_border(self.point + Point::new(1, 0), Dir::Down) { count += 1 }
                if self.cw.get_border(self.point + Point::new(0, 1), Dir::Right) { count += 1 }
                result = PrintItem::Cross(count > 1);
            } else {
                result = PrintItem::VertBorder(self.cw.get_border(self.point, Dir::Right));
            }
            self.point.x += 1;
            self.between_chars = false;
        } else {
            if self.between_lines {
                result = PrintItem::HorizBorder(self.cw.get_border(self.point, Dir::Down));
            } else {
                result = match self.cw.get_char(self.point).unwrap() {
                    BLOCK => PrintItem::Block,
                    c => PrintItem::Character(c),
                };
            }
            self.between_chars = true;
        }
        Some(result)
    }
}

