use cw::{CVec, Range};
use dict::Dict;

pub struct WordRangeIter<'a> {
    ranges: Vec<Range>,
    range_i: usize,
    dicts: &'a[Dict],
    dict_i: usize,
    word_i: usize,
}

impl<'a> WordRangeIter<'a> {
    pub fn new(ranges: Vec<Range>, dicts: &'a[Dict]) -> WordRangeIter<'a> {
        WordRangeIter {
            ranges: ranges,
            dicts: dicts,
            word_i: 0,
            range_i: 0,
            dict_i: 0,
        }
    }

    #[inline]
    fn get_word(&self) -> Option<CVec> {
        let range = match self.ranges.get(self.range_i) {
            None => return None,
            Some(r) => r,
        };
        self.dicts.get(self.dict_i).and_then(|dict| dict.get_word(range.len, self.word_i))
    }

    fn advance(&mut self) {
        self.word_i += 1;
        while self.dict_i < self.dicts.len() && self.get_word().is_none() {
            self.word_i = 0;
            self.range_i += 1;
            while self.range_i < self.ranges.len() && self.get_word().is_none() {
                self.range_i += 1;
            }
            if self.range_i >= self.ranges.len() {
                self.range_i = 0;
                self.dict_i += 1;
            }
        }
    }

    pub fn next(&mut self) -> Option<(Range, CVec)> {
        let mut oword = self.get_word();
        while oword.is_none() && self.dict_i < self.dicts.len() {
            self.advance();
            oword = self.get_word();
        }
        if let Some(word) = oword {
            let range = self.ranges[self.range_i];
            self.advance();
            Some((range, word))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cw::{Dir, Point, Range};
    use dict::Dict;
    use std::collections::HashSet;

    #[test]
    fn test_range_iter() {
        let point = Point::new(0, 0);
        let ranges = vec!(
            Range { point: point, dir: Dir::Right, len: 6 },
            Range { point: point, dir: Dir::Right, len: 3 },
            Range { point: point, dir: Dir::Right, len: 2 },
        );
        let dicts = vec!(
            Dict::new(&vec!("FAV".to_string(),
                            "TOOLONG".to_string()).into_iter().collect::<HashSet<_>>()),
            Dict::new(&vec!("YO".to_string(),
                            "FOO".to_string(),
                            "FOOBAR".to_string()).into_iter().collect::<HashSet<_>>()),
        );
        let mut iter = WordRangeIter::new(ranges.clone(), &dicts);
        assert_eq!(Some((ranges[1], "FAV".chars().collect())), iter.next());
        assert_eq!(Some((ranges[0], "FOOBAR".chars().collect())), iter.next());
        assert_eq!(Some((ranges[1], "FOO".chars().collect())), iter.next());
        assert_eq!(Some((ranges[2], "YO".chars().collect())), iter.next());
    }
}
