[![Build Status](https://travis-ci.org/afck/crosswords-rs.svg?branch=master)](https://travis-ci.org/afck/crosswords-rs)

# Crosswords-rs

A crosswords generator written in [Rust](https://github.com/rust-lang/rust).

Crosswords-rs reads a list of words and performs a search of all possible arrangements of a
subset of these words to fill a grid, satisfying a configurable set of requirements, e. g.:

* Every word must cross at least 2 other words.
* At least 50 % of the letters of each word must belong to a perpendicular word.

It outputs the crosswords and the solution in two HTML files which can then be edited to give the
hints for the word and produce the complete puzzle.

If several word lists are preferred, the algorithm will prefer the earlier ones. Thus to create a
themed crosswords, give a list of words matching the theme first, and then a general dictionary
(e. g. the 10000 most common words).


## Usage

Build using [Cargo](https://crates.io/):
``` sh
cargo build --release
```
This will download all dependencies and produce the crosswords-rs binary in the target/release
directory. Obtain a word list, e. g.
[this one](https://github.com/first20hours/google-10000-english), and run
crosswords-rs to produce a grid:
``` sh
target/release/crosswords-rs -d dict/google-10000-english.txt
```
After it has found a solution, it will create the puzzle.html and solution.html files.

There are several command line options to tweak the outcome. Use the --help option to view them:
``` sh
target/release/crosswords-rs --help
```
