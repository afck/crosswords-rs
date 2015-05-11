use cw::{BLOCK, Crosswords, Dir, Point};

pub enum PrintItem {
    VertBorder(bool),
    HorizBorder(bool),
    Cross(bool),
    Block,
    CharHint(char, Option<u32>),
    LineBreak,
}

pub struct PrintIter<'a> {
    point: Point,
    between_lines: bool,
    between_chars: bool,
    cw: &'a Crosswords,
    hint_count: u32,
}

impl<'a> PrintIter<'a> {
    pub fn new(cw: &'a Crosswords) -> Self {
        PrintIter {
            point: Point::new(-1, -1),
            between_lines: true,
            between_chars: true,
            cw: cw,
            hint_count: 0,
        }
    }
}

impl<'a> Iterator for PrintIter<'a> {
    type Item = PrintItem;
    fn next(&mut self) -> Option<PrintItem> {
        if self.point.y >= self.cw.height as i32 {
            return None;
        }
        let result;
        if self.point.x >= self.cw.width as i32 {
            result = PrintItem::LineBreak;
            self.point.x = -1;
            if self.between_lines {
                self.point.y += 1;
            }
            self.between_chars = true;
            self.between_lines = !self.between_lines;
        } else if self.between_chars {
            if self.between_lines {
                let mut count = 0;
                if self.cw.get_border(self.point, Dir::Down) { count += 1 }
                if self.cw.get_border(self.point, Dir::Right) { count += 1 }
                if self.cw.get_border(self.point + Point::new(1, 0), Dir::Down) { count += 1 }
                if self.cw.get_border(self.point + Point::new(0, 1), Dir::Right) { count += 1 }
                result = PrintItem::Cross(count > 1);
            } else {
                result = PrintItem::VertBorder(self.cw.get_border(self.point, Dir::Right));
            }
            self.point.x += 1;
            self.between_chars = false;
        } else {
            if self.between_lines {
                result = PrintItem::HorizBorder(self.cw.get_border(self.point, Dir::Down));
            } else {
                result = match self.cw.get_char(self.point).unwrap() {
                    BLOCK => PrintItem::Block,
                    c => PrintItem::CharHint(c, if self.cw.has_hint_at(self.point) {
                            self.hint_count += 1;
                            Some(self.hint_count)
                        } else {
                            None
                        }),
                };
            }
            self.between_chars = true;
        }
        Some(result)
    }
}

