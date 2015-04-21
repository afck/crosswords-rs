use point::Point;
use std::collections::HashSet;
use std::iter;
use std::fmt;
use std::fmt::{Debug, Formatter};

pub static BLOCK: char = '\u{2588}';

#[derive(Clone, Copy)]
pub struct Range {
    pub point: Point,
    pub dir: Dir,
    pub len: usize,
}

pub struct RangeIter<'a> {
    range: Range,
    cw: &'a Crosswords,
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

pub struct RangesIter<'a> {
    point: Point,
    dir: Dir,
    ended: bool,
    cw: &'a Crosswords,
    words: bool,
}

impl<'a> RangesIter<'a> {
    fn new_free(cw: &'a Crosswords) -> Self {
        RangesIter {
            point: Point::new(0, 0),
            dir: Dir::Right,
            ended: false,
            cw: cw,
            words: false,
        }
    }

    fn new_words(cw: &'a Crosswords) -> Self {
        RangesIter {
            point: Point::new(0, 0),
            dir: Dir::Right,
            ended: false,
            cw: cw,
            words: true,
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
            let len = match self.words {
                true => self.cw.get_word_len_at(self.point, self.dir),
                false => self.cw.get_free_range_at(self.point, self.dir),
            };
            if len > 1 {
                let range = Range { point: self.point, dir: self.dir, len: len };
                self.advance(len); // TODO: If self.words, advance len + 2?
                return Some(range);
            }
            self.advance(1);
        }
        None
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
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

pub struct Crosswords {
    width: usize,
    height: usize,
    chars: Vec<char>,
    right_border: Vec<bool>,
    down_border: Vec<bool>,
    words: HashSet<Vec<char>>,
}

impl Crosswords {
    pub fn new(width: usize, height: usize) -> Crosswords {
        Crosswords {
            width: width,
            height: height,
            chars: iter::repeat(BLOCK).take(width * height).collect(),
            right_border: iter::repeat(true).take((width - 1) * height).collect(),
            down_border: iter::repeat(true).take(width * (height - 1)).collect(),
            words: HashSet::new(),
        }
    }

    pub fn get_width(&self) -> usize { self.width }
    pub fn get_height(&self) -> usize { self.height }

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

    pub fn get_char(&self, point: Point) -> Option<char> {
        point.coord(self.width, self.height).map(|p| self.chars[p])
    }

    pub fn chars<'a>(&'a self, range: Range) -> RangeIter<'a> {
        RangeIter {
            range: range,
            cw: &self,
        }
    }

    #[inline]
    fn put_char(&mut self, point: Point, c: char) -> char {
        let p = point.coord(self.width, self.height).unwrap();
        let existing = self.chars[p];
        self.chars[p] = c;
        existing
    }

    pub fn is_word_allowed(&self, point: Point, dir: Dir, word: &Vec<char>) -> bool {
        let dp = dir.point();
        let len = word.len() as i32;
        !self.words.contains(word) && !word.is_empty()
            && self.get_border(point - dp, dir)
            && self.get_border(point + dp * (len - 1), dir)
            && word.iter().enumerate().all(|(i, &c)| self.is_char_allowed(point + dp * i, c))
    }

    fn push_word(&mut self, point: Point, dir: Dir, word: &Vec<char>) {
        let dp = dir.point();
        let len = word.len() as i32;
        for (i, &c) in word.iter().enumerate() {
            self.put_char(point + dp * i, c);
        }
        for i in 0..(len - 1) {
            self.set_border(point + dp * i, dir, false);
        }
        self.words.insert(word.clone());
    }

    pub fn try_word(&mut self, point: Point, dir: Dir, word: &Vec<char>) -> bool {
        if self.is_word_allowed(point, dir, word) {
            self.push_word(point, dir, word);
            true
        } else {
            false
        }
    }

    pub fn free_ranges<'a>(&'a self) -> RangesIter<'a> {
        RangesIter::new_free(&self)
    }

    pub fn get_free_range_at(&self, mut point: Point, dir: Dir) -> usize {
        let dp = dir.point();
        let mut len = 0;
        if (point - dp).coord(self.width, self.height).is_none()
                || (self.get_border(point - dp, dir) && !self.get_border(point - dp * 2, dir)) {
            while point.coord(self.width, self.height).is_some() && self.get_border(point, dir) {
                len += 1;
                point = point + dp;
            }
        }
        len
    }

    pub fn words<'a>(&'a self) -> RangesIter<'a> {
        RangesIter::new_words(&self)
    }

    pub fn get_word_len_at(&self, mut point: Point, dir: Dir) -> usize {
        let dp = dir.point();
        if self.get_border(point - dp, dir) {
            let mut len = 1;
            while !self.get_border(point, dir) {
                len += 1;
                point = point + dp;
            }
            len
        } else { 0 }
    }

    pub fn pop_word(&mut self, mut point: Point, dir: Dir) -> Option<Vec<char>> {
        let dp = dir.point();
        if self.get_border(point, dir) || !self.get_border(point - dp, dir) {
            return None;
        }
        let odir = dir.other();
        let odp = odir.point();
        let mut word = Vec::new();
        while let Some(p) = point.coord(self.width, self.height) {
            if self.get_border(point, odir) && self.get_border(point - odp, odir) {
                word.push(self.put_char(point, BLOCK));
            } else {
                word.push(self.chars[p]);
            }
            if self.set_border(point, dir, true) { break; }
            point = point + dp;
        }
        self.words.remove(&word);
        Some(word)
    }

    fn count_borders(&self) -> usize {
        let mut count = 0;
        for p in 0..((self.width - 1) * self.height) {
            if self.right_border[p] { count += 1; }
        }
        for p in 0..(self.width * (self.height - 1)) {
            if self.down_border[p] { count += 1; }
        }
        count
    }
}

impl Debug for Crosswords {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        for _ in 0..self.width {
            try!(formatter.write_str(" \u{2014}"));
        }
        try!(formatter.write_str(" \n"));
        for y in 0..self.height {
            try!(formatter.write_str("|"));
            for x in 0..self.width {
                let point = Point::new(x as i32, y as i32);
                try!((&self.chars[point.coord(self.width, self.height).unwrap()] as &fmt::Display)
                     .fmt(formatter));
                try!(formatter.write_str(
                        if self.get_border(point, Dir::Right) { "|" } else { " " }));
            }
            try!(formatter.write_str("\n"));
            for x in 0..self.width {
                let point = Point::new(x as i32, y as i32);
                try!(formatter.write_str(
                        if self.get_border(point, Dir::Down) { " \u{2014}" } else { "  " }));
            }
            try!(formatter.write_str(" \n"));
        }
        let bcount = self.count_borders();
        let btotal = 2 * self.width * self.height - self.width - self.height;
        let bpercent = 100.0 * (bcount as f32) / (btotal as f32);
        try!(formatter.write_fmt(format_args!("{} / {} borders ({}%)", bcount, btotal, bpercent)));
        Ok(())
    }
}
