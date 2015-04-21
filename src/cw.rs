use point::Point;
use std::collections::HashSet;
use std::iter;
use std::fmt;
use std::fmt::{Debug, Formatter};

pub static BLOCK: char = '\u{2588}';

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

    pub fn get_ranges(&self) -> Vec<(Point, Dir, Vec<char>)> {
        let mut ranges = Vec::new();
        for y in 0..self.height {
            let mut x = 0;
            while x + 1 < self.width {
                let point = Point::new(x as i32, y as i32);
                let range = self.get_range(point, Dir::Right);
                if range.len() > 1 {
                    ranges.push((point, Dir::Right, range.clone()));
                    x += range.len();
                } else { x += 1; }
            }
        }
        for x in 0..self.width {
            let mut y = 0;
            while y + 1 < self.height {
                let point = Point::new(x as i32, y as i32);
                let range = self.get_range(point, Dir::Down);
                if range.len() > 1 {
                    ranges.push((point, Dir::Down, range.clone()));
                    y += range.len();
                } else { y += 1; }
            }
        }
        ranges
    }

    pub fn get_range(&self, mut point: Point, dir: Dir) -> Vec<char> {
        let dp = dir.point();
        if !self.get_border(point - dp, dir) {
            return Vec::new();
        }
        let mut word = Vec::new();
        while let Some(p) = point.coord(self.width, self.height) {
            if !self.get_border(point, dir) { break; }
            word.push(self.chars[p]);
            point = point + dp;
        }
        word
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
