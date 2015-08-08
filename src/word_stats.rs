use cw::{BLOCK, CVec};
use std::cmp;
use std::collections::HashMap;
use std::iter;
use std::usize;
use word_constraint::WordConstraint;

/// A `WordStats` represents word frequency statistics for one or more dictionaries. It contains
/// numbers of words satisfying each `WordConstraint` and using these can estimate numbers of words
/// matching a given pattern.
pub struct WordStats {
    freq: HashMap<WordConstraint, usize>,
    max_n: usize,
    min_len: usize,
}

impl WordStats {
    /// Create a new `WordStats` that will count word frequencies for n-grams of up to `max_n`
    /// letters.
    pub fn new(max_n: usize) -> WordStats {
        WordStats {
            freq: HashMap::new(),
            max_n: max_n,
            min_len: usize::MAX,
        }
    }

    /// Add all words in the iterator.
    pub fn add_words<'a, T: Iterator<Item = &'a CVec>>(&mut self, words: T) {
        for word in words {
            self.add_word(word);
        }
    }

    fn get(&self, wc: &WordConstraint) -> usize {
        *self.freq.get(wc).unwrap_or(&0)
    }

    fn get_total(&self, len: usize) -> usize {
        self.get(&WordConstraint::Length(len))
    }

    /// Return the length of the shortest word, or `usize::MAX` if no words have been added yet.
    pub fn get_min_len(&self) -> usize {
        self.min_len
    }

    fn get_freq(&self, ngram: &[char], pos: usize, len: usize) -> usize {
        self.get(&WordConstraint::with_ngram(ngram, pos, len))
    }

    fn increase(&mut self, wc: WordConstraint) {
        let prev_freq = self.get(&wc);
        self.freq.insert(wc, prev_freq + 1);
    }

    /// Increase the word count for each `WordConstraint` matching the given word.
    pub fn add_word(&mut self, word: &CVec) {
        self.min_len = cmp::min(self.min_len, word.len());
        for wc in WordConstraint::all(word, self.max_n) {
            self.increase(wc);
        }
    }

    fn get_estimate(&self, subword: &[char], pos: usize, len: usize) -> f32 {
        let n = cmp::min(self.max_n, subword.len());
        let mut estimate = self.get_freq(&subword[0..n], pos, len) as f32;
        if estimate == 0. {
            return 0.;
        }
        for dp in 1..(subword.len() - n) {
            let next_est = self.get_freq(&subword[dp..(dp + n)], pos + dp, len) as f32;
            if next_est == 0. {
                return 0.;
            }
            estimate *= next_est;
            if n > 1 {
                estimate /= self.get_freq(&subword[dp..(dp + n - 1)], pos + dp, len) as f32;
            }
        }
        estimate
    }

    /// Compute an estimate of the number of words that will match the given pattern.
    pub fn estimate_matches(&self, pattern: &CVec) -> f32 {
        let len = pattern.len();
        let total = self.get_total(len) as f32;
        if total == 0. {
            return 0.;
        }
        let mut probability = 1.;
        let mut pos = 0;
        for i in pattern.iter().enumerate()
                .filter(|&(_, ch)| ch == &BLOCK)
                .map(|(i, _)| i)
                .chain(iter::once(len)) {
            if i > pos {
                probability *= self.get_estimate(&pattern[pos..i], pos, len) / total;
                if probability == 0. {
                    return 0.;
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
        assert_eq!(1., ws.estimate_matches(&"AB##".chars().collect()));
        assert_eq!(1., ws.estimate_matches(&"#B##".chars().collect()));
        assert_eq!(0., ws.estimate_matches(&"#AB#".chars().collect()));
        assert_eq!(0., ws.estimate_matches(&"###A".chars().collect()));
        assert_eq!(0., ws.estimate_matches(&"##".chars().collect()));
        assert_eq!(0., ws.estimate_matches(&"#####".chars().collect()));
        assert_eq!(2., ws.estimate_matches(&"A###".chars().collect()));
        assert_eq!(1., ws.estimate_matches(&"#B##".chars().collect()));
        assert_eq!(1., ws.estimate_matches(&"ABC#".chars().collect()));
        assert_eq!(0., ws.estimate_matches(&"#C##".chars().collect()));
    }
}
