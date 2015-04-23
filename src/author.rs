use cw::{BLOCK, Crosswords, Dir, Range};
use dict::Dict;
use point::Point;
use rand::Rng;
use std::cmp::Ordering;

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
