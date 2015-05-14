use cw::{BLOCK, CVec};
use std::cmp;
use std::collections::HashMap;
use std::usize;
use word_constraint::WordConstraint;

pub struct WordStats {
    freq: HashMap<WordConstraint, usize>,
    max_n: usize,
    min_len: usize,
}

impl WordStats {
    pub fn new(max_n: usize) -> WordStats {
        WordStats {
            freq: HashMap::new(),
            max_n: max_n,
            min_len: usize::MAX,
        }
    }

    pub fn add_words<'a, T: Iterator<Item = &'a CVec>>(&mut self, words: T) {
        for word in words {
            self.add_word(word);
        }
    }

    fn get(&self, wc: &WordConstraint) -> usize { *self.freq.get(wc).unwrap_or(&0) }

    fn get_total(&self, len: usize) -> usize { self.get(&WordConstraint::Length(len)) }

    pub fn get_min_len(&self) -> usize { self.min_len }

    fn get_freq(&self, ngram: &[char], pos: usize, len: usize) -> usize {
        self.get(&WordConstraint::with_ngram(ngram, pos, len))
    }

    fn increase(&mut self, wc: WordConstraint) {
        let prev_freq = self.get(&wc);
        self.freq.insert(wc, prev_freq + 1);
    }

    pub fn add_word(&mut self, word: &CVec) {
        self.min_len = cmp::min(self.min_len, word.len());
        for wc in WordConstraint::all(word, self.max_n) {
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
