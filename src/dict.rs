use cw::BLOCK;
use rand;
use rand::Rng;
use std::ascii::AsciiExt;
use std::collections::{HashMap, HashSet};
use std::iter;

pub struct Dict {
    words: Vec<Vec<Vec<char>>>,
    ngram_freq: HashMap<Vec<char>, f32>,
    ngram_n: usize,
}

impl Dict {
    pub fn new(all_string_words: &HashSet<String>) -> Dict {
        let mut words = Vec::new();
        let ngram_n = 3;
        let mut ng_total: Vec<_> = iter::repeat(0).take(ngram_n).collect();
        let mut ng_count: Vec<_> = (0..ngram_n).map(|_| HashMap::new()).collect();
        let all_words: HashSet<Vec<char>> = all_string_words.iter().filter_map(|string_word|
            Dict::normalize_word(string_word)).collect();
        for word in all_words {
            while words.len() < word.len() + 1 {
                words.push(Vec::new());
            }
            for i in 0..ngram_n {
                for ng in word.windows(i + 1) {
                    ng_total[i] += 1;
                    let old_count = *ng_count[i].get(&ng.to_vec()).unwrap_or(&0);
                    ng_count[i].insert(ng.to_vec(), old_count + 1);
                }
            }
            words[word.len()].push(word);
        }
        let mut ngram_freq = HashMap::new();
        for i in 0..ngram_n {
            let t = ng_total[i] as f32;
            ngram_freq.extend(ng_count[i].iter().map(|(ng, &c)| (ng.clone(), (c as f32) / t)));
        }
        let mut rng = rand::thread_rng();
        for i in 0..words.len() {
            rng.shuffle(&mut words[i][..]);
        }
        ngram_freq.insert(Vec::new(), 1_f32);
        Dict {
            words: words,
            ngram_freq: ngram_freq,
            ngram_n: ngram_n,
        }
    }

    fn normalize_word(string_word: &String) -> Option<Vec<char>> {
        // TODO: Use to_uppercase() once it's stable.
        let word: Vec<char> = string_word.to_ascii_uppercase().trim()
                       .replace("ä", "AE")
                       .replace("Ä", "AE")
                       .replace("ö", "OE")
                       .replace("Ö", "OE")
                       .replace("ü", "UE")
                       .replace("Ü", "UE")
                       .replace("ß", "SS").chars().collect();
        if word.iter().all(|&c| c.is_alphabetic() && c.is_ascii()) && word.len() > 1 {
            Some(word)
        } else {
            None
        }
    }

    pub fn get_word(&self, len: usize, n: usize) -> Option<Vec<char>> {
        self.words.get(len).and_then(|w| w.get(n)).cloned()
    }

    /*pub fn matches(word: &Vec<char>, pattern: &Vec<char>) -> bool {
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
    }*/

    pub fn estimate_matches(&self, pattern: Vec<char>) -> f32 {
        let candidates = match self.words.get(pattern.len()) {
            None => return 0_f32,
            Some(s) => s.len() as f32,
        };
        let mut est = 1_f32;
        let mut ng = Vec::new();
        for c in pattern.into_iter().chain(Some(BLOCK).into_iter()) {
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
        }
        est * candidates
    }
}

