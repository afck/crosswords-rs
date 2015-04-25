use cw::{BLOCK, Crosswords, Dir, Range};
use dict::Dict;
use point::Point;
use std::cmp;
use std::cmp::Ordering;
use std::collections::HashSet;

// TODO
//
// 3 stages?
// (1) Favorite words (empty cells allowed)
// (2) Fill in the rest from the dictionary
// (3) Finalize: Put every word that's still possible
//
// Functionality (mainly for 2):
// (a) Find sets of ranges one of which must be filled:
//      * Each word crosses N (2?) others?
//      * > N% (30%?) of each word's characters cross other word.
//      * No cell can remain empty.
// (b) Choose the most restrictive set of ranges:
//      * Lowest estimate of matching words.
// (c) Iterate over all possible words and recursively complete the crosswords
//      * Start with the most favorable ranges and the favorite words.
//      * (Optional: Always compare 10 options and choose the most promising one, e. g. by highest
//                   estimate of possible crossing words.)
// (d) If no words are possible, backtrack:
//      * Remove the latest words, up to the latest one intersecting the impossible ranges.
//      * (Optional: Go even further if it's easy to determine that that won't help yet.)
//      * (Optional: Keep the current state - in case of failure, return the "best" failure.)
//
// Evaluation for ranges:
// * Must contain a letter, i. e. cross another word, to ensure connectedness.
// * Long ranges are preferable.
// * Crossing many words is a plus.
//
// Evaluation for complete result:
// * Must be connected.
// * No (few?) empty cells.
// * Percentage of borders.
// * Number of favorites. (Weighted by length?)
// * Minimum/average percentage of letters per word that don't belong to a crossing word.

struct WordRangeIter<'a> {
    ranges: Vec<Range>,
    range_i: usize,
    dicts: &'a Vec<Dict>,
    dict_i: usize,
    word_i: usize,
}

impl<'a> WordRangeIter<'a> {
    fn new(ranges: Vec<Range>, dicts: &'a Vec<Dict>) -> WordRangeIter<'a> {
        WordRangeIter {
            ranges: ranges,
            dicts: dicts,
            word_i: 0,
            range_i: 0,
            dict_i: 0,
        }
    }

    #[inline]
    fn get_word(&self) -> Option<Vec<char>> {
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

    fn next(&mut self) -> Option<(Range, Vec<char>)> {
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

    fn into_ranges(self) -> Vec<Range> {
        self.ranges
    }
}

#[test]
fn test_range_iter() {
    let point = Point::new(0, 0);
    let ranges = vec!(
        Range { point: point, dir: Dir::Right, len: 6 },
        Range { point: point, dir: Dir::Right, len: 3 },
        Range { point: point, dir: Dir::Right, len: 2 },
    );
    let dicts = vec!(
        Dict::new(vec!("FAV".to_string(),
                       "TOOLONG".to_string()).into_iter()),
        Dict::new(vec!("YO".to_string(),
                       "FOO".to_string(),
                       "BAR".to_string(),
                       "FOOBAR".to_string()).into_iter()),
    );
    let mut iter = WordRangeIter::new(ranges.clone(), &dicts);
    assert_eq!(Some((ranges[1], "FAV".chars().collect())), iter.next());
    assert_eq!(Some((ranges[0], "FOOBAR".chars().collect())), iter.next());
    assert_eq!(Some((ranges[1], "FOO".chars().collect())), iter.next());
    assert_eq!(Some((ranges[1], "BAR".chars().collect())), iter.next());
    assert_eq!(Some((ranges[2], "YO".chars().collect())), iter.next());
}

#[derive(Clone, PartialEq)]
struct RangeSet {
    ranges: HashSet<Range>,
    est: f32, // Estimated number of words that fit in one of the ranges.
}

impl PartialOrd for RangeSet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.est.partial_cmp(&other.est)
    }
}

impl RangeSet {
    fn new() -> RangeSet {
        RangeSet { ranges: HashSet::new(), est: 0_f32 }
    }

    fn union<T: Iterator<Item = RangeSet>>(itr: T) -> RangeSet {
        let mut result = RangeSet::new();
        for rs in itr {
            result.ranges.extend(rs.ranges.iter().cloned());
            result.est += rs.est;
        }
        result
    }
}

pub struct Author {
    min_crossing: usize,
    min_crossing_rel: f32,
    dicts: Vec<Dict>,
}

impl Author {
    pub fn new(words: &Vec<HashSet<String>>, min_crossing: usize, min_crossing_rel: f32) -> Author {
        Author {
            min_crossing: min_crossing,
            //min_crossing_rel: 1.0,
            min_crossing_rel: min_crossing_rel,
            dicts: words.iter().map(|s| Dict::new(s)).collect(),
        }
    }

    fn get_max_noncrossing(&self, len: usize) -> usize {
        len - cmp::min(len,
                       cmp::max(self.min_crossing, (self.min_crossing_rel * len as f32) as usize))
    }

    fn add_range(&self, cw: &Crosswords, rs: &mut RangeSet, range: Range) {
        let dp = range.dir.point();
        let p = range.point;
        let is_valid = self.min_crossing_rel != 1.0
            || !(p.x + dp.x * (range.len as i32 + 1) == cw.get_width() as i32
            || p.y + dp.y * (range.len as i32 + 1) == cw.get_height() as i32
            || p.x - dp.x * 2 == -1 || p.y - dp.y * 2 == -1);
        if is_valid && rs.ranges.insert(range) {
            rs.est += self.dicts[1].estimate_matches(cw.chars(range).collect());
        }
    }

