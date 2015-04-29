mod word_range_iter;

use cw::{BLOCK, Crosswords, CVec, Dir, Point, Range};
use dict::Dict;
use word_stats::WordStats;
use std::cmp;
use std::cmp::Ordering;
use std::collections::HashSet;
use author::word_range_iter::WordRangeIter;

// TODO (some thoughts on extending the algorithm):
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


#[derive(Clone, PartialEq)]
struct RangeSet {
    ranges: HashSet<Range>,
    backtrack_ranges: HashSet<Range>,
    est: f32, // Estimated number of words that fit in one of the ranges.
}

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
            est: 0_f32,
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

pub struct Author {
    dicts: Vec<Dict>,
    cw: Crosswords,
    min_crossing: usize,
    min_crossing_rel: f32,
    stats: WordStats,
    verbose: bool,
}

macro_rules! result_range_set {
    ( $result:expr, $rs:expr ) => {
        if $rs.est == 0_f32 {
            return Some($rs);
        }
        if $result.iter().all(|result_rs| &$rs < result_rs) { $result = Some($rs) }
    };
}

impl Author {
    pub fn new(init_cw: &Crosswords,
               words: &Vec<HashSet<String>>,
               min_crossing: usize,
               min_crossing_rel: f32,
               verbose: bool) -> Author {
        let mut all_words = HashSet::new();
        for word_set in words {
            all_words.extend(word_set.iter().map(|s| s.chars().collect()));
        }
        if min_crossing_rel < 0_f32 || min_crossing_rel > 1_f32 {
            unreachable!("min_crossing_rel must be between 0 and 1");
        }
        // TODO: Dicts should be disjoint.
        Author {
            dicts: words.iter().map(|s| Dict::new(s)).collect(),
            cw: init_cw.clone(),
            // TODO: min_fav_words / max_nonfav_words ...
            //min_avg_cells_per_word: 5_f32, // TODO: Make this a command line option.
            min_crossing: min_crossing,
            min_crossing_rel: min_crossing_rel,
            stats: WordStats::new(3, &all_words),
            verbose: verbose,
        }
    }

    pub fn get_word_category(&self, word: &CVec) -> Option<usize> {
        for (i, dict) in self.dicts.iter().enumerate() {
            if dict.contains(word) {
                return Some(i);
            }
        }
        None
    }

    /// Returns the maximum number of characters of a word of the given length that don't need to
    /// be connected to a crossing word.
    fn get_max_noncrossing(&self, len: usize) -> usize {
        let max_noncrossing = (1_f32 - self.min_crossing_rel) * (len as f32);
        cmp::min(max_noncrossing as usize, len - self.min_crossing)
    }

    fn add_range(&self, rs: &mut RangeSet, range: Range) {
        let blocks_words = self.min_crossing_rel == 1_f32
            && (self.cw.get_range_before(range).len == 1
                || self.cw.get_range_after(range).len == 1);
        if !blocks_words {
            let est = self.stats.estimate_matches(&self.cw.chars(range).collect());
            if est != 0_f32 && rs.ranges.insert(range) {
                rs.est += est;
            }
        }
    }

    /// Returns a range set containing all free ranges with the given point in the given direction.
    /// The backtrack range extends one field past the longest of these ranges.
    fn get_all_ranges(&self, point: Point, dir: Dir) -> RangeSet {
        let mut rs = RangeSet::new();
        let mut i = 1;
        let mut j = 0;
        let dp = dir.point();
        while self.cw.get_border(point - dp * i, dir) && self.cw.contains(point - dp * (i - 1)) {
            j = 0;
            while self.cw.get_border(point + dp * j, dir) && self.cw.contains(point + dp * j) {
                if i + j > 1 {
                    self.add_range(&mut rs, Range {
                        point: point - dp * (i - 1),
                        dir: dir,
                        len: i + j,
                    });
                }
                j += 1;
            }
            i += 1;
        }
        rs.backtrack_ranges.insert(Range {
            point: point - dp * (i - 2),
            dir: dir,
            len: i + j - 2,
        });
        rs
    }

    fn get_word_range_set(&self) -> Option<RangeSet> {
        let mut result = None;
        for range in self.cw.word_ranges() {
            let odir = range.dir.other();
            let odp = odir.point();
            let candidate_points: Vec<Point> = range.points().filter(|&p| {
                self.cw.get_border(p, odir) && self.cw.get_border(p - odp, odir)
            }).collect();
            let nc = candidate_points.len();
            let mnc = self.get_max_noncrossing(range.len);
            if nc > mnc {
                if mnc == 0 {
                    for p in candidate_points.into_iter() {
                        let rs = self.get_all_ranges(p, odir);
                        result_range_set!(result, rs);
                    }
                } else {
                    let mut rsets = candidate_points.into_iter()
                        .map(|p| self.get_all_ranges(p, odir)).collect::<Vec<_>>();
                    rsets.sort_by(|rs0, rs1| rs0.partial_cmp(rs1).unwrap_or(Ordering::Equal));
                    let rs = RangeSet::union(rsets.into_iter().take(mnc + 1));
                    result_range_set!(result, rs);
                }
            }
        }
        result
    }

