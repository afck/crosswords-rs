extern crate rand;

mod author;
mod cw;
mod point;
mod dict;

pub use cw::{Crosswords, Dir, Point, PrintItem};

use author::Author;
use dict::Dict;
use std::collections::BTreeSet;

pub fn generate_crosswords(words: &BTreeSet<String>, width: usize, height: usize) -> Crosswords {
    let mut author = Author::new(Crosswords::new(width, height), Dict::new(words.iter().cloned()),
                                 rand::thread_rng());
    author.create_cw();
    println!("{}", author.get_cw());
    //println!("Finalizing ...");
    //author.finalize_cw();
    //println!("{:?}", author.get_cw());
    author.get_cw().clone()
}
