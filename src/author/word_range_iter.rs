use cw::Range;
use dict::{Dict, PatternIter};

/// An iterator over all possibilities to fill one of the given ranges with a word from a set of
/// dictionaries.
pub struct WordRangeIter<'a> {
    ranges: Vec<(Range, Vec<char>)>,
    dicts: &'a [Dict],
    range_i: usize,
    dict_i: usize,
    pi: Option<PatternIter<'a>>,
}

impl<'a> WordRangeIter<'a> {
    pub fn new(ranges: Vec<(Range, Vec<char>)>, dicts: &'a [Dict]) -> WordRangeIter<'a> {
        WordRangeIter {
            ranges: ranges,
            dicts: dicts,
            range_i: 0,
            dict_i: 0,
            pi: None,
        }
    }

    #[inline]
    fn get_word(&mut self) -> Option<Vec<char>> {
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
            self.pi = self.dicts.get(self.dict_i).map(|dict| dict.matching_words(&pattern));
            self.pi.is_some()
        } else {
            false
        }
    }
}

impl<'a> Iterator for WordRangeIter<'a> {
    type Item = (Range, Vec<char>);

    fn next(&mut self) -> Option<(Range, Vec<char>)> {
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
    use test_util::*;

    #[test]
    fn test_range_iter() {
        let point = Point::new(0, 0);
        let ranges = vec!(
            (Range { point: point, dir: Dir::Right, len: 6 }, str_to_cvec("######")),
            (Range { point: point, dir: Dir::Right, len: 3 }, str_to_cvec("###")),
            (Range { point: point, dir: Dir::Right, len: 2 }, str_to_cvec("##")),
        );
        let dicts = [
            Dict::new(strs_to_cvecs(&["FAV", "TOOLONG"])),
            Dict::new(strs_to_cvecs(&["YO", "FOO", "FOOBAR"])),
        ];
        let mut iter = WordRangeIter::new(ranges.clone(), &dicts);
        assert_eq!(Some((ranges[1].0, str_to_cvec("FAV"))), iter.next());
        assert_eq!(Some((ranges[0].0, str_to_cvec("FOOBAR"))), iter.next());
        assert_eq!(Some((ranges[1].0, str_to_cvec("FOO"))), iter.next());
        assert_eq!(Some((ranges[2].0, str_to_cvec("YO"))), iter.next());
    }
}
