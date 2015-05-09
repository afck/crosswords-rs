mod boundary_iter;
mod point_iter;
mod print_iter;
mod range_iter;
mod ranges_iter;
mod point;
mod range;

pub use cw::point_iter::PointIter;
pub use cw::print_iter::PrintItem;
pub use cw::range::Range;
pub use cw::point::Point;

use std::collections::HashSet;
use std::iter::{repeat, Zip};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::slice;
use cw::boundary_iter::BoundaryIter;
use cw::print_iter::PrintIter;
use cw::range_iter::RangeIter;
use cw::ranges_iter::RangesIter;

pub type CVec = Vec<char>;

pub const BLOCK: char = '#';

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Dir {
    Right,
    Down,
}

impl Dir {
    pub fn other(&self) -> Dir {
        match *self {
            Dir::Right => Dir::Down,
            Dir::Down => Dir::Right,
        }
    }

    pub fn point(&self) -> Point {
        match *self {
            Dir::Right => Point::new(1, 0),
            Dir::Down => Point::new(0, 1),
        }
    }
}

fn word_iter<'a>(word: &'a CVec, point: Point, dir: Dir)
        -> Zip<slice::Iter<'a, char>, PointIter> {
    word.iter().zip(PointIter::new(point, dir, word.len()))
}

/// A crosswords grid that keeps track of the words it contains and doesn't allow duplicates.
#[derive(Clone)]
pub struct Crosswords {
    width: usize,
    height: usize,
    chars: CVec,
    right_border: Vec<bool>,
    down_border: Vec<bool>,
    words: HashSet<CVec>,
}

impl Crosswords {
    pub fn new(width: usize, height: usize) -> Crosswords {
        Crosswords {
            width: width,
            height: height,
            chars: repeat(BLOCK).take(width * height).collect(),
            right_border: repeat(true).take((width - 1) * height).collect(),
            down_border: repeat(true).take(width * (height - 1)).collect(),
            words: HashSet::new(),
        }
    }

    #[inline]
    pub fn get_width(&self) -> usize { self.width }

    #[inline]
    pub fn get_height(&self) -> usize { self.height }

