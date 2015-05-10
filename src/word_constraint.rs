use cw::CVec;
use std::iter;
use std::slice;
use std::ops;


#[derive(Clone, Hash, Eq, PartialEq)]
pub enum WordConstraint {
    Length(usize),
    CharAt(char, usize, usize),
    BigramAt([char; 2], usize, usize),
    TrigramAt([char; 3], usize, usize),
    NGramAt(CVec, usize, usize),
}

pub type NgramIter<'a> = iter::Map<
    iter::Zip<slice::Windows<'a, char>, iter::Enumerate<iter::Repeat<usize>>>,
    fn((&[char], (usize, usize))) -> WordConstraint>;

impl WordConstraint {
    pub fn with_ngram(ngram: &[char], pos: usize, len: usize) -> WordConstraint {
        match ngram.len() {
            0 => WordConstraint::Length(len),
            1 => WordConstraint::CharAt(ngram[0], pos, len),
            2 => WordConstraint::BigramAt([ngram[0], ngram[1]], pos, len),
            3 => WordConstraint::TrigramAt([ngram[0], ngram[1], ngram[2]], pos, len),
            _ => WordConstraint::NGramAt(ngram.to_vec(), pos, len),
        }
    }

    fn ngram_constraints<'a>(word: &'a CVec, n: usize) -> NgramIter<'a> {
        fn to_constraint((ngram, (pos, len)): (&[char], (usize, usize))) -> WordConstraint {
            WordConstraint::with_ngram(ngram, pos, len)
        };
        word.windows(n).zip(iter::repeat(word.len()).enumerate()).map(to_constraint)
    }

    pub fn all_constraints<'a>(word: &'a CVec, max_n: usize) -> iter::FlatMap<
            iter::Zip<iter::Repeat<&'a CVec>, ops::Range<usize>>,
            NgramIter<'a>,
            fn((&'a Vec<char>, usize)) -> NgramIter<'a>> {
        fn to_iter<'a>((word, n): (&'a CVec, usize)) -> NgramIter<'a> {
            WordConstraint::ngram_constraints(word, n)
        };
        iter::repeat(word).zip(1..(max_n + 1)).flat_map(to_iter)
    }
}

