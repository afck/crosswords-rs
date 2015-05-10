use cw::{CVec, BLOCK};
use rand;
use rand::Rng;
use std::ascii::AsciiExt;
use std::cmp;
use std::collections::{HashMap, HashSet};
use word_constraint::WordConstraint;

fn matches(word: &CVec, pattern: &CVec) -> bool {
    word.len() <= pattern.len()
        && word.iter().zip(pattern.iter()).all(|(&cw, &cp)| cw == cp || cp == BLOCK)
}

pub struct PatternIter<'a> {
    dict: &'a Dict,
    pattern: CVec,
    list: Option<&'a Vec<usize>>,
    index: usize,
}

impl<'a> PatternIter<'a> {
    fn get_word(&self) -> Option<&'a CVec> {
        self.dict.words.get(self.pattern.len()).and_then(|w| w.get(match self.list {
            None => self.index,
            Some(list) => *list.get(self.index).unwrap_or(&w.len()),
        }))
    }
}

impl<'a> Iterator for PatternIter<'a> {
    type Item = &'a CVec;

    fn next(&mut self) -> Option<&'a CVec> {
        loop {
            let word = self.get_word();
            self.index += 1;
            if word.iter().all(|w| matches(w, &self.pattern)) {
                return word;
            }
        }
    }
}

pub struct Dict {
    words: Vec<Vec<CVec>>,
    lists: HashMap<WordConstraint, Vec<usize>>,
    max_n: usize,
    empty_list: Vec<usize>,
}

impl Dict {
    pub fn to_cvec_set<T: Iterator<Item = String>>(string_words: T) -> HashSet<CVec> {
        string_words.filter_map(|string_word| Dict::normalize_word(string_word)).collect()
    }

    pub fn new<'a, T: Iterator<Item = &'a CVec>>(all_words: T) -> Dict {
        let mut dict = Dict {
            words: Vec::new(),
            lists: HashMap::new(),
            max_n: 3, // TODO: Make this a parameter?
            empty_list: Vec::new(),
        };
        for word in all_words {
            dict.add_word(word);
        }
        let mut rng = rand::thread_rng(); // TODO: Make this a parameter?
        for i in 0..dict.words.len() {
            rng.shuffle(&mut dict.words[i][..]);
        }
        for len in 0..dict.words.len() {
            for (i, word) in dict.words[len].iter().enumerate() {
                for wc in WordConstraint::all_constraints(word, dict.max_n) {
                    if !dict.lists.get(&wc).is_some() {
                        dict.lists.insert(wc.clone(), Vec::new());
                    }
                    dict.lists.get_mut(&wc).unwrap().push(i);
                }
            }
        }
        dict
    }

    fn add_word(&mut self, word: &CVec) {
        while self.words.len() < word.len() + 1 {
            self.words.push(Vec::new());
        }
        self.words[word.len()].push(word.clone());
    }

    fn normalize_word(string_word: String) -> Option<CVec> {
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

    fn get_list<'a>(&'a self, wc: &WordConstraint) -> &'a Vec<usize> {
        self.lists.get(wc).unwrap_or(&self.empty_list)
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

    /// Return an iterator over all words in the dictionary.
    pub fn all_words<'a>(&'a self) -> Box<Iterator<Item = &CVec> + 'a> {
        Box::new(self.words.iter().flat_map(|list| list.iter()))
    }

    /// Return an iterator over all words in the dictionary matching the given pattern.
    pub fn matching_words<'a>(&'a self, pattern: CVec) -> PatternIter<'a> {
        let len = pattern.len();
        let mut list: Option<&'a Vec<usize>> = None;
        let mut pos = 0;
        'outer: for i in pattern.iter().enumerate()
                .filter(|&(_, ch)| ch == &BLOCK)
                .map(|(i, _)| i)
                .chain(Some(len).into_iter()) {
            if i > pos {
                let subword = &pattern[pos..i];
                let n = cmp::min(self.max_n, subword.len());
                for dp in 1..(subword.len() - n) {
                    let wc = WordConstraint::with_ngram(&subword[dp..(dp + n)], pos + dp, len);
                    let new_list = self.get_list(&wc);
                    if list.iter().all(|l| l.len() > new_list.len()) {
                        list = Some(new_list);
                        if new_list.len() == 0 {
                            break 'outer;
                        }
                    }
                }
            }
            pos = i + 1;
        }
        PatternIter {
            dict: self,
            pattern: pattern,
            list: list,
            index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test() {
        let words = Dict::to_cvec_set(vec!("FOO", "FOOBAR", "FOE", "TOE").into_iter().map(|s| s.to_string()));
        let dict = Dict::new(words.iter());
        assert_eq!(2, dict.matching_words("#OE".chars().collect()).count());
        assert_eq!(1, dict.matching_words("F#E".chars().collect()).count());
        assert_eq!(0, dict.matching_words("T#O".chars().collect()).count());
        assert_eq!(0, dict.matching_words("F###".chars().collect()).count());
        assert_eq!(0, dict.matching_words("##".chars().collect()).count());
    }
}

