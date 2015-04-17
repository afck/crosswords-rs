extern crate crosswords_rs;

use crosswords_rs::generate_crosswords;
use std::ascii::AsciiExt;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};

fn load_dict(filename: &str) -> Result<BTreeSet<String>> {
    let file = try!(File::open(filename));
    let reader = BufReader::new(file);
    let mut dict = BTreeSet::new();
    for line in reader.lines() {
        // TODO: Use to_uppercase() once it's stable.
        let word = line.unwrap().to_ascii_uppercase().trim()
                       .replace("ä", "AE")
                       .replace("ö", "OE")
                       .replace("ü", "UE")
                       .replace("ß", "SS");
        dict.insert(word);
    }
    Ok(dict)
}

fn main() {
    let dict = load_dict("dict/top1000de.txt").unwrap();
    generate_crosswords(&dict, 20, 10);
}
