extern crate rand;

mod author;
mod cw;
mod point;
mod dict;

pub use cw::{Crosswords, Dir, Point, PrintItem, Range};

use author::Author;
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

fn evaluate(cw: &Crosswords, fav_set: &HashSet<String>) -> i32 {
    let mut score = 0;
    for range in cw.words() {
        score += evaluate_word(cw, &range);
        if fav_set.contains(&cw.chars(range).collect::<String>()) {
            score += 5;
        }
    }
    score
}

pub fn generate_crosswords(words: &Vec<HashSet<String>>, width: usize, height: usize)
        -> Crosswords {
    let new_author = Author::new(words);
    let cw = new_author.complete_cw(&Crosswords::new(width, height));
    println!("Score: {}", evaluate(&cw, &words[0]));
    println!("{}", cw);
    //println!("Finalizing ...");
    //author.finalize_cw();
    //println!("{:?}", author.get_cw());
    cw
}
