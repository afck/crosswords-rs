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
    /// One of these ranges must be filled.
    ranges: HashSet<Range>,
    /// If none of the ranges could be filled, backtracking until a word crossing or extending one
    /// of the backtrack ranges is removed will open up new possibilities.
    backtrack_ranges: HashSet<Range>,
    /// Estimated number of words that fit in one of the ranges.
    est: f32,
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

struct StackItem {
    bt_ranges: HashSet<Range>,
    iter: WordRangeIter,
    range: Range,
}

pub struct Author {
    dicts: Vec<Dict>,
    cw: Crosswords,
    min_crossing: usize,
    min_crossing_rel: f32,
    stats: WordStats,
    verbose: bool,
    stack: Vec<StackItem>,
}

// TODO: Find a saner way to do this.
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
            stack: Vec::new(),
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

    fn is_min_crossing_possible_without(&self, range: Range, filled_range: Range) -> bool {
        if range.len < 2 {
            return true;
        }
        let mut c_opts = 0;
        let odir = range.dir.other();
        let dp = range.dir.other().point();
        for p in range.points() {
            let r0 = Range { point: p, dir: odir, len: 2 };
            let r1 = Range { point: p - dp, dir: odir, len: 2 };
            // TODO: Also consider stats here?
            if !self.cw.both_borders(p, range.dir)
                    || (!r0.intersects(&filled_range) && self.cw.is_range_free(r0))
                    || (!r1.intersects(&filled_range) && self.cw.is_range_free(r1)) {
                c_opts += 1;
                c_opts < self.min_crossing || return true;
            }
        }
        false
    }

    fn would_block(&self, range: Range, point: Point) -> bool {
        let ch = self.cw.get_char(point);
        let dp = range.dir.point();
        if ch == None || !self.cw.both_borders(point, range.dir) {
            return false;
        }
        // Make sure this range doesn't isolate a cluster of empty cells.
        if ch == Some(BLOCK) && self.cw.get_boundary_iter_for(point, range).all(|r| {
            !self.cw.is_range_free(r) || (r.dir == range.dir && range.intersects(&r))
        }) {
            return true;
        }
        // Make sure it doesn't make min_crossing crossing words impossible for the perpendicular.
        if self.min_crossing_rel == 1_f32 {
            return !self.cw.is_range_free(Range {
                point: range.point - dp, dir: range.dir, len: 3
            });
        }
        let r = match ch {
            Some(BLOCK) => self.cw.get_free_range_containing(point, range.dir.other()),
            _ => self.cw.get_word_range_containing(point, range.dir.other()),
        };
        !self.is_min_crossing_possible_without(r, range)
    }

    /// Returns the maximum number of characters of a word of the given length that don't need to
    /// be connected to a crossing word.
    fn get_max_noncrossing(&self, len: usize) -> usize {
        let max_noncrossing = (1_f32 - self.min_crossing_rel) * (len as f32);
        cmp::min(max_noncrossing as usize, len - self.min_crossing)
    }

    fn add_range(&self, rs: &mut RangeSet, range: Range) {
        let p = range.point;
        let dp = range.dir.point();
        if self.would_block(range, p - dp) || self.would_block(range, p + dp * range.len) {
            return;
        }
        if !self.is_min_crossing_possible_without(self.cw.get_range_before(range), range)
                || !self.is_min_crossing_possible_without(self.cw.get_range_after(range), range) {
            return
        }
        let est = self.stats.estimate_matches(&self.cw.chars(range).collect());
        if est != 0_f32 && rs.ranges.insert(range) {
            rs.est += est;
        }
    }

    /// Returns a range set containing all free ranges with the given point in the given direction.
    /// The backtrack range extends one field past the longest of these ranges.
    fn get_all_ranges(&self, point: Point, dir: Dir) -> RangeSet {
        let mut rs = RangeSet::new();
        let range = self.cw.get_free_range_containing(point, dir);
        let dp = dir.point();
        let t = (point.x - range.point.x + point.y - range.point.y) as usize;
        for i in 0..(t + 1) {
            for j in t..range.len {
                if j - i > 0 {
                    self.add_range(&mut rs, Range {
                        point: range.point + dp * i,
                        dir: dir,
                        len: j - i + 1,
                    });
                }
            }
        }
        rs.backtrack_ranges.insert(range);
        rs
    }

    fn get_word_range_set(&self) -> Option<RangeSet> {
        let mut result = None;
        for range in self.cw.word_ranges() {
            let odir = range.dir.other();
            let candidate_points: Vec<Point> = range.points().filter(|&p| {
                self.cw.both_borders(p, odir)
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

    // TODO: Constructing range sets should abort as soon as the estimate surpasses the lowest one
    //       found so far.
    // TODO: Prefer ranges that are close to the previous one for more efficient backtracking?
    fn get_range_set(&self) -> Option<RangeSet> {
        let mut result = if self.cw.is_empty() {
            Some(self.get_ranges_for_empty())
        } else {
            self.get_word_range_set()
        };
        // TODO: Avoid ranges that would isolate clusters of empty cells in the first place. (done)
        //       This could be combined with finding empty clusters, as we're only interested in
        //       their boundaries anyway.
        if /*result.is_none() &&*/ !self.cw.is_full() && !self.cw.is_empty() {
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
            //result = Some(rs);
            result_range_set!(result, rs);
        }
        result
    }

    fn get_sorted_ranges(&self, range_set: HashSet<Range>) -> Vec<Range> {
        let mut ranges: Vec<Range> = range_set.into_iter().collect();
        ranges.sort_by(|r0, r1| self.range_score(r1).cmp(&self.range_score(r0)));
        ranges
    }

    fn pop(&mut self) -> Option<StackItem> {
        let opt_item = self.stack.pop();
        if let Some(ref item) = opt_item {
            let range = item.range;
            if self.verbose {
                println!("{}", &self.cw);
                println!("Popping {} at ({}, {}) {:?}",
                         self.cw.chars(range).collect::<String>(),
                         range.point.x, range.point.y, range.dir);
            }
            self.cw.pop_word(range.point, range.dir);
        }
        opt_item
    }

    pub fn pop_to_n_words(&mut self, n: usize) {
        while self.stack.len() > n {
            self.pop();
        }
    }

    fn range_meets(range: &Range, bt_ranges: &HashSet<Range>) -> bool {
        bt_ranges.is_empty()
            || bt_ranges.iter().any(|r| range.intersects(r) || range.is_adjacent_to(r))
    }

    // TODO: Calling complete_cw() repeatedly should iterate through
    //       all solutions. Add a method to pop all but the first item to allow iterating through
    //       a set of substantially different solutions.
    pub fn complete_cw(&mut self) -> Option<Crosswords> {
        let mut bt_ranges = HashSet::new();
        let mut iter = match self.pop() {
            Some(item) => item.iter, // Drop bt_ranges, as iter was successful!.
            None => match self.get_range_set() {
                Some(rs) => WordRangeIter::new(self.get_sorted_ranges(rs.ranges)),
                None => return None,
            },
        };
        'main: loop {
            while let Some((range, word)) = iter.next(&self.dicts) {
                if self.cw.try_word(range.point, range.dir, &word) {
                    // TODO: Check for isolated empty areas.
                    self.stack.push(StackItem {
                        bt_ranges: bt_ranges,
                        range: range,
                        iter: iter,
                    });
                    match self.get_range_set() {
                        Some(rs) => {
                            bt_ranges = rs.backtrack_ranges;
                            iter = WordRangeIter::new(self.get_sorted_ranges(rs.ranges));
                        }
                        None => return Some(self.cw.clone()),
                    };
                }
            }
            while let Some(item) = self.pop() {
                // TODO: Remember which characters not to try again.
                // TODO: Save the current range set as a "try next" hint. (Is there a way to make
                //       that work recursively ...?)
                if Author::range_meets(&item.range, &bt_ranges) {
                    bt_ranges = item.bt_ranges;
                    iter = item.iter;
                    continue 'main;
                }
            }
            // Went all up the stack but found nothing? Give up.
            return None;
            // TODO: If time is up or user interrupts, break.
        }
    }
}

