use cw::{BLOCK, Crosswords, Dir};
use dict::Dict;
use point::Point;
use rand::Rng;

pub struct Author<T: Rng> {
    cw: Crosswords,
    dict: Dict,
    rng: T,
}

impl<T: Rng> Author<T> {
    pub fn new(cw: Crosswords, dict: Dict, rng: T) -> Self {
        Author { cw: cw, dict: dict, rng: rng }
    }

    pub fn get_cw<'a>(&'a self) -> &'a Crosswords {
        &self.cw
    }

    fn choose_range(&mut self) -> Option<(Point, Dir, Vec<char>)> {
        let mut est = Vec::new();
        for range in self.cw.free_ranges() {
            let chars = self.cw.chars(range).collect();
            est.push((range, (self.dict.estimate_matches(&chars) * 10000_f32) as u64));
        }
        if est.is_empty() { return None; }
        est.sort_by(|&(_, ref s0), &(_, ref s1)| s0.cmp(s1));
        let r: f32 = self.rng.gen_range(0_f32, 1_f32);
        let (range, _) = est[(r * r * r * (est.len() as f32)).trunc() as usize];
        Some((range.point, range.dir, self.cw.chars(range).collect()))
    }

    fn remove_word(&mut self) {
        let n = self.rng.gen_range(0, 3);
        for _ in 0..n {
            let point = Point::new(self.rng.gen_range(0, self.cw.get_width() as i32),
                                   self.rng.gen_range(0, self.cw.get_height() as i32));
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
            if self.cw.get_char(point) == Some(BLOCK) {
                let range = self.cw.get_char(point - odp).into_iter()
                    .chain(Some(m[i]).into_iter())
                    .chain(self.cw.get_char(point + odp).into_iter()).collect();
                eval += self.dict.estimate_matches(&range);
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
            let matches = self.sort_matches(self.dict.get_matches(&range, 100), point, dir);
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

    pub fn create_cw(&mut self) {
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
