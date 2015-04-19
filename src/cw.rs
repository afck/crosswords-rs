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
    fn other(&self) -> Dir {
        match *self {
            Dir::Right => Dir::Down,
            Dir::Down => Dir::Right,
        }
    }
}

pub struct Crosswords {
    width: usize,
    height: usize,
    chars: Vec<char>,
    right_border: Vec<bool>,
    down_border: Vec<bool>,
    stack: Vec<(usize, usize, Dir)>,
    words: HashSet<Vec<char>>,
}

impl Crosswords {
    pub fn new(width: usize, height: usize) -> Crosswords {
        Crosswords {
            width: width,
            height: height,
            chars: iter::repeat(BLOCK).take(width * height).collect(),
            right_border: iter::repeat(true).take(width * height).collect(),
            down_border: iter::repeat(true).take(width * height).collect(),
            stack: Vec::new(),
            words: HashSet::new(),
        }
    }

    #[inline]
    fn get_border(&self, p: usize, dir: Dir) -> bool {
        match dir {
            Dir::Right => self.right_border[p],
            Dir::Down => self.down_border[p],
        }
    }

    #[inline]
    fn set_border(&mut self, p: usize, dir: Dir, value: bool) {
        match dir {
            Dir::Right => self.right_border[p] = value,
            Dir::Down => self.down_border[p] = value,
        }
    }

    #[inline]
    fn get_corridor(&self, p: usize, dir: Dir) -> bool {
        self.get_border(p, dir.other()) && match dir {
            Dir::Right => p < self.width || self.down_border[p - self.width],
            Dir::Down => p % self.width == 0 || self.right_border[p - 1],
        }
    }

    #[inline]
    fn is_char_allowed(&self, p: usize, c: char) -> bool {
        let existing = self.chars[p];
        c == existing || existing == BLOCK
    }

    #[inline]
    fn put_char(&mut self, p: usize, c: char) -> char {
        let existing = self.chars[p];
        self.chars[p] = c;
        existing
    }

    #[inline]
    fn get_p_dp(&self, x: usize, y: usize, dir: Dir) -> (usize, usize) {
        let p = x + self.width * y;
        let dp = match dir { Dir::Right => 1, Dir::Down => self.width };
        (p, dp)
    }

    pub fn is_word_allowed(&self, x: usize, y: usize, dir: Dir, word: &Vec<char>) -> bool {
        let (p, dp) = self.get_p_dp(x, y, dir);
        let (r, lim) = match dir {
            Dir::Right => (x, self.width),
            Dir::Down => (y, self.height),
        };
        !self.words.contains(word)
            && !word.is_empty() && r + word.len() - 1 < lim
            && (r == 0 || self.get_border(p - dp, dir))
            && word.iter().enumerate().all(|(i, &c)| self.is_char_allowed(p + i * dp, c)
                                                     && self.get_border(p + i * dp, dir))
    }

    fn push_word(&mut self, x: usize, y: usize, dir: Dir, word: &Vec<char>) {
        let (p, dp) = self.get_p_dp(x, y, dir);
        for (i, &c) in word.iter().enumerate() {
            self.put_char(p + i * dp, c);
        }
        for i in 0..(word.len() - 1) {
            self.set_border(p + i * dp, dir, false);
        }
        self.words.insert(word.clone());
        self.stack.push((x, y, dir));
    }

    pub fn try_word(&mut self, x: usize, y: usize, dir: Dir, word: &Vec<char>) -> bool {
        if self.is_word_allowed(x, y, dir, word) {
            self.push_word(x, y, dir, word);
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
        let (p, dp) = self.get_p_dp(x, y, dir);
        let (r, lim) = match dir {
            Dir::Right => (x, self.width),
            Dir::Down => (y, self.height),
        };
        let mut word = Vec::new();
        if !self.get_border(p, dir) || !(r == 0 || self.get_border(p - dp, dir)) {
            return word;
        }
        word.push(self.chars[p]);
        for i in 1..(lim - r) {
            if !self.get_border(p + i * dp, dir) { break; }
            word.push(self.chars[p + i * dp]);
        }
        word
    }

    pub fn pop_word(&mut self) -> Option<Vec<char>> {
        self.stack.pop().map(|(x, y, dir)| {
            let (p, dp) = self.get_p_dp(x, y, dir);
            let (_, dop) = self.get_p_dp(x, y, dir.other());
            let mut word = Vec::new();
            word.push(self.put_char(p, BLOCK));
            let mut i = 0;
            while !self.get_border(p + i * dp, dir) {
                self.set_border(p + i * dp, dir, true);
                i += 1;
                if self.get_corridor(p + i * dp, dir) {
                    word.push(self.put_char(p + i * dp, BLOCK));
                } else {
                    word.push(self.chars[p + i * dp]);
                }
            }
            self.words.remove(&word);
            word
        })
    }

    fn count_borders(&self) -> usize {
        let mut count = 0;
        for p in 0..(self.width * self.height) {
            if self.right_border[p] { count += 1; }
            if self.down_border[p] { count += 1; }
        }
        // The rightmost and bottom borders are always there.
        count - self.width - self.height
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
            for p in (y * self.width)..((y + 1) * self.width) {
                try!((&self.chars[p] as &fmt::Display).fmt(formatter));
                try!(formatter.write_str(if self.get_border(p, Dir::Right) { "|" } else { " " }));
            }
            try!(formatter.write_str("\n"));
            for p in (y * self.width)..((y + 1) * self.width) {
                try!(formatter.write_str(
                        if self.get_border(p, Dir::Down) { " \u{2014}" } else { "  " }));
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
