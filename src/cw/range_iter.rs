use cw::{Crosswords, Range};
use cw::PointIter;

pub struct RangeIter<'a> {
    pi: PointIter,
    cw: &'a Crosswords,
}

impl<'a> RangeIter<'a> {
    pub fn new(range: Range, cw: &'a Crosswords) -> RangeIter<'a> {
        RangeIter { pi: range.points(), cw: cw }
    }
}

impl<'a> Iterator for RangeIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.pi.next().and_then(|point| self.cw.get_char(point))
    }

    fn size_hint(&self) -> (usize, Option<usize>) { self.pi.size_hint() }
}

