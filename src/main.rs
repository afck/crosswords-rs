extern crate crosswords_rs;

mod html;

use crosswords_rs::generate_crosswords;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};

fn load_dict(filename: &str) -> Result<HashSet<String>> {
    let mut dict = HashSet::new();
    let file = try!(File::open(filename));
    for line in BufReader::new(file).lines() {
        if let Ok(word) = line {
            dict.insert(word);
        }
    }
    Ok(dict)
}

fn main() {
    let words = vec!(
        load_dict("dict/favorites.txt").unwrap(),
        load_dict("dict/top10000de.txt").unwrap(),
        //load_dict("dict/ngerman.txt").unwrap(),
    );
    html::write_html(generate_crosswords(&words, 21, 12)).unwrap();
    //write_html(generate_crosswords(&words, 10, 5)).unwrap();
}
