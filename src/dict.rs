use cw::CVec;
use rand;
use rand::Rng;
use std::ascii::AsciiExt;
use std::collections::HashSet;

pub struct Dict {
    words: Vec<Vec<CVec>>,
}

impl Dict {
    pub fn new(all_string_words: &HashSet<String>) -> Dict {
        let mut words = Vec::new();
        let all_words: HashSet<CVec> = all_string_words.iter().filter_map(|string_word|
            Dict::normalize_word(string_word)).collect();
        for word in all_words {
            while words.len() < word.len() + 1 {
                words.push(Vec::new());
            }
            words[word.len()].push(word);
        }
        let mut rng = rand::thread_rng();
        for i in 0..words.len() {
            rng.shuffle(&mut words[i][..]);
        }
        Dict { words: words }
    }

    fn normalize_word(string_word: &String) -> Option<CVec> {
        // TODO: Use to_uppercase() once it's stable.
        let word: CVec = string_word.to_ascii_uppercase().trim()
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

    // TODO: Index words by n-grams for faster pattern matching.
    pub fn get_word(&self, len: usize, n: usize) -> Option<CVec> {
        self.words.get(len).and_then(|w| w.get(n)).cloned()
    }

    pub fn contains(&self, word: &CVec) -> bool {
        match self.words.get(word.len()) {
            None => false,
            Some(v) => v.iter().any(|w| w == word),
        }
    }

    /*pub fn matches(word: &CVec, pattern: &CVec) -> bool {
        word.len() <= pattern.len()
            && word.iter().zip(pattern.iter()).all(|(&cw, &cp)| cw == cp || cp == BLOCK)
    }

    pub fn find_matches(&self, pattern: &CVec, n: usize) -> Vec<CVec> {
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
}

