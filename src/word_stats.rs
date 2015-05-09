use cw::{BLOCK, CVec};
use std::cmp;
use std::collections::HashMap;
use std::iter;
use std::slice;
use std::ops;

#[derive(Hash, Eq, PartialEq)]
enum WordConstraint {
    Length(usize),
    CharAt(char, usize, usize),
    BigramAt([char; 2], usize, usize),
    TrigramAt([char; 3], usize, usize),
    NGramAt(CVec, usize, usize),
}

type NgramIter<'a> = iter::Map<
            iter::Zip<slice::Windows<'a, char>, iter::Enumerate<iter::Repeat<usize>>>,
            fn((&[char], (usize, usize))) -> WordConstraint>;

impl WordConstraint {
    fn with_ngram(ngram: &[char], pos: usize, len: usize) -> WordConstraint {
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

    fn all_constraints<'a>(word: &'a CVec, max_n: usize) -> iter::FlatMap<
            iter::Zip<iter::Repeat<&'a CVec>, ops::Range<usize>>,
            NgramIter<'a>,
            fn((&'a Vec<char>, usize)) -> NgramIter<'a>> {
        fn to_iter<'a>((word, n): (&'a CVec, usize)) -> NgramIter<'a> {
            WordConstraint::ngram_constraints(word, n)
        };
        iter::repeat(word).zip(1..(max_n + 1)).flat_map(to_iter)
    }
}

pub struct WordStats {
    freq: HashMap<WordConstraint, usize>,
    max_n: usize,
}

impl WordStats {
    pub fn new(max_n: usize) -> WordStats {
        WordStats {
            freq: HashMap::new(),
            max_n: max_n,
        }
    }

    pub fn add_words<'a, T: Iterator<Item = &'a CVec>>(&mut self, words: T) {
        for word in words {
            self.add_word(word);
        }
    }

    fn get(&self, wc: &WordConstraint) -> usize { *self.freq.get(wc).unwrap_or(&0) }
    
    fn get_total(&self, len: usize) -> usize { self.get(&WordConstraint::Length(len)) }

    fn get_freq(&self, ngram: &[char], pos: usize, len: usize) -> usize {
        self.get(&WordConstraint::with_ngram(ngram, pos, len))
    }

    fn increase(&mut self, wc: WordConstraint) {
        let prev_freq = self.get(&wc);
        self.freq.insert(wc, prev_freq + 1);
    }

    pub fn add_word(&mut self, word: &CVec) {
        self.increase(WordConstraint::Length(word.len()));
        for wc in WordConstraint::all_constraints(word, self.max_n) {
            self.increase(wc);
        }
    }

    fn get_estimate(&self, subword: &[char], pos: usize, len: usize) -> f32 {
        let n = cmp::min(self.max_n, subword.len());
        let mut estimate = self.get_freq(&subword[0..n], pos, len) as f32;
        if estimate == 0_f32 {
            return 0_f32;
        }
        for dp in 1..(subword.len() - n) {
            let next_est = self.get_freq(&subword[dp..(dp + n)], pos + dp, len) as f32;
            if next_est == 0_f32 {
                return 0_f32;
            }
            estimate *= next_est;
            if n > 1 {
                estimate /= self.get_freq(&subword[dp..(dp + n - 1)], pos + dp, len) as f32;
            }
        }
        estimate
    }

    pub fn estimate_matches(&self, pattern: &CVec) -> f32 {
        let len = pattern.len();
        let total = self.get_total(len) as f32;
        if total == 0_f32 {
            return 0_f32;
        }
        let mut probability = 1_f32;
        let mut pos = 0;
        for i in pattern.iter().enumerate()
                .filter(|&(_, ch)| ch == &BLOCK)
                .map(|(i, _)| i)
                .chain(Some(len).into_iter()) {
            if i > pos {
                probability *= self.get_estimate(&pattern[pos..i], pos, len) / total;
                if probability == 0_f32 {
                    return 0_f32;
                }
            }
            pos = i + 1;
        }
        probability * total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cw::CVec;
    use std::collections::HashSet;

    #[test]
    fn test() {
        let mut words: HashSet<CVec> = HashSet::new();
        words.insert("ABCD".chars().collect());
        words.insert("AXYZ".chars().collect());
        let mut ws = WordStats::new(2);
        ws.add_words(words.iter());
        assert_eq!(1_f32, ws.estimate_matches(&"AB##".chars().collect()));
        assert_eq!(1_f32, ws.estimate_matches(&"#B##".chars().collect()));
        assert_eq!(0_f32, ws.estimate_matches(&"#AB#".chars().collect()));
        assert_eq!(0_f32, ws.estimate_matches(&"###A".chars().collect()));
        assert_eq!(0_f32, ws.estimate_matches(&"##".chars().collect()));
        assert_eq!(0_f32, ws.estimate_matches(&"#####".chars().collect()));
        assert_eq!(2_f32, ws.estimate_matches(&"A###".chars().collect()));
        assert_eq!(1_f32, ws.estimate_matches(&"#B##".chars().collect()));
        assert_eq!(1_f32, ws.estimate_matches(&"ABC#".chars().collect()));
        assert_eq!(0_f32, ws.estimate_matches(&"#C##".chars().collect()));
    }
}