    fn get_all_ranges(&self, cw: &Crosswords, point: Point, dir: Dir) -> RangeSet {
        let mut rs = RangeSet::new();
        let mut i = 1;
        let dp = dir.point();
        while cw.get_border(point - dp * i, dir)
                && (point - dp * (i - 1)).coord(cw.get_width(), cw.get_height()).is_some() {
            let mut j = 0;
            while cw.get_border(point + dp * j, dir)
                    && (point + dp * j).coord(cw.get_width(), cw.get_height()).is_some() {
                if i + j > 1 {
                    self.add_range(cw, &mut rs, Range {
                        point: point - dp * (i - 1),
                        dir: dir,
                        len: i + j,
                    });
                }
                j += 1;
            }
            i += 1;
        }
        rs
    }

    fn get_word_range_set(&self, cw: &Crosswords) -> Option<RangeSet> {
        let mut result = None;
        for range in cw.words() {
            let odir = range.dir.other();
            let odp = odir.point();
            let candidate_points: Vec<Point> = range.points().filter(|&p| {
                cw.get_border(p, odir) && cw.get_border(p - odp, odir)
            }).collect();
            let nc = candidate_points.len();
            let mnc = self.get_max_noncrossing(range.len);
            if nc > mnc {
                if mnc == 0 {
                    for p in candidate_points.into_iter() {
                        let rs = self.get_all_ranges(cw, p, odir);
                        if result.iter().all(|result_rs| &rs < result_rs) { result = Some(rs) }
                    }
                } else {
                    let mut rsets = candidate_points.into_iter()
                        .map(|p| self.get_all_ranges(cw, p, odir)).collect::<Vec<_>>();
                    rsets.sort_by(|rs0, rs1| rs0.partial_cmp(rs1).unwrap_or(Ordering::Equal));
                    let rs = RangeSet::union(rsets.into_iter().take(mnc + 1));
                    if result.iter().all(|result_rs| &rs < result_rs) { result = Some(rs) }
                }
            }
        }
        result
    }

    fn range_score(&self, cw: &Crosswords, range: &Range) -> i32 {
        let dp = range.dir.point();
        let p = range.point;
        let penalty = if p.x + dp.x * (range.len as i32 + 1) == cw.get_width() as i32
            || p.y + dp.y * (range.len as i32 + 1) == cw.get_height() as i32
            || p.x - dp.x * 2 == -1 || p.y - dp.y * 2 == -1 {
          10
        } else { 0 };
        (cw.chars(*range).filter(|&c| c != BLOCK).count() + range.len) as i32 - penalty
    }

    fn get_ranges_for_empty(&self, cw: &Crosswords) -> RangeSet {
        let mut result = RangeSet::new();
        let point = Point::new(0, 0);
        for len in (2..(1 + cw.get_width())) {
            self.add_range(cw, &mut result, Range { point: point, dir: Dir::Right, len: len });
        }
        for len in (2..(1 + cw.get_height())) {
            self.add_range(cw, &mut result, Range { point: point, dir: Dir::Down, len: len });
        }
        result
    }

    fn get_ranges(&self, cw: &Crosswords) -> Option<Vec<Range>> {
        let mut result = if cw.is_empty() {
            Some(self.get_ranges_for_empty(cw))
        } else {
            self.get_word_range_set(cw)
        };
        // TODO: Avoid ranges that would isolate clusters of empty cells in the first place.
        if result.is_none() && !cw.is_full() {
            let mut rs = RangeSet::new();
            for point in cw.get_smallest_empty_cluster() {
                for range in self.get_all_ranges(cw, point, Dir::Right).ranges.into_iter().chain(
                        self.get_all_ranges(cw, point, Dir::Down).ranges.into_iter()) {
                    if cw.chars(range).any(|c| c != BLOCK) {
                        self.add_range(cw, &mut rs, range);
                    }
                }
            }
            result = Some(rs);
        }
        result.map(|rs| {
            let mut ranges: Vec<Range> = rs.ranges.iter().cloned().collect();
            ranges.sort_by(|r0, r1| self.range_score(cw, r1).cmp(&self.range_score(cw, r0)));
            ranges
        })
    }

    pub fn complete_cw(&self, init_cw: &Crosswords) -> Crosswords {
        let mut cw = init_cw.clone();
        // TODO: Don't stop at success: Backtrack and compare with other solutions.
        let mut stack = Vec::new();
        let mut iter = match self.get_ranges(&cw/*, &freqs*/) {
            Some(ranges) => WordRangeIter::new(ranges, &self.dicts),
            None => return cw,
        };
        'main: loop {
            if let Some((range, word)) = iter.next() {
                if cw.try_word(range.point, range.dir, &word) {
                    stack.push((range, iter));
                    if let Some(ranges) = self.get_ranges(&cw/*, &freqs*/) {
                        iter = WordRangeIter::new(ranges, &self.dicts);
                    } else {
                        return cw;
                    }
                }
            } else {
                let ranges = iter.into_ranges();
                while let Some((range, prev_iter)) = stack.pop() {
                    iter = prev_iter;
                    println!("{}", cw);
                    let word = cw.pop_word(range.point, range.dir);
                    println!("Popping {}, range {:?}",
                             word.clone().into_iter().collect::<String>(), range);
                    // TODO: If empty, pop some word next to the empty cells.
                    if ranges.is_empty() || ranges.iter().any(|r| range.intersects(r)) {
                        continue 'main;
                    }
                }
                // Went all up the stack but found nothing? Give up and return unchanged grid.
                return cw;
            }
            // TODO: If time is up or user interrupts, break.
        }
    }
}

