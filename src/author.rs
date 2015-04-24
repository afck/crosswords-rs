use cw::{BLOCK, Crosswords, Dir, Range};
use dict::Dict;
use point::Point;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::slice;

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
            range_i: 0,
            dicts: dicts,
            dict_i: 0,
            word_i: 0,
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
            if self.range_i < self.ranges.len() {
                self.range_i = 0;
                self.dict_i += 1;
            }
        }
    }

    fn next(&mut self) -> Option<(Range, Vec<char>)> {
        let word = self.get_word();
        if word.is_some() {
            self.advance();
        }
        word.map(|w| (self.ranges[self.range_i], w))
    }

    fn into_ranges(self) -> Vec<Range> {
        self.ranges
    }
}

pub struct NewAuthor {
    min_crossing: usize,
    dicts: Vec<Dict>,
}

impl NewAuthor {
    pub fn new(words: &Vec<HashSet<String>>) -> NewAuthor {
        NewAuthor {
            min_crossing: 1,
            dicts: words.iter().map(|s| Dict::new(s.iter().cloned())).collect(),
        }
    }

    fn get_ranges(&self, cw: &Crosswords/*, freqs: &NGramFrequencies*/) -> Option<Vec<Range>> {
        let mut ranges = Vec::new();
        if cw.is_empty() {
            let point = Point::new(0, 0);
            ranges.extend((2..cw.get_width()).map(|len| Range {
                point: point,
                dir: Dir::Right,
                len: len,
            }));
            ranges.extend((2..cw.get_height()).map(|len| Range {
                point: point,
                dir: Dir::Down,
                len: len,
            }));
            // No words? Start in the top left corner.
        } else {
            // Words crossing < min_cross others? Go through them.
            // Optional: Words crossing < min_cross_percent % others? Go through their letters.
        }
        // Optional: Constrained empty cells?
        // None of these? If there are empty cells left, go through the boundary of the filled space.
        if ranges.is_empty() {
            // Puzzle complete? Return None
            None
        } else {
            // TODO: Better sorting - most favorable ranges first.
            ranges.sort_by(|r0, r1| r1.len.cmp(&r0.len));
            Some(ranges)
        }
    }

    pub fn complete_cw<F, T>(&self, init_cw: &Crosswords, eval: F, rng: &mut T)
            -> Crosswords where F: Fn(&Crosswords) -> i32, T: Rng {
        // TODO: let freqs = get_n_gram_frequencies(dicts);
        let mut cw = init_cw.clone();
        // TODO: Don't stop at success: Backtrack and compare with other solutions.
        //let (mut best_cw, mut best_score = (None, i32::MIN));
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
                    let word = cw.pop_word(range.point, range.dir);
                    if ranges.iter().any(|r| range.intersects(r)) {
                        continue 'main;
                    }
                }
                // Went all up the stack but found nothing? Give up.
                return cw;
            }
            // If time's up, break.
        }
    }
}

// TODO: Remove old implementation:

fn choose_lowest<S: Copy, SI: Iterator<Item = (S, f32)>, T: Rng>(rng: &mut T, si: SI) -> Option<S> {
    let mut est: Vec<(S, f32)> = si.collect();
    if est.is_empty() { return None; }
    est.sort_by(|&(_, s0), &(_, s1)| s0.partial_cmp(&s1).unwrap_or(Ordering::Equal));
    let r = rng.gen_range(0_f32, 1_f32);
    let (s, _) = est[(r * r * r * (est.len() as f32)).trunc() as usize];
    Some(s)
}

fn range_connectedness(cw: &Crosswords, range: &Range) -> f32 {
    let odir = range.dir.other();
    let (dp, odp) = (range.dir.point(), odir.point());
    let count = (0..range.len).filter(|&i| {
        let p = range.point + dp * i;
        !cw.get_border(p, odir) || !cw.get_border(p - odp, odir)
    }).count();
    (count as f32) / (range.len as f32)
}

#[derive(Clone)]
struct Match {
    word: Vec<char>,
    point: Point,
    dir: Dir,
}

pub struct Author<T: Rng> {
    cw: Crosswords,
    dict: Dict,
    favorites: Vec<Vec<char>>,
    rng: T,
}

impl<T: Rng> Author<T> {
    pub fn new(cw: Crosswords, dict: Dict, favorites: Vec<Vec<char>>, rng: T) -> Self {
        Author { cw: cw, dict: dict, favorites: favorites, rng: rng }
    }

