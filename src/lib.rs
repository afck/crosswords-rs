extern crate rand;

mod author;
mod cw;
mod point;
mod dict;

pub use cw::{Crosswords, Dir, Point, PrintItem, Range};

use author::Author;
use dict::Dict;
use std::collections::{BTreeSet, HashSet};

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

fn evaluate(cw: &Crosswords, fav_set: HashSet<Vec<char>>) -> i32 {
    let mut score = 0;
    for range in cw.words() {
        score += evaluate_word(cw, &range);
        if fav_set.contains(&cw.word_at(range.point, range.dir)) {
            score += 5;
        }
    }
    score
}

pub fn generate_crosswords(words: &BTreeSet<String>, favorites: &BTreeSet<String>,
                           width: usize, height: usize) -> Crosswords {
    let fav_set = favorites.iter().map(|s| s.chars().collect()).collect();
    let fav_vec = favorites.iter().map(|s| s.chars().collect()).collect();
    let mut author = Author::new(Crosswords::new(width, height), Dict::new(words.iter().cloned()),
                                 fav_vec, rand::thread_rng());
    author.create_cw();
    println!("Score: {}", evaluate(author.get_cw(), fav_set));
    println!("{}", author.get_cw());
    //println!("Finalizing ...");
    //author.finalize_cw();
    //println!("{:?}", author.get_cw());
    author.get_cw().clone()
}
