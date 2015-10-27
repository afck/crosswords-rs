use cw::CVec;
use std::iter;
use std::slice;
use std::ops;
use std::option;

/// A `WordConstraint` represents the subset of all words with a given length, and optionally with
/// a given n-gram at a specific position.
#[derive(Clone, Hash, Eq, PartialEq)]
pub enum WordConstraint {
    /// All words with the given length.
    Length(usize),
    /// Words with the given character at the specified position.
    CharAt(char, usize, usize),
    /// Words with the given bigram at the specified position.
    BigramAt([char; 2], usize, usize),
    /// Words with the given trigram at the specified position.
    TrigramAt([char; 3], usize, usize),
    /// Words with the given n-gram at the specified position. For n <= 3, the corresponding more
    /// specific variant should be used to avoid ambiguity and heap-allocating the n-gram.
    NGramAt(CVec, usize, usize),
}

type NgramIter<'a> = iter::Map<
    iter::Zip<slice::Windows<'a, char>, iter::Enumerate<iter::Repeat<usize>>>,
    fn((&[char], (usize, usize))) -> WordConstraint>;

type AllNgramIter<'a> = iter::FlatMap<
    iter::Zip<iter::Repeat<&'a CVec>, ops::Range<usize>>,
    NgramIter<'a>,
    fn((&'a Vec<char>, usize)) -> NgramIter<'a>>;

/// An iterator over all constraints applying to a given word.
pub type AllConstraintsIter<'a> = iter::Chain<AllNgramIter<'a>, option::IntoIter<WordConstraint>>;

impl WordConstraint {
    /// Create a `WordConstraint` that specifies all words of the given length, and, if the given
    /// n-gram is not empty, with the given n-gram at the specified position.
    pub fn with_ngram(ngram: &[char], pos: usize, len: usize) -> WordConstraint {
        match ngram.len() {
            0 => WordConstraint::Length(len),
            1 => WordConstraint::CharAt(ngram[0], pos, len),
            2 => WordConstraint::BigramAt([ngram[0], ngram[1]], pos, len),
            3 => WordConstraint::TrigramAt([ngram[0], ngram[1], ngram[2]], pos, len),
            _ => WordConstraint::NGramAt(ngram.to_vec(), pos, len),
        }
    }

    fn ngram_constraints(word: &CVec, n: usize) -> NgramIter {
        fn to_constraint((ngram, (pos, len)): (&[char], (usize, usize))) -> WordConstraint {
            WordConstraint::with_ngram(ngram, pos, len)
        };
        word.windows(n).zip(iter::repeat(word.len()).enumerate()).map(to_constraint)
    }

    fn all_ngram_constraints(word: &CVec, max_n: usize) -> AllNgramIter {
        fn to_iter((word, n): (&CVec, usize)) -> NgramIter {
            WordConstraint::ngram_constraints(word, n)
        };
        iter::repeat(word).zip(1..(max_n + 1)).flat_map(to_iter)
    }

    /// Return an iterator over all constraints applying to a given word.
    pub fn all(word: &CVec, max_n: usize) -> AllConstraintsIter {
        WordConstraint::all_ngram_constraints(word, max_n)
            .chain(Some(WordConstraint::Length(word.len())))
    }
}

