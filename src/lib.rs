extern crate rand;

mod author;
mod cw;
mod dict;
mod point;
mod word_stats;

pub use cw::{Crosswords, Dir, Point, PrintItem, Range};
pub use author::Author;
use std::collections::HashSet;

fn evaluate_word(cw: &Crosswords, range: &Range) -> i32 {
    let mut score = range.len as i32;
    let odir = range.dir.other();
    for p in range.points() {
        if !cw.get_border(p, odir) || !cw.get_border(p - odir.point(), odir) {
            score += 1; // Crosses another word.
        }
    }
    score
}

pub fn evaluate(cw: &Crosswords, fav_set: &HashSet<String>) -> i32 {
    let mut score = 0;
    for range in cw.words() {
        score += evaluate_word(cw, &range);
        if fav_set.contains(&cw.chars(range).collect::<String>()) {
            score += 5;
        }
    }
    score
}