    pub fn get_cw<'a>(&'a self) -> &'a Crosswords {
        &self.cw
    }

    fn choose_range(&mut self) -> Option<Range> {
        let (cw, dict) = (&self.cw, &self.dict);
        choose_lowest(&mut self.rng, cw.free_ranges().filter_map(|range| {
            if cw.chars(range).any(|c| c == BLOCK) {
                let chars = cw.chars(range);
                Some((range, dict.estimate_matches(chars)))
            } else { None }
        }))
    }

    fn remove_word(&mut self) {
        let cw = &mut self.cw;
        if let Some(range) = choose_lowest(&mut self.rng, cw.words().map(|range|
                (range, -range_connectedness(&cw, &range)))) {
            cw.pop_word(range.point, range.dir);
        }
    }

    fn eval_match(&self, m: &Match) -> u64 {
        let mut eval = 0_f32; //m.len() as f32;
        let dp = m.dir.point();
        let odir = m.dir.other();
        let odp = odir.point();
        let mut point = m.point;
        for i in 0..m.word.len() {
            if self.cw.get_char(m.point) == Some(BLOCK) {
                let range = self.cw.get_char(m.point - odp).into_iter()
                    .chain(Some(m.word[i]).into_iter())
                    .chain(self.cw.get_char(m.point + odp).into_iter());
                eval += self.dict.estimate_matches(range);
            }
            point = point + dp;
        }
        let range = Range { point: m.point, dir: m.dir, len: m.word.len() };
        eval += range_connectedness(&self.cw, &range);
        (100000_f32 * eval) as u64
    }

    fn sort_matches(&self, matches: Vec<Match>) -> Vec<Match> {
        let mut evaluated: Vec<_> =
            matches.into_iter().map(|m| {
                let e = self.eval_match(&m);
                (m, e)
            }).collect();
        evaluated.sort_by(|&(_, ref e0), &(_, ref e1)| e1.cmp(e0));
        evaluated.into_iter().map(|(m, _)| m).collect()
    }

    fn improve_cw(&mut self) {
        if let Some(range) = self.choose_range() {
            let (point, dir) = (range.point, range.dir);
            let ms = self.dict.get_matches(&self.cw.chars(range).collect(), 100).into_iter().map(|word|
                Match { word: word, point: point, dir: dir }).collect();
            let matches = self.sort_matches(ms);
            if matches.is_empty() {
                self.remove_word();
            } else {
                for m in matches.into_iter() {
                    if self.cw.try_word(m.point, m.dir, &m.word) {
                        break;
                    }
                }
            }
        } else {
            self.remove_word();
            self.finalize_cw();
        }
    }

    fn insert_favorites(&mut self) {
        self.rng.shuffle(&mut self.favorites);
        for word in self.favorites.iter() {
            let mut matches = Vec::new();
            for x in 0..(self.cw.get_width() as i32) {
                for y in 0..(self.cw.get_height() as i32) {
                    let point = Point::new(x, y);
                    if self.cw.is_word_allowed(point, Dir::Right, word) {
                        matches.push(Match { word: word.clone(), point: point, dir: Dir::Right });
                    }
                    if self.cw.is_word_allowed(point, Dir::Down, word) {
                        matches.push(Match { word: word.clone(), point: point, dir: Dir::Down });
                    }
                }
            }
            for m in self.sort_matches(matches).into_iter() {
                if self.cw.try_word(m.point, m.dir, &m.word) {
                    break;
                }
            }
        }
    }

    pub fn create_cw(&mut self) {
        self.insert_favorites();
        for _ in 0..1000 {
            self.improve_cw();
        }
    }

    fn finalize_range(&mut self, point: Point, len: i32, dir: Dir) {
        let dp = dir.point(); 
        let range: Vec<_> = (0..len).map(|i| self.cw.get_char(point + dp * i).unwrap()).collect();
        for i in 0..(len - 1) {
            if self.cw.get_border(point + dp * (i - 1), dir) {
                for j in (i + 1)..len {
                    if self.cw.get_border(point + dp * j, dir) {
                        for word in self.dict.find_matches(&range[(i as usize)..].to_vec(), 3) {
                            if self.cw.try_word(point + dp * i, dir, &word) {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn finalize_cw(&mut self) {
        let width = self.cw.get_width() as i32;
        let height = self.cw.get_height() as i32;
        for x in 0..width {
            self.finalize_range(Point::new(x, 0), height, Dir::Down);
        }
        for y in 0..height {
            self.finalize_range(Point::new(0, y), width, Dir::Right);
        }
    }
}
