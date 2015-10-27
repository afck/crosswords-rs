extern crate getopts;
extern crate hyper;
extern crate regex;
extern crate rand;

mod author;
mod cw;
mod dict;
mod word_constraint;
mod word_stats;

use getopts::Options;
use std::collections::HashMap;
use std::env;
use std::i32;

mod html;
mod get_hints;

use author::Author;
use cw::Crosswords;
use dict::Dict;
use get_hints::get_hints;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Result};
use std::path::Path;
use std::usize;

/// Write the crosswords grid to the file with the given name.
fn write_html_to_file<P: AsRef<Path>>(filename: P, cw: &Crosswords, solution: bool,
                                      hint_text: &HashMap<String, String>) -> Result<()> {
    let file = try!(File::create(filename));
    let mut writer = BufWriter::new(file);
    html::write_html(&mut writer, cw, solution, hint_text)
}

/// Print the usage help message.
fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

/// Score the crosswords grid according to how many borders and favorite words it contains.
fn evaluate(cw: &Crosswords, author: &Author) -> i32 {
    let empty_borders = (cw.max_border_count() - cw.count_borders()) as i32;
    let mut word_count = 0;
    let mut word_category_count = 0;
    for word in cw.get_words() {
        word_count += 1;
        word_category_count += author.get_word_category(word).unwrap() as i32;
    }
    empty_borders + word_count - 2 * word_category_count
}

/// Print the crosswords grid and the word count.
fn print_cw(cw: &Crosswords, author: &Author) {
    println!("{} / {} words are favorites. Score: {}",
        cw.get_words().iter().filter(|w| author.get_word_category(&w) == Some(0)).count(),
        cw.get_words().len(), evaluate(&cw, author));
    println!("{}", cw);
}

/// Create the Options object containing the list of valid command line options.
fn create_opts() -> Options {
    let mut opts = Options::new();
    opts.optopt("s", "size", "size of the crosswords grid", "<Width>x<Height>");
    opts.optopt("c", "min_crossing", "minimum number of words crossing any given word", "INTEGER");
    opts.optopt("p", "min_crossing_percent",
                "minimum percentage letters of any given word shared with another word", "FLOAT");
    opts.optmulti("d", "dict", "a dictionary file", "FILENAME");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "verbose", "print the current grid status during computation");
    opts.optopt("m", "min_word_len", "don't use words shorter than that", "INTEGER");
    opts.optopt("", "samples", "number of grids to create and select the best from", "INTEGER");
    opts.optopt("", "wikipedia", "use hints from Wikipedia in the given language", "LANGUAGE");
    opts.optopt("", "max_attempts", "the maximum number of words to try out in each position",
                "INTEGER");
    opts
}

/// Return a list of dictionaries read from the given filenames.
fn get_dicts<T: Iterator<Item = String>>(filenames: T, min_word_len: usize) -> Vec<Dict> {
    let mut existing_words = HashSet::new();
    filenames.map(|filename| {
        let get_file_lines = |filename| BufReader::new(filename).lines().filter_map(Result::ok);
        let file_lines = File::open(filename).map(get_file_lines).unwrap();
        let dict = Dict::new(Dict::to_cvec_set(file_lines)
                .difference(&existing_words)
                .filter(|word| word.len() >= min_word_len));
        existing_words.extend(dict.all_words().cloned());
        dict
    }).collect()
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let opts = create_opts();
    let matches = opts.parse(&args[1..]).unwrap();
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    // TODO: Sanity checks for option values; proper error messages.
    let size: Vec<usize> = matches.opt_str("s").map_or(vec!(15, 10), |s| s.split('x')
        .map(|s| s.parse().unwrap()).collect());
    let (width, height): (usize, usize) = (size[0], size[1]);
    let min_crossing = matches.opt_str("c").map_or(2, |s| s.parse().unwrap());
    let min_crossing_rel = 0.01 * matches.opt_str("p").map_or(30., |s| s.parse().unwrap());
    let min_word_len = matches.opt_str("m").map_or(2, |s| s.parse().unwrap());
    let max_attempts = matches.opt_str("max_attempts").map_or(usize::MAX, |s| s.parse().unwrap());
    let samples = matches.opt_str("samples").map_or(1, |s| s.parse().unwrap());
    let verbose = matches.opt_present("v");
    let dicts = get_dicts(match matches.opt_count("d") {
        0 => vec!("dict/favorites.txt".to_owned(), "dict/dict.txt".to_owned()),
        _ => matches.opt_strs("d"),
    }.into_iter(), min_word_len);
    let mut author = Author::new(&Crosswords::new(width, height), &dicts)
        .with_min_crossing(min_crossing, min_crossing_rel)
        .with_verbosity(verbose)
        .with_max_attempts(max_attempts);
    let (mut best_cw, mut best_val) = (None, i32::MIN);
    for i in 0..samples {
        if let Some(cw) = author.complete_cw() {
            let val = evaluate(&cw, &author);
            if samples > 1 {
                println!("Solution {} of {}:", i + 1, samples);
                print_cw(&cw, &author);
            }
            if val > best_val {
                best_cw = Some(cw);
                best_val = val;
            }
            author.pop_to_n_words(1);
        }
    }
    if let Some(cw) = best_cw {
        if samples > 1 {
            println!("Best candidate:");
        }
        print_cw(&cw, &author);
        let hint_text = match matches.opt_str("wikipedia") {
            None => HashMap::new(),
            Some(lang) => {
                let word_iter = cw.get_words().iter().map(|cvec| cvec.iter().cloned().collect());
                get_hints(word_iter, lang)
            }
        };
        write_html_to_file("puzzle.html", &cw, false, &hint_text).unwrap();
        write_html_to_file("solution.html", &cw, true, &hint_text).unwrap();
    }
}
