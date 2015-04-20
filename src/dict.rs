use cw::BLOCK;
use std::collections::{BTreeSet, HashMap};

pub struct Dict {
    words: Vec<BTreeSet<Vec<char>>>,
    char_count: HashMap<char, usize>,
    bigram_count: HashMap<(char, char), usize>,
    char_total: usize,
    bigram_total: usize,
}

impl Dict {
    pub fn new() -> Dict {
        Dict {
            words: Vec::new(),
            char_count: HashMap::new(),
            bigram_count: HashMap::new(),
            char_total: 0,
            bigram_total: 0,
        }
    }

    pub fn with_words<T: Iterator<Item = String>>(all_words: T) -> Dict {
        let mut dict = Dict::new();
        for word in all_words {
            dict.add_word(&word.chars().collect());
        }
        dict
    }

    fn add_word(&mut self, word: &Vec<char>) {
        while self.words.len() < word.len() + 1 {
            self.words.push(BTreeSet::new());
        }
        if !self.words[word.len()].insert(word.clone()) {
            return; // Word already present
        }
        let mut prev_c = BLOCK;
        for &c in word.iter() {
            let old_count = *self.char_count.get(&c).unwrap_or(&0);
            self.char_count.insert(c, old_count + 1);
            self.char_total += 1;
            if prev_c != BLOCK {
                let old_bg_count = *self.bigram_count.get(&(prev_c, c)).unwrap_or(&0);
                self.bigram_count.insert((prev_c, c), old_bg_count + 1);
                self.bigram_total += 1;
            }
            prev_c = c;
        }
    }

    pub fn matches(word: &Vec<char>, pattern: &Vec<char>) -> bool {
        word.len() <= pattern.len()
            && word.iter().zip(pattern.iter()).all(|(&cw, &cp)| cw == cp || cp == BLOCK)
    }

    pub fn get_matches(&self, pattern: &Vec<char>) -> Vec<Vec<char>> {
        let mut matches = Vec::new();
        for i in (2..(pattern.len() + 1)).rev() {
            if i >= self.words.len() { continue; }
            for word in self.words[i].iter() {
                if Dict::matches(word, pattern) {
                    matches.push(word.clone());
                    if matches.len() > 5 {
                        return matches;
                    }
                }
            }
        }
        matches
    }

    pub fn estimate_matches(&self, pattern: &Vec<char>) -> f32 {
        let mut est =
            self.words.iter().take(pattern.len() + 1).fold(0, |c, set| c + set.len()) as f32;
        let mut prev_c = BLOCK;
        for &c in pattern.iter() {
            if c != BLOCK {
                est *= if prev_c != BLOCK {
                    (*self.bigram_count.get(&(prev_c, c)).unwrap_or(&0) as f32)
                        / (self.bigram_total as f32)
                        / (*self.char_count.get(&prev_c).unwrap_or(&0) as f32)
                        * (self.char_total as f32)
                } else {
                    (*self.char_count.get(&c).unwrap_or(&0) as f32) / (self.char_total as f32)
                }
            }
            prev_c = c;
        }
        est
    }
}

