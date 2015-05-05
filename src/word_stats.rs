use cw::{BLOCK, CVec};
use std::cmp;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq)]
struct WordConstraint {
    ngram: CVec,
    pos: usize,
}

pub struct WordStats {
    freq: Vec<HashMap<WordConstraint, usize>>,
    total: Vec<usize>,
    max_n: usize,
}

impl WordStats {
    pub fn new(max_n: usize) -> WordStats {
        WordStats {
            freq: Vec::new(),
            total: Vec::new(),
            max_n: max_n,
        }
    }

    pub fn add_words<T: Iterator<Item = CVec>>(&mut self, words: T) {
        for word in words {
            self.add_word(word);
        }
    }

    pub fn add_word(&mut self, word: CVec) {
        while self.total.len() <= word.len() {
            self.total.push(0);
            self.freq.push(HashMap::new());
        }
        self.total[word.len()] += 1;
        for n in 1..(self.max_n + 1) {
            for (pos, ngram) in word.windows(n).enumerate() {
                let wc = WordConstraint { ngram: ngram.to_vec(), pos: pos };
                let prev_freq = *self.freq[word.len()].get(&wc).unwrap_or(&0);
                self.freq[word.len()].insert(wc, prev_freq + 1);
            }
        }
    }

    fn get_freq(&self, ngram: &[char], pos: usize, len: usize) -> usize {
        *self.freq[len].get(&WordConstraint {
            pos: pos,
            ngram: ngram.to_vec(),
        }).unwrap_or(&0)
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
        let total = *self.total.get(pattern.len()).unwrap_or(&0) as f32;
        if total == 0_f32 {
            return 0_f32;
        }
        let mut probability = 1_f32;
        let (mut pos, mut len) = (0, 0);
        while pos + len < pattern.len() {
            if pattern[pos + len] == BLOCK {
                if len > 0 {
                    probability *= self.get_estimate(&pattern[pos..(pos + len)], pos, pattern.len()) / total;
                    if probability == 0_f32 {
                        return 0_f32;
                    }
                }
                pos = pos + len + 1;
                len = 0;
            } else {
                len += 1;
            }
        }
        if len > 0 {
            probability *= self.get_estimate(&pattern[pos..(pos + len)], pos, pattern.len()) / total;
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
        ws.add_words(words.into_iter());
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