    pub fn get_words<'a>(&'a self) -> &'a HashSet<CVec> { &self.words }

    #[inline]
    pub fn get_border(&self, point: Point, dir: Dir) -> bool {
        match dir {
            Dir::Right => match point.coord(self.width - 1, self.height) {
                None => true,
                Some(p) => self.right_border[p],
            },
            Dir::Down => match point.coord(self.width, self.height - 1) {
                None => true,
                Some(p) => self.down_border[p],
            }
        }
    }

    #[inline]
    pub fn both_borders(&self, point: Point, dir: Dir) -> bool {
        self.get_border(point, dir) && self.get_border(point - dir.point(), dir)
    }

    #[inline]
    fn set_border(&mut self, point: Point, dir: Dir, value: bool) -> bool {
        match dir {
            Dir::Right => match point.coord(self.width - 1, self.height) {
                None => if value { true } else { unreachable!() },
                Some(p) => {
                    let existing = self.right_border[p];
                    self.right_border[p] = value;
                    existing
                },
            },
            Dir::Down => match point.coord(self.width, self.height - 1) {
                None => if value { true} else { unreachable!() },
                Some(p) => {
                    let existing = self.down_border[p];
                    self.down_border[p] = value;
                    existing
                }
            }
        }
    }

    #[inline]
    fn is_char_allowed(&self, point: Point, c: char) -> bool {
        match point.coord(self.width, self.height) {
            None => false,
            Some(p) => {
                let existing = self.chars[p];
                c == existing || existing == BLOCK
            }
        }
    }

    #[inline]
    pub fn get_char(&self, point: Point) -> Option<char> {
        point.coord(self.width, self.height).and_then(|p| self.chars.get(p).cloned())
    }

    pub fn chars<'a>(&'a self, range: Range) -> RangeIter<'a> { RangeIter::new(range, &self) }

    pub fn chars_at<'a>(&'a self, point: Point, dir: Dir) -> RangeIter<'a> {
        self.chars(self.get_word_range_at(point, dir))
    }

    pub fn word_at(&self, point: Point, dir: Dir) -> CVec {
        self.chars_at(point, dir).collect()
    }

    #[inline]
    fn put_char(&mut self, point: Point, c: char) -> char {
        let p = point.coord(self.width, self.height).unwrap();
        let existing = self.chars[p];
        self.chars[p] = c;
        existing
    }

    pub fn is_word_allowed(&self, point: Point, dir: Dir, word: &CVec) -> bool {
        let dp = dir.point();
        let len = word.len() as i32;
        !self.words.contains(word) && len > 1
            && self.get_border(point - dp, dir)
            && self.get_border(point + dp * (len - 1), dir)
            && word_iter(word, point, dir).all(|(&c, p)| self.is_char_allowed(p, c))
    }

    fn push_word(&mut self, point: Point, dir: Dir, word: &CVec) {
        for (&c, p) in word_iter(word, point, dir) {
            let existing = self.word_at(p, dir);
            self.words.remove(&existing);
            self.put_char(p, c);
        }
        for p in PointIter::new(point, dir, word.len() - 1) {
            self.set_border(p, dir, false);
        }
        self.words.insert(word.clone());
    }

    pub fn pop_word(&mut self, point: Point, dir: Dir) -> CVec {
        let word: Vec<_> = self.word_at(point, dir);
        if word.len() <= 1 {
            return Vec::new();
        }
        let odir = dir.other();
        for p in PointIter::new(point, dir, word.len()) {
            self.set_border(p, dir, true);
            if self.both_borders(p, odir) {
                self.put_char(p, BLOCK);
            }
        }
        self.words.remove(&word);
        word
    }

    pub fn try_word(&mut self, point: Point, dir: Dir, word: &CVec) -> bool {
        if self.is_word_allowed(point, dir, word) {
            self.push_word(point, dir, word);
            true
        } else {
            false
        }
    }

    pub fn free_ranges<'a>(&'a self) -> RangesIter<'a> { RangesIter::new_free(&self) }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= 0 && point.y >= 0 && point.x < self.width as i32 && point.y < self.height as i32
    }

    pub fn is_range_free(&self, range: Range) -> bool {
        let dp = range.dir.point();
        self.contains(range.point) && self.contains(range.point + dp * (range.len - 1))
            && self.get_border(range.point - dp, range.dir)
            && range.points().all(|p| self.get_border(p, range.dir))
    }

    pub fn get_free_range_containing(&self, mut point: Point, dir: Dir) -> Range {
        let dp = dir.point();
        while self.contains(point - dp) && self.get_border(point - dp * 2, dir) {
            point = point - dp;
        }
        self.get_free_range_at(point, dir)
    }

    pub fn get_free_range_at(&self, point: Point, dir: Dir) -> Range {
        let dp = dir.point();
        if !self.contains(point - dp)
                || (self.get_border(point - dp, dir) && !self.get_border(point - dp * 2, dir)) {
            Range::cells_with(point, dir, |p| self.contains(p) && self.get_border(p, dir))
        } else {
            Range { point: point, dir: dir, len: 0 }
        }
    }

    pub fn get_range_after(&self, range: &Range) -> Range {
        let dp = range.dir.point();
        let mut len = 0;
        let mut p = range.point + dp * range.len;
        while self.get_border(p, range.dir) && self.contains(p) {
            len += 1;
            p = p + dp;
        }
        Range {
            point: range.point + dp * range.len,
            dir: range.dir,
            len: len,
        }
    }

    pub fn get_range_before(&self, range: &Range) -> Range {
        let dp = range.dir.point();
        let mut len = 0;
        let mut p = range.point;
        while self.get_border(p - dp * 2, range.dir) && self.contains(p - dp) {
            len += 1;
            p = p - dp;
        }
        Range {
            point: p,
            dir: range.dir,
            len: len,
        }
    }

    #[inline]
    fn is_letter(&self, point: Point) -> bool {
        match self.get_char(point) {
            None | Some(BLOCK) => false,
            Some(_) => true,
        }
    }

    fn is_boundary_point(&self, point: Point) -> bool {
        self.get_char(point) == Some(BLOCK) && (self.is_letter(point + Point::new(1, 0))
            || self.is_letter(point + Point::new(-1, 0))
            || self.is_letter(point + Point::new(0, 1))
            || self.is_letter(point + Point::new(0, -1)))
    }

    pub fn get_smallest_boundary(&self) -> HashSet<(Point, Point)> {
        let mut points = HashSet::new();
        let mut smallest = HashSet::new();
        for x in 0..(self.width as i32) {
            for y in 0..(self.height as i32) {
                let point = Point::new(x, y);
                if !points.contains(&point) && self.is_boundary_point(point) {
                    let boundary: HashSet<_> = self.get_boundary_iter_for(point, None).collect();
                    boundary.len() > 1 || return boundary;
                    if points.is_empty() || boundary.len() < smallest.len() {
                        smallest = boundary.clone();
                    }
                    points.extend(boundary.into_iter().map(|(p0, _)| p0));
                }
            }
        }
        smallest
    }

    pub fn word_ranges<'a>(&'a self) -> RangesIter<'a> { RangesIter::new_words(&self) }

    pub fn get_word_range_containing(&self, mut point: Point, dir: Dir) -> Range {
        let dp = dir.point();
        while !self.get_border(point - dp, dir) {
            point = point - dp;
        }
        self.get_word_range_at(point, dir)
    }

    pub fn get_word_range_at(&self, point: Point, dir: Dir) -> Range {
        let dp = dir.point();
        Range::cells_with(point, dir, |p| (p == point) == self.get_border(p - dp, dir))
    }

    pub fn has_hint_at_dir(&self, point: Point, dir: Dir) -> bool {
        !self.get_border(point, dir) && self.get_border(point - dir.point(), dir)
    }

    pub fn has_hint_at(&self, point: Point) -> bool {
        self.has_hint_at_dir(point, Dir::Right) || self.has_hint_at_dir(point, Dir::Down)
    }

    pub fn is_empty(&self) -> bool { self.words.is_empty() }

    pub fn is_full(&self) -> bool {
        (0..(self.width * self.height)).all(|p| self.chars[p] != BLOCK)
    }

    pub fn count_borders(&self) -> usize {
        self.right_border.iter().chain(self.down_border.iter()).filter(|&&b| b).count()
    }

    pub fn max_border_count(&self) -> usize {
        2 * self.width * self.height - self.width - self.height
    }

    pub fn print_items_solution<'a>(&'a self) -> PrintIter<'a> { PrintIter::new_solution(&self) }

    pub fn print_items_puzzle<'a>(&'a self) -> PrintIter<'a> { PrintIter::new_puzzle(&self) }

    pub fn get_boundary_iter_for<'a>(&'a self, point: Point, range: Option<Range>)
            -> BoundaryIter<'a> {
        BoundaryIter::new(point, range, &self)
    }
}

