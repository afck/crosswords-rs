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
        if let Ok(lword) = line {
            let word = lword.to_ascii_uppercase().trim()
                           .replace("ä", "AE")
                           .replace("Ä", "AE")
                           .replace("ö", "OE")
                           .replace("Ö", "OE")
                           .replace("ü", "UE")
                           .replace("Ü", "UE")
                           .replace("ß", "SS");
            if word.chars().all(|c| c.is_alphabetic() && c.is_ascii()) && word.len() > 1 {
                dict.insert(word.clone());
            }
        }
    }
    Ok(dict)
}

fn main() {
    let dict = load_dict("dict/top10000de.txt").unwrap();
    println!("{} words", dict.len());
    generate_crosswords(&dict, 20, 10);
}
