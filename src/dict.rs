use cw::BLOCK;
use std::collections::{BTreeSet, HashMap};
use std::iter;

pub struct Dict {
    words: Vec<BTreeSet<Vec<char>>>,
    ngram_freq: HashMap<Vec<char>, f32>,
    ngram_n: usize,
}

impl Dict {
    pub fn new<T: Iterator<Item = String>>(all_words: T) -> Dict {
        let mut words = Vec::new();
        let ngram_n = 3;
        let mut ng_total: Vec<_> = iter::repeat(0).take(ngram_n).collect();
        let mut ng_count: Vec<_> = (0..ngram_n).map(|_| HashMap::new()).collect();
        for string_word in all_words {
            let word: Vec<char> = string_word.chars().collect();
            while words.len() < word.len() + 1 {
                words.push(BTreeSet::new());
            }
            if words[word.len()].insert(word.clone()) {
                for i in 0..ngram_n {
                    for ng in word.windows(i + 1) {
                        ng_total[i] += 1;
                        let old_count = *ng_count[i].get(&ng.to_vec()).unwrap_or(&0);
                        ng_count[i].insert(ng.to_vec(), old_count + 1);
                    }
                }
            }
        }
        let mut ngram_freq = HashMap::new();
        for i in 0..ngram_n {
            let t = ng_total[i] as f32;
            ngram_freq.extend(ng_count[i].iter().map(|(ng, &c)| (ng.clone(), (c as f32) / t)));
        }
        ngram_freq.insert(Vec::new(), 1_f32);
        Dict {
            words: words,
            ngram_freq: ngram_freq,
            ngram_n: ngram_n,
        }
    }

    pub fn matches(word: &Vec<char>, pattern: &Vec<char>) -> bool {
        word.len() <= pattern.len()
            && word.iter().zip(pattern.iter()).all(|(&cw, &cp)| cw == cp || cp == BLOCK)
    }

    pub fn find_matches(&self, pattern: &Vec<char>, n: usize) -> Vec<Vec<char>> {
        let len = pattern.len();
        if len >= self.words.len() { return Vec::new(); }
        let mut matches = Vec::new();
        for word in self.words[len].iter() {
            if Dict::matches(word, pattern) {
                matches.push(word.clone());
                if matches.len() > n {
                    return matches;
                }
            }
        }
        matches
    }

    pub fn get_matches(&self, pattern: &Vec<char>, n: usize) -> Vec<Vec<char>> {
        let mut matches = Vec::new();
        for i in (2..(pattern.len() + 1)).rev() {
            let len = matches.len();
            matches.extend(self.find_matches(&pattern[..i].to_vec(), n - len));
            if matches.len() > n {
                return matches;
            }
        }
        matches
    }

    pub fn estimate_matches<T: Iterator<Item = char>>(&self, pattern: T) -> f32 {
        let mut est = 1_f32;
        let mut candidates = 0_f32;
        let mut ng = Vec::new();
        for (i, c) in pattern.chain(Some(BLOCK).into_iter()).enumerate() {
            if c == BLOCK {
                est *= *self.ngram_freq.get(&ng).unwrap_or(&0_f32);
                ng.clear();
            } else {
                ng.push(c);
                if ng.len() == self.ngram_n {
                    est *= *self.ngram_freq.get(&ng).unwrap_or(&0_f32);
                    ng.remove(0);
                }
            }
            if let Some(s) = self.words.get(i) {
                candidates += s.len() as f32;
            }
        }
        est * candidates
    }
}