impl Display for Crosswords {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        {
            let bc = self.count_borders();
            let bt = self.max_border_count();
            let br = 100_f32 * (bc as f32) / (bt as f32);
            try!(formatter.write_fmt(format_args!("{} / {} borders ({}%)\n", bc, bt, br)));
        }
        for item in self.print_items_solution() {
            try!(formatter.write_str(&match item {
                PrintItem::Cross(true) => '\u{00B7}',
                PrintItem::VertBorder(true) => '|',
                PrintItem::HorizBorder(true) => '\u{2014}',
                PrintItem::Cross(false) | PrintItem::VertBorder(false)
                    | PrintItem::HorizBorder(false) => ' ',
                PrintItem::Block => '\u{2588}',
                PrintItem::Character(c) => c,
                PrintItem::Hint(_) => '\'',
                PrintItem::LineBreak => '\n',
            }.to_string()[..]))
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_word() {
        let mut cw = Crosswords::new(6, 2);
        let p00 = Point::new(0, 0);
        let p01 = Point::new(0, 1);
        let p30 = Point::new(3, 0);
        // Words are too long:
        assert_eq!(false, cw.try_word(p00, Dir::Down, &"FOO".chars().collect()));
        assert_eq!(false, cw.try_word(p00, Dir::Right, &"FOOBARBAZ".chars().collect()));
        // BAR fits horizontally, but cannot be duplicated.
        assert_eq!(true, cw.try_word(p00, Dir::Right, &"BAR".chars().collect()));
        assert_eq!(false, cw.try_word(p01, Dir::Right, &"BAR".chars().collect()));
        assert_eq!("BAR".to_string(), cw.chars_at(p00, Dir::Right).collect::<String>());
        assert_eq!(true, cw.try_word(p30, Dir::Right, &"BAZ".chars().collect()));
        // BARBAZ is also a word. Combine BAR and BAZ, so that they are free again:
        assert_eq!(true, cw.try_word(p00, Dir::Right, &"BARBAZ".chars().collect()));
        assert_eq!(true, cw.try_word(p01, Dir::Right, &"BAR".chars().collect()));
        assert_eq!(true, cw.try_word(p00, Dir::Down, &"BB".chars().collect()));
    }
}
