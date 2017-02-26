mod word_range_iter;

use cw::{BLOCK, Crosswords, Dir, Point, Range};
use dict::Dict;
use itertools::Itertools;
use word_stats::WordStats;
use std::cmp;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::usize;
use author::word_range_iter::WordRangeIter;

/// A `RangeSet` represents a choice of ranges in the crosswords grid one of which must be filled
/// in order to satisfy the requirements.
#[derive(Clone, PartialEq)]
struct RangeSet {
    /// One of these ranges must be filled.
    ranges: HashSet<Range>,
    /// If none of the ranges could be filled, backtracking until a word crossing or extending one
    /// of the backtrack ranges is removed will open up new possibilities.
    backtrack_ranges: HashSet<Range>,
    /// Estimated number of words that fit in one of the ranges.
    est: f32,
}

/// `RangeSet`s are ordered by the estimated number of possibilities to place words.
impl PartialOrd for RangeSet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.est.partial_cmp(&other.est)
    }
}

impl RangeSet {
    fn new() -> RangeSet {
        RangeSet {
            ranges: HashSet::new(),
            backtrack_ranges: HashSet::new(),
            est: 0.,
        }
    }

    fn extend(&mut self, other: RangeSet) {
        self.ranges.extend(other.ranges.into_iter());
        self.backtrack_ranges.extend(other.backtrack_ranges.into_iter());
        self.est += other.est;
    }

    fn union<T: Iterator<Item = RangeSet>>(itr: T) -> RangeSet {
        let mut result = RangeSet::new();
        for rs in itr {
            result.extend(rs);
        }
        result
    }
}

struct StackItem<'a> {
    bt_ranges: HashSet<Range>,
    iter: WordRangeIter<'a>,
    range: Range,
    attempts: usize,
}

/// An `Author` produces crossword grids from a given set of dictionaries.
pub struct Author<'a> {
    dicts: &'a [Dict],
    cw: Crosswords,
    min_crossing: usize,
    min_crossing_percent: usize,
    max_attempts: usize,
    stats: WordStats,
    verbose: bool,
    stack: Vec<StackItem<'a>>,
}

/// Replaces the `$result` with the given range set `$rs` if that has a lower estimated word count.
/// If the estimate is 0, return immediately.
// TODO: Find a saner way to do this.
macro_rules! result_range_set {
    ( $result:expr, $rs:expr ) => {
        if $rs.est == 0. {
            return Some($rs);
        }
        if $result.iter().all(|result_rs| &$rs < result_rs) {
            $result = Some($rs);
        }
    };
}

impl<'a> Author<'a> {
    /// Creates a new `Author` with the given initial crosswords grid and the given dictionaries.
    pub fn new(init_cw: &Crosswords, dicts: &'a [Dict]) -> Author<'a> {
        let mut stats = WordStats::new(3);
        stats.add_words(dicts.iter().flat_map(|dict| dict.all_words()));
        Author {
            dicts: dicts,
            stats: stats,
            cw: init_cw.clone(),
            verbose: false,
            min_crossing: 2,
            min_crossing_percent: 0,
            max_attempts: usize::MAX,
            stack: Vec::new(),
        }
    }

    /// Sets the values for the minimum absolute and relative numbers of letters in each word that
    /// are required to be shared with a perpendicular word, and return the modified `Author`.
    pub fn with_min_crossing(mut self,
                             min_crossing: usize,
                             min_crossing_percent: usize)
                             -> Author<'a> {
        if min_crossing_percent > 100 {
            panic!("min_crossing_percent must be between 0 and 100");
        }
        self.min_crossing = min_crossing;
        self.min_crossing_percent = min_crossing_percent;
        self
    }