    fn range_score(&self, range: &Range) -> i32 {
        (self.cw.chars(*range).filter(|&c| c != BLOCK).count() + range.len) as i32
    }

    fn get_ranges_for_empty(&self) -> RangeSet {
        let mut result = RangeSet::new();
        let point = Point::new(0, 0);
        for len in (2..(1 + self.cw.get_width())) {
            self.add_range(&mut result, Range { point: point, dir: Dir::Right, len: len });
        }
        for len in (2..(1 + self.cw.get_height())) {
            self.add_range(&mut result, Range { point: point, dir: Dir::Down, len: len });
        }
        result
    }

    fn get_range_set(&self) -> Option<RangeSet> {
        let mut result = if self.cw.is_empty() {
            Some(self.get_ranges_for_empty())
        } else {
            self.get_word_range_set()
        };
        // TODO: Avoid ranges that would isolate clusters of empty cells in the first place.
        if result.is_none() && !self.cw.is_full() /*&& !self.cw.is_empty()*/ {
            let mut rs = RangeSet::new();
            for point in self.cw.get_smallest_empty_cluster() {
                let mut p_ranges = self.get_all_ranges(point, Dir::Right);
                p_ranges.extend(self.get_all_ranges(point, Dir::Down));
                for range in p_ranges.ranges.into_iter() {
                    if self.cw.chars(range).any(|c| c != BLOCK) {
                        self.add_range(&mut rs, range);
                    }
                }
                for range in p_ranges.backtrack_ranges.into_iter() {
                    rs.backtrack_ranges.insert(range);
                }
            }
            result = Some(rs);
            //result_range_set!(result, rs);
        }
        result
    }

    fn get_sorted_ranges(&self, range_set: HashSet<Range>) -> Vec<Range> {
        let mut ranges: Vec<Range> = range_set.into_iter().collect();
        ranges.sort_by(|r0, r1| self.range_score(r1).cmp(&self.range_score(r0)));
        ranges
    }

    // TODO: Move the stack to Author, so that calling complete_cw() repeatedly iterates through
    //       all solutions. Add a method to pop all but the first item to allow iterating through
    //       a set of substantially different solutions.
    pub fn complete_cw(&mut self) -> Crosswords {
        let mut stack = Vec::new();
        let mut bt_ranges = HashSet::new();
        let mut iter = match self.get_range_set() {
            Some(rs) => WordRangeIter::new(self.get_sorted_ranges(rs.ranges), &self.dicts),
            None => return self.cw.clone(),
        };
        'main: loop {
            while let Some((range, word)) = iter.next() {
                if self.cw.try_word(range.point, range.dir, &word) {
                    match self.get_range_set() {
                        Some(rs) => {
                            stack.push((bt_ranges, range, iter));
                            bt_ranges = rs.backtrack_ranges;
                            iter = WordRangeIter::new(
                                self.get_sorted_ranges(rs.ranges), &self.dicts[..]);
                        }
                        None => return self.cw.clone(),
                    };
                }
            }
            while let Some((prev_bt_ranges, range, prev_iter)) = stack.pop() {
                if self.verbose {
                    println!("{}", &self.cw);
                    println!("Popping {} at ({}, {}) {:?}",
                             self.cw.chars(range).collect::<String>(),
                             range.point.x, range.point.y, range.dir);
                    println!("Backtrack ranges: {:?}", bt_ranges);
                }
                self.cw.pop_word(range.point, range.dir);
                // TODO: Remember which characters not to try again.
                // TODO: Save the current range set as a "try next" hint. (Is there a way to make
                //       that work recursively ...?)
                if bt_ranges.is_empty() || bt_ranges.iter().any(
                        |r| range.intersects(r) || range.is_adjacent_to(r)) {
                    bt_ranges = prev_bt_ranges;
                    iter = prev_iter;
                    continue 'main;
                }
            }
            // Went all up the stack but found nothing? Give up and return unchanged grid.
            // TODO: Distinguish success and failure in the return value! (Option? Result?)
            return self.cw.clone();
            // TODO: If time is up or user interrupts, break.
        }
    }
}

