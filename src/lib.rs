extern crate rand;

mod author;
mod cw;
mod point;
mod dict;

use cw::Crosswords;
use author::Author;
use dict::Dict;
use std::collections::BTreeSet;


pub fn generate_crosswords(words: &BTreeSet<String>, width: usize, height: usize) {
    let mut author = Author::new(Crosswords::new(width, height), Dict::new(words.iter().cloned()),
                                 rand::thread_rng());
    author.create_cw();
    println!("{:?}", author.get_cw());
    //println!("Finalizing ...");
    //author.finalize_cw();
    //println!("{:?}", author.get_cw());
}