    /// Sets the maximum number of words to try out in each position. After `max_attempts` words
    /// have been unsuccessfully tried out, the algorithm will backtrack further. Setting this to a
    /// small value can speed up the search but can overlook valid solutions.
    /// Return the modified `Author`.
    pub fn with_max_attempts(mut self, max_attempts: usize) -> Author<'a> {
        self.max_attempts = max_attempts;
        self
    }

    /// Sets the verbosity mode and return the modified `Author`. If `verbose` is true, the current
    /// status of the crosswords grid is printed every time the algorithm backtracks.
    pub fn with_verbosity(mut self, verbose: bool) -> Author<'a> {
        self.verbose = verbose;
        self
    }

    /// Returns the index of the dictionary containing the given word, or None if not found.
    pub fn get_word_category(&self, word: &[char]) -> Option<usize> {
        self.dicts.iter().position(|dict| dict.contains(word))
    }

    fn is_min_crossing_possible_without(&self, range: Range, filled_range: Range) -> bool {
        if self.min_crossing_percent == 100 {
            return range.len == 0 || range.len >= self.stats.get_min_len();
        }
        if range.len < 2 {
            return true;
        }
        let mut c_opts = 0;
        let odir = range.dir.other();
        let odp = odir.point();
        for p in range.points() {
            let r0 = Range {
                point: p,
                dir: odir,
                len: 2,
            };
            let r1 = Range {
                point: p - odp,
                dir: odir,
                len: 2,
            };
            // TODO: Also consider stats here? Require word estimate > 0.
            if !self.cw.both_borders(p, odir) ||
               (!r0.intersects(&filled_range) && self.cw.is_range_free(r0)) ||
               (!r1.intersects(&filled_range) && self.cw.is_range_free(r1)) {
                c_opts += 1;
                if c_opts >= self.min_crossing {
                    return true;
                }
            }
        }
        false
    }

    fn would_isolate_empty_cluster(&self, range: Range, point: Point) -> bool {
        if self.cw.is_letter(point) {
            return false;
        }
        self.cw.get_boundary_iter_for(point, Some(range)).all(|(p0, p1)| {
            let r = Range::with_points(p0, p1);
            !self.cw.is_range_free(r) || (r.dir == range.dir && range.intersects(&r))
        })
    }

    fn wouldnt_block(&self, range: Range, point: Point) -> bool {
        if !self.cw.both_borders(point, range.dir) || !self.cw.contains(point) {
            return true; // Point already belongs to a word or is outside the grid.
        }
        if self.would_isolate_empty_cluster(range, point) {
            return false;
        }
        // Make sure it doesn't make min_crossing crossing words impossible for the perpendicular.
        if self.min_crossing_percent == 100 {
            return true; // Then leaving unfilled length-1 ranges isn't allowed anyway.
        }
        let r = if self.cw.is_letter(point) {
            self.cw.get_word_range_containing(point, range.dir.other())
        } else {
            self.cw.get_free_range_containing(point, range.dir.other())
        };
        self.is_min_crossing_possible_without(r, range)
    }

    /// Returns the maximum number of characters of a word of the given length that don't need to
    /// be connected to a crossing word.
    fn get_max_noncrossing(&self, len: usize) -> usize {
        if self.min_crossing > len {
            return len;
        }
        let rel_min_crossing = (self.min_crossing_percent * len / 100) as usize;
        len - cmp::max(rel_min_crossing, self.min_crossing)
    }

    /// Returns a factor for the word count estimate of a range, depending on how many neighboring
    /// cells are already filled: This lowers the estimate of a range that is adjacent to a
    /// parallel word, and makes such a range more likely to be considered next. This is desirable
    /// because it will lead to a much more restricted set of options in the next turn, and makes
    /// iteration over matches more efficient as the words are indexed by n-grams.
    fn restriction_multiplier(&self, range: Range) -> f32 {
        let mut mul = 1.;
        let odp = range.dir.other().point();
        for p in range.points().filter(|&p| !self.cw.is_letter(p)) {
            // TODO: Figure out the ideal factors.
            mul *= match (self.cw.is_letter(p - odp), self.cw.is_letter(p + odp)) {
                (false, false) => 1.5, // Neighbors are empty.
                (true, true) => 0.5, // Very preferable: Both neighbors are letters.
                _ => 0.8, // Also preferable: One neighbor is a letter.
            }
        }
        mul
    }

    fn add_range(&self, rs: &mut RangeSet, range: Range) {
        let p = range.point;
        let dp = range.dir.point();
        if self.wouldnt_block(range, p - dp) && self.wouldnt_block(range, p + dp * range.len) &&
           self.is_min_crossing_possible_without(self.cw.get_range_before(&range), range) &&
           self.is_min_crossing_possible_without(self.cw.get_range_after(&range), range) {
            let pattern: Vec<_> = self.cw.chars(range).collect();
            let est = self.stats.estimate_matches(&pattern);
            if est != 0. && rs.ranges.insert(range) {
                rs.est += est * self.restriction_multiplier(range);
            }
        }
    }

    /// Returns a range set containing all free ranges with the given point in the given direction.
    fn get_all_ranges(&self, point: Point, dir: Dir, best: &Option<RangeSet>) -> Option<RangeSet> {
        let mut rs = RangeSet::new();
        let range = self.cw.get_free_range_containing(point, dir);
        let dp = dir.point();
        let t = (point.x - range.point.x + point.y - range.point.y) as usize;
        for i in 0..(t + 1) {
            for j in t..range.len {
                if j - i > 0 {
                    self.add_range(&mut rs,
                                   Range {
                                       point: range.point + dp * i,
                                       dir: dir,
                                       len: j - i + 1,
                                   });
                    if best.iter().any(|r| rs.est >= r.est) {
                        return None; // Wouldn't have smaller est than the best range set so far.
                    }
                }
            }
        }
        rs.backtrack_ranges.insert(range);
        Some(rs)
    }

    fn get_word_range_set(&self) -> Option<RangeSet> {
        let mut result = None;
        for range in self.cw.word_ranges() {
            let odir = range.dir.other();
            let candidate_points: Vec<Point> = range.points()
                .filter(|&p| self.cw.both_borders(p, odir))
                .collect();
            let nc = candidate_points.len();
            let mnc = self.get_max_noncrossing(range.len);
            if nc > mnc {
                if mnc == 0 {
                    for p in candidate_points {
                        if let Some(rs) = self.get_all_ranges(p, odir, &result) {
                            result_range_set!(result, rs);
                        }
                    }
                } else {
                    let mut rsets = candidate_points.into_iter()
                        .filter_map(|p| self.get_all_ranges(p, odir, &result))
                        .collect_vec();
                    if rsets.len() >= mnc + 1 {
                        rsets.sort_by(|rs0, rs1| rs0.partial_cmp(rs1).unwrap_or(Ordering::Equal));
                        let rs = RangeSet::union(rsets.into_iter().take(mnc + 1));
                        result_range_set!(result, rs);
                    }
                }
            }
        }
        result
    }

    fn get_range_len_penalty(range: Range) -> i32 {
        match range.len {
            1 => 10,
            2 => 3,
            _ => 0,
        }
    }

    fn range_score(&self, range: &Range) -> i32 {
        (self.cw.chars(*range).filter(|&c| c != BLOCK).count() + range.len) as i32 -
        Author::get_range_len_penalty(self.cw.get_range_before(range)) -
        Author::get_range_len_penalty(self.cw.get_range_after(range))
    }

    fn get_ranges_for_empty(&self) -> RangeSet {
        let mut result = RangeSet::new();
        let point = Point::new(0, 0);
        for len in 2..(1 + self.cw.get_width()) {
            self.add_range(&mut result,
                           Range {
                               point: point,
                               dir: Dir::Right,
                               len: len,
                           });
        }
        for len in 2..(1 + self.cw.get_height()) {
            self.add_range(&mut result,
                           Range {
                               point: point,
                               dir: Dir::Down,
                               len: len,
                           });
        }
        result
    }

    fn get_range_set(&self) -> Option<RangeSet> {
        if self.cw.is_empty() {
            return Some(self.get_ranges_for_empty());
        }
        let mut result = self.get_word_range_set();
        if self.cw.is_full() {
            return result;
        }
        let mut rs = RangeSet::new();
        for (p0, p1) in self.cw.get_smallest_boundary() {
            let dir = if p0.y == p1.y { Dir::Right } else { Dir::Down };
            let p_ranges = match self.get_all_ranges(p0, dir, &result) {
                Some(r) => r,
                _ => return result,
            };
            for range in p_ranges.ranges {
                if self.cw.chars(range).any(|c| c != BLOCK) {
                    self.add_range(&mut rs, range);
                    if result.iter().any(|r| rs.est >= r.est) {
                        return result;
                    }
                }
            }
            rs.backtrack_ranges.extend(p_ranges.backtrack_ranges.into_iter());
        }
        result_range_set!(result, rs);
        result
    }

    fn get_sorted_ranges(&self, range_set: HashSet<Range>) -> Vec<(Range, Vec<char>)> {
        let mut ranges: Vec<(Range, Vec<char>)> = range_set.into_iter()
            .map(|range| (range, self.cw.chars(range).collect()))
            .collect();
        ranges.sort_by(|r0, r1| self.range_score(&r1.0).cmp(&self.range_score(&r0.0)));
        ranges
    }

    fn pop(&mut self) -> Option<StackItem<'a>> {
        let opt_item = self.stack.pop();
        if let Some(ref item) = opt_item {
            let range = item.range;
            if self.verbose {
                println!("{}", &self.cw);
                println!("Popping {} at ({}, {}) {:?}",
                         self.cw.chars(range).collect::<String>(),
                         range.point.x,
                         range.point.y,
                         range.dir);
            }
            self.cw.pop_word(range.point, range.dir);
        }
        opt_item
    }

    /// Pops the stack until there are no more than n words left in the grid.
    pub fn pop_to_n_words(&mut self, n: usize) {
        while self.stack.len() > n {
            self.pop();
        }
    }

    fn range_meets(range: &Range, bt_ranges: &HashSet<Range>) -> bool {
        bt_ranges.is_empty() ||
        bt_ranges.iter().any(|r| range.intersects(r) || range.is_adjacent_to(r))
    }

    pub fn complete_cw(&mut self) -> Option<Crosswords> {
        let mut bt_ranges = HashSet::new();
        let mut attempts = 0;
        let mut iter = match self.pop() {
            Some(item) => item.iter, // Drop bt_ranges, as iter was successful!.
            None => {
                match self.get_range_set() {
                    Some(rs) => WordRangeIter::new(self.get_sorted_ranges(rs.ranges), self.dicts),
                    None => return None,
                }
            }
        };
        'main: loop {
            while let Some((range, word)) = iter.next() {
                if self.cw.try_word(range.point, range.dir, &word) {
                    self.stack.push(StackItem {
                        bt_ranges: bt_ranges,
                        range: range,
                        iter: iter,
                        attempts: attempts + 1,
                    });
                    match self.get_range_set() {
                        Some(rs) => {
                            bt_ranges = rs.backtrack_ranges;
                            iter = WordRangeIter::new(self.get_sorted_ranges(rs.ranges),
                                                      self.dicts);
                            attempts = 0;
                        }
                        None => return Some(self.cw.clone()),
                    };
                }
            }
            while let Some(item) = self.pop() {
                // TODO: Remember which characters not to try again.
                // TODO: Save the current range set as a "try next" hint. (Is there a way to make
                //       that work recursively ...?)
                if Author::range_meets(&item.range, &bt_ranges) &&
                   (item.attempts < self.max_attempts || self.stack.is_empty()) {
                    bt_ranges.extend(item.bt_ranges);
                    iter = item.iter;
                    attempts = item.attempts;
                    continue 'main;
                }
            }
            // Went all up the stack but found nothing? Give up.
            return None;
            // TODO: If time is up or user interrupts, break.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cw::Crosswords;
    use dict::Dict;
    #[cfg(feature = "nightly")]
    use test::Bencher;
    use test_util::*;

    #[test]
    fn test_complete_cw_possible() {
        let dicts = vec![Dict::new(strs_to_cvecs(&["ABC", "EFG"])),
                         Dict::new(strs_to_cvecs(&["AEX", "BFX", "CGX"]))];
        let mut author = Author::new(&Crosswords::new(3, 3), &dicts);
        assert!(author.complete_cw().is_some());
    }

    #[test]
    fn test_complete_cw_impossible() {
        let dicts = vec![Dict::new(strs_to_cvecs(&["ABC", "ABCD"]))];
        let mut author = Author::new(&Crosswords::new(3, 3), &dicts);
        assert!(author.complete_cw().is_none());
    }

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_complete_cw(bencher: &mut Bencher) {
        let width = 5;
        let height = 4;
        let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"[0..(width + height - 1)].as_bytes();
        let horiz_words = letters.windows(width).map(String::from_utf8_lossy).map(str_to_cvec);
        let vert_words = letters.windows(height).map(String::from_utf8_lossy).map(str_to_cvec);
        let dicts = vec![Dict::new(horiz_words), Dict::new(vert_words)];
        bencher.bench_n(10, |b| {
            b.iter(|| {
                assert!(Author::new(&Crosswords::new(width, height), &dicts)
                    .complete_cw()
                    .is_some())
            })
        });
    }
}
