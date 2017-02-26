use cw::{Crosswords, Range};
use cw::PointIter;

/// An iterator over the characters in a particular range in a crosswords grid.
pub struct RangeIter<'a> {
    pi: PointIter,
    cw: &'a Crosswords,
}

impl<'a> RangeIter<'a> {
    /// Creates an iterator over the characters in the given range.
    pub fn new(range: Range, cw: &'a Crosswords) -> RangeIter<'a> {
        RangeIter {
            pi: range.points(),
            cw: cw,
        }
    }
}

impl<'a> Iterator for RangeIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.pi.next().and_then(|point| self.cw.get_char(point))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.pi.size_hint()
    }
}
