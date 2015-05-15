use cw::{CVec, Range};
use dict::{Dict, PatternIter};

/// An iterator over all possibilities to fill one of the given ranges with a word from a set of
/// dictionaries.
pub struct WordRangeIter<'a> {
    ranges: Vec<(Range, CVec)>,
    dicts: &'a Vec<Dict>,
    range_i: usize,
    dict_i: usize,
    pi: Option<PatternIter<'a>>,
}

impl<'a> WordRangeIter<'a> {
    pub fn new(ranges: Vec<(Range, CVec)>, dicts: &'a Vec<Dict>) -> WordRangeIter<'a> {
        WordRangeIter {
            ranges: ranges,
            dicts: dicts,
            range_i: 0,
            dict_i: 0,
            pi: None,
        }
    }

    #[inline]
    fn get_word(&mut self) -> Option<CVec> {
        match self.pi {
            None => None,
            Some(ref mut iter) => iter.next().cloned(),
        }
    }

    fn advance(&mut self) -> bool {
        if self.pi.is_some() {
            self.range_i += 1;
            if self.range_i >= self.ranges.len() {
                self.range_i = 0;
                self.dict_i += 1;
            }
        }
        if let Some(&(_, ref pattern)) = self.ranges.get(self.range_i) {
            self.pi = self.dicts.get(self.dict_i).map(|dict| dict.matching_words(pattern.clone()));
            self.pi.is_some()
        } else {
            false
        }
    }
}

impl<'a> Iterator for WordRangeIter<'a> {
    type Item = (Range, CVec);

    fn next(&mut self) -> Option<(Range, CVec)> {
        let mut oword = self.get_word();
        while oword.is_none() && self.advance() {
            oword = self.get_word();
        }
        oword.map(|word| {
            let (range, _) = self.ranges[self.range_i];
            (range, word)
        })
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
            (Range { point: point, dir: Dir::Right, len: 6 }, "######".chars().collect()),
            (Range { point: point, dir: Dir::Right, len: 3 }, "###".chars().collect()),
            (Range { point: point, dir: Dir::Right, len: 2 }, "##".chars().collect()),
        );
        let dicts = vec!(
            Dict::new(vec!("FAV".chars().collect(),
                           "TOOLONG".chars().collect()).iter()),
            Dict::new(vec!("YO".chars().collect(),
                           "FOO".chars().collect(),
                           "FOOBAR".chars().collect()).iter()),
        );
        let mut iter = WordRangeIter::new(ranges.clone(), &dicts);
        assert_eq!(Some((ranges[1].0, "FAV".chars().collect())), iter.next());
        assert_eq!(Some((ranges[0].0, "FOOBAR".chars().collect())), iter.next());
        assert_eq!(Some((ranges[1].0, "FOO".chars().collect())), iter.next());
        assert_eq!(Some((ranges[2].0, "YO".chars().collect())), iter.next());
    }
}
