extern crate rand;

use rand::Rng;
use std::collections::{BTreeSet, HashSet};
use std::fmt::{Debug, Formatter};

static BLOCK: char = '#';

#[derive(Clone, Copy)]
enum Orientation {
    HORIZONTAL,
    VERTICAL,
}

struct Crosswords {
    width: usize,
    height: usize,
    chars: Vec<char>,
    border: Vec<bool>,
    stack: Vec<(usize, usize, Orientation)>,
    words: HashSet<String>,
}

impl Crosswords {
    fn new(width: usize, height: usize) -> Crosswords {
        Crosswords {
            width: width,
            height: height,
            chars: std::iter::repeat(BLOCK).take(width * height).collect(),
            border: std::iter::repeat(true).take(2 * width * height).collect(),
            stack: Vec::new(),
            words: HashSet::new(),
        }
    }

    fn is_char_allowed(&self, p: usize, c: char) -> bool {
        let existing = self.chars[p];
        c == existing || existing == BLOCK
    }

    fn put_char(&mut self, p: usize, c: char) -> char {
        let existing = self.chars[p];
        self.chars[p] = c;
        existing
    }

    fn is_word_allowed(&self, x: usize, y: usize, orientation: Orientation, word: &String) -> bool {
        let p = x + self.width * y;
        let (r, brd, lim, dp) = match orientation {
            Orientation::HORIZONTAL => (x, 0, self.width, 1),
            Orientation::VERTICAL => (y, 1, self.height, self.width),
        };
        let len = word.len();
        !self.words.contains(word)
            && len >= 1 && r + len - 1 < lim
            && (r == 0 || self.border[brd + 2 * (p - dp)])
            && word.chars().enumerate().all(|(i, c)| self.is_char_allowed(p + i * dp, c)
                                            && self.border[brd + 2 * (p + i * dp)])
    }

    fn put_word(&mut self, x: usize, y: usize, orientation: Orientation, word: &String) {
        let p = x + self.width * y;
        let (brd, dp) = match orientation {
            Orientation::HORIZONTAL => (0, 1),
            Orientation::VERTICAL => (1, self.width),
        };
        let len = word.len();
        for (i, c) in word.chars().enumerate() {
            self.put_char(p + i * dp, c);
        }
        for i in 0..(len - 1) {
            self.border[brd + 2 * (p + i * dp)] = false;
        }
        self.words.insert(word.clone());
        self.stack.push((x, y, orientation));
    }

    fn try_word(&mut self, x: usize, y: usize, orientation: Orientation, word: &String) -> bool {
        if self.is_word_allowed(x, y, orientation, word) {
            self.put_word(x, y, orientation, word);
            true
        } else {
            false
        }
    }

    /*fn pop_word(&mut self) -> Option<String> {
        self.stack.pop().unwrap().map(|(x, y, orientation)|
            let mut word = String::new();

        for (i, c) in sword.chars().enumerate() {
            self.chars[p + i * dp] = c;
        }
        self.words.remove(&word);
        word
        )
    }*/
}

impl Debug for Crosswords {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        for _ in 0..self.width {
            try!(formatter.write_str(" \u{2014}"));
        }
        try!(formatter.write_str(" \n"));
        for y in 0..self.height {
            try!(formatter.write_str("|"));
            for p in (y * self.width)..((y + 1) * self.width) {
                try!((&self.chars[p] as &std::fmt::Display).fmt(formatter));
                try!(formatter.write_str(if self.border[2 * p] { "|" } else { " " }));
            }
            try!(formatter.write_str("\n"));
            for p in (y * self.width)..((y + 1) * self.width) {
                try!(formatter.write_str(if self.border[2 * p + 1] { " \u{2014}" } else { "  " }));
            }
            try!(formatter.write_str(" \n"));
        }
        Ok(())
    }
}

pub fn generate_crosswords(dict: &BTreeSet<String>, width: usize, height: usize) {
    let mut cw = Crosswords::new(width, height);
    let mut rng = rand::thread_rng();
    let words: Vec<String> = dict.iter().cloned().collect();
    for _ in 0..10000 {
        let (x, y) = (rng.gen_range(0, width - 1), rng.gen_range(0, height - 1));
        let word = rng.choose(&words[..]).unwrap();
        cw.try_word(x, y, Orientation::VERTICAL, word)
          || cw.try_word(x, y, Orientation::HORIZONTAL, word);
    }
    println!("{:?}", cw);
}

#[test]
fn it_works() {
}
