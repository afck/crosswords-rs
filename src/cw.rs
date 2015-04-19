use std::collections::HashSet;
use std::iter;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::{Add, Mul, Sub};

pub static BLOCK: char = '\u{2588}';

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Point {
    x: i32,
    y: i32,
}

impl Point {
    #[inline]
    fn coord(&self, w: usize, h: usize) -> Option<usize> {
        if self.x < 0 || self.y < 0 || self.x as usize >= w || self.y as usize >= h {
            None
        } else {
            Some((self.x as usize) + w * (self.y as usize))
        }
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<i32> for Point {
    type Output = Point;

    fn mul(self, rhs: i32) -> Point {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<usize> for Point {
    type Output = Point;

    fn mul(self, rhs: usize) -> Point {
        Point {
            x: self.x * (rhs as i32),
            y: self.y * (rhs as i32),
        }
    }
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum Dir {
    Right,
    Down,
}

impl Dir {
    fn other(&self) -> Dir {
        match *self {
            Dir::Right => Dir::Down,
            Dir::Down => Dir::Right,
        }
    }

    fn point(&self) -> Point {
        match *self {
            Dir::Right => Point { x: 1, y: 0 },
            Dir::Down => Point { x: 0, y: 1 },
        }
    }
}

pub struct Crosswords {
    width: usize,
    height: usize,
    chars: Vec<char>,
    right_border: Vec<bool>,
    down_border: Vec<bool>,
    stack: Vec<(Point, Dir)>,
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
            stack: Vec::new(),
            words: HashSet::new(),
        }
    }

    #[inline]
    fn get_border(&self, point: Point, dir: Dir) -> bool {
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
    fn set_border(&mut self, point: Point, dir: Dir, value: bool) {
        match dir {
            Dir::Right => match point.coord(self.width - 1, self.height) {
                None => if !value { unreachable!() },
                Some(p) => self.right_border[p] = value,
            },
            Dir::Down => match point.coord(self.width, self.height - 1) {
                None => if !value { unreachable!() },
                Some(p) => self.down_border[p] = value,
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
        self.stack.push((point, dir));
    }

    pub fn try_word(&mut self, x: usize, y: usize, dir: Dir, word: &Vec<char>) -> bool {
        let point = Point { x: x as i32, y: y as i32 };
        if self.is_word_allowed(point, dir, word) {
            self.push_word(point, dir, word);
            true
        } else {
            false
        }
    }

    pub fn get_ranges(&self) -> Vec<(usize, usize, Dir, Vec<char>)> {
        let mut ranges = Vec::new();
        for y in 0..self.height {
            let mut x = 0;
            while x + 1 < self.width {
                let range = self.get_range(x, y, Dir::Right);
                if range.len() > 1 {
                    ranges.push((x, y, Dir::Right, range.clone()));
                    x += range.len();
                } else { x += 1; }
            }
        }
        for x in 0..self.width {
            let mut y = 0;
            while y + 1 < self.height {
                let range = self.get_range(x, y, Dir::Down);
                if range.len() > 1 {
                    ranges.push((x, y, Dir::Down, range.clone()));
                    y += range.len();
                } else { y += 1; }
            }
        }
        ranges
    }

    pub fn get_range(&self, x: usize, y: usize, dir: Dir) -> Vec<char> {
        let mut point = Point { x: x as i32, y: y as i32 };
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

    pub fn pop_word(&mut self) -> Option<Vec<char>> {
        self.stack.pop().map(|(mut point, dir)| {
            let mut word = Vec::new();
            let dp = dir.point();
            let odir = dir.other();
            let odp = odir.point();
            while let Some(p) = point.coord(self.width, self.height) {
                let last = self.get_border(point, dir);
                self.set_border(point, dir, true);
                if self.get_border(point, odir) && self.get_border(point - odp, odir) {
                    word.push(self.put_char(point, BLOCK));
                } else {
                    word.push(self.chars[p]);
                }
                if last { break; }
                point = point + dp;
            }
            self.words.remove(&word);
            word
        })
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
                let point = Point { x: x as i32, y: y as i32};
                try!((&self.chars[point.coord(self.width, self.height).unwrap()] as &fmt::Display)
                     .fmt(formatter));
                try!(formatter.write_str(
                        if self.get_border(point, Dir::Right) { "|" } else { " " }));
            }
            try!(formatter.write_str("\n"));
            for x in 0..self.width {
                let point = Point { x: x as i32, y: y as i32};
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
