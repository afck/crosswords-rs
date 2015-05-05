use cw::{CVec, Range};
use dict::Dict;

pub struct WordRangeIter {
    ranges: Vec<Range>,
    range_i: usize,
    dict_i: usize,
    word_i: usize,
}

impl WordRangeIter {
    pub fn new(ranges: Vec<Range>) -> WordRangeIter {
        WordRangeIter {
            ranges: ranges,
            word_i: 0,
            range_i: 0,
            dict_i: 0,
        }
    }

    #[inline]
    fn get_word(&self, dicts: &Vec<Dict>) -> Option<CVec> {
        let range = match self.ranges.get(self.range_i) {
            None => return None,
            Some(r) => r,
        };
        dicts.get(self.dict_i).and_then(|dict| dict.get_word(range.len, self.word_i))
    }

    fn advance(&mut self, dicts: &Vec<Dict>) {
        self.word_i += 1;
        while self.dict_i < dicts.len() && self.get_word(dicts).is_none() {
            self.word_i = 0;
            self.range_i += 1;
            while self.range_i < self.ranges.len() && self.get_word(dicts).is_none() {
                self.range_i += 1;
            }
            if self.range_i >= self.ranges.len() {
                self.range_i = 0;
                self.dict_i += 1;
            }
        }
    }

    pub fn next(&mut self, dicts: &Vec<Dict>) -> Option<(Range, CVec)> {
        let mut oword = self.get_word(dicts);
        while oword.is_none() && self.dict_i < dicts.len() {
            self.advance(dicts);
            oword = self.get_word(dicts);
        }
        if let Some(word) = oword {
            let range = self.ranges[self.range_i];
            self.advance(dicts);
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

    #[test]
    fn test_range_iter() {
        let point = Point::new(0, 0);
        let ranges = vec!(
            Range { point: point, dir: Dir::Right, len: 6 },
            Range { point: point, dir: Dir::Right, len: 3 },
            Range { point: point, dir: Dir::Right, len: 2 },
        );
        let dicts = vec!(
            Dict::new(vec!("FAV".chars().collect(),
                           "TOOLONG".chars().collect()).iter()),
            Dict::new(vec!("YO".chars().collect(),
                           "FOO".chars().collect(),
                           "FOOBAR".chars().collect()).iter()),
        );
        let mut iter = WordRangeIter::new(ranges.clone());
        assert_eq!(Some((ranges[1], "FAV".chars().collect())), iter.next(&dicts));
        assert_eq!(Some((ranges[0], "FOOBAR".chars().collect())), iter.next(&dicts));
        assert_eq!(Some((ranges[1], "FOO".chars().collect())), iter.next(&dicts));
        assert_eq!(Some((ranges[2], "YO".chars().collect())), iter.next(&dicts));
    }
}
