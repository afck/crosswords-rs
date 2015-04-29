extern crate crosswords_rs;
extern crate getopts;
use getopts::Options;
use std::env;

mod html;

use crosswords_rs::{Author, Crosswords, evaluate};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Result};

fn load_dict(filename: &String, min_word_len: usize) -> Result<HashSet<String>> {
    let mut dict = HashSet::new();
    let file = try!(File::open(filename));
    for line in BufReader::new(file).lines() {
        if let Ok(word) = line {
            // TODO: First replace umlauts, then compare length!
            if word.chars().count() >= min_word_len {
                dict.insert(word);
            }
        }
    }
    Ok(dict)
}

fn write_html_to_file(filename: &str, cw: &Crosswords, solution: bool) -> Result<()> {
    let file = try!(File::create(filename));
    let mut writer = BufWriter::new(file);
    html::write_html(&mut writer, cw, solution)
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("s", "size", "size of the crosswords grid", "<Width>x<Height>");
    opts.optopt("c", "min_crossing", "minimum number of words crossing any given word", "INTEGER");
    opts.optopt("p", "min_crossing_percent",
                "minimum percentage letters of any given word shared with another word", "FLOAT");
    opts.optmulti("d", "dict", "a dictionary file", "FILENAME");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "verbose", "print the current grid status during computation");
    opts.optopt("m", "min_word_len", "don't use words shorter than that", "INTEGER");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let size: Vec<usize> = matches.opt_str("s").unwrap_or("15x10".to_string()).split('x')
        .map(|s| s.parse().unwrap()).collect();
    let (width, height): (usize, usize) = (size[0], size[1]);
    let min_crossing = matches.opt_str("c").unwrap_or("2".to_string()).parse().unwrap();
    let min_crossing_rel = 0.01
        * matches.opt_str("p").unwrap_or("30".to_string()).parse::<f32>().unwrap();
    let min_word_len = matches.opt_str("m").unwrap_or("2".to_string()).parse().unwrap();
    let verbose = matches.opt_present("v");
    let words = match matches.opt_count("d") {
        0 => vec!("dict/favorites.txt".to_string(), "dict/dict.txt".to_string()),
        _ => matches.opt_strs("d")
    }.into_iter().map(|filename| load_dict(&filename, min_word_len).unwrap()).collect();
    let mut author = Author::new(&Crosswords::new(width, height),
                             &words,
                             min_crossing,
                             min_crossing_rel,
                             verbose);
    let cw = author.complete_cw();
    println!("{} / {} words are favorites. Score: {}",
        cw.get_words().iter().filter(|w| author.get_word_category(&w) == Some(0)).count(),
        cw.get_words().len(), evaluate(&cw, &words[0]));
    println!("{}", cw);
    write_html_to_file("puzzle.html", &cw, false).unwrap();
    write_html_to_file("solution.html", &cw, true).unwrap();
}
