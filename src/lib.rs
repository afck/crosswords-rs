extern crate rand;

mod cw;
mod point;
mod dict;

use cw::{BLOCK, Crosswords, Dir};
use dict::Dict;
use point::Point;
use rand::Rng;
use std::collections::BTreeSet;

struct Author<T: Rng> {
    cw: Crosswords,
    dict: Dict,
    rng: T,
}

impl<T: Rng> Author<T> {
    fn choose_range(&mut self) -> Option<(Point, Dir, Vec<char>)> {
        let mut est = Vec::new();
        for (point, dir, range) in self.cw.get_ranges().into_iter() {
            est.push((point, dir, (self.dict.estimate_matches(&range) * 10000_f32) as u64));
        }
        if est.is_empty() { return None; }
        est.sort_by(|&(_, _, ref s0), &(_, _, ref s1)| s0.cmp(s1));
        let r: f32 = self.rng.gen_range(0_f32, 1_f32);
        let (point, dir, _) = est[(r * r * r * (est.len() as f32)).trunc() as usize];
        Some((point, dir, self.cw.get_range(point, dir)))
    }

    fn remove_word(&mut self) {
        let n = self.rng.gen_range(0, 3);
        for _ in 0..n {
            let point = Point::new(self.rng.gen_range(0, 20), self.rng.gen_range(0, 10));
            let dir = match self.rng.gen_range(0, 2) { 0 => Dir::Right, _ => Dir::Down };
            self.cw.pop_word(point, dir);
        }
    }

    fn eval_match(&self, m: &Vec<char>, mut point: Point, dir: Dir) -> u64 {
        let mut eval = 0_f32; //m.len() as f32;
        let dp = dir.point();
        let odir = dir.other();
        let odp = odir.point();
        for i in 0..m.len() {
            if self.cw.get_char(point) == Some(BLOCK)
                    && self.cw.get_border(point - odp, odir)
                    && self.cw.get_border(point, odir) {
                let range = self.cw.get_char(point - odp).into_iter()
                    .chain(Some(m[i]).into_iter())
                    .chain(self.cw.get_char(point + odp).into_iter()).collect();
                eval += 10.0 * self.dict.estimate_matches(&range);
            }
            point = point + dp;
        }
        (100000_f32 * eval) as u64
    }

    fn sort_matches(&self, matches: Vec<Vec<char>>, point: Point, dir: Dir) -> Vec<Vec<char>> {
        let mut evaluated: Vec<_> =
            matches.into_iter().map(|m| (m.clone(), self.eval_match(&m, point, dir))).collect();
        evaluated.sort_by(|&(_, ref e0), &(_, ref e1)| e1.cmp(e0));
        evaluated.into_iter().map(|(m, _)| m).collect()
    }

    fn improve_cw(&mut self) {
        if let Some((point, dir, range)) = self.choose_range() {
            let matches = self.sort_matches(self.dict.get_matches(&range), point, dir);
            if matches.is_empty() {
                self.remove_word();
            } else {
                for word in matches.into_iter() {
                    if self.cw.try_word(point, dir, &word) {
                        break;
                    }
                }
            }
        }
    }
}

pub fn generate_crosswords(words: &BTreeSet<String>, width: usize, height: usize) {
    let mut author = Author {
        cw: Crosswords::new(width, height),
        dict: Dict::with_words(words.iter().cloned()),
        rng: rand::thread_rng(),
    };
    for _ in 0..1000 {
        author.improve_cw();
    }
    println!("{:?}", author.cw);
}

#[test]
fn it_works() {
}
