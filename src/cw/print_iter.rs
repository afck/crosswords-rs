use cw::{BLOCK, Crosswords, Dir, Point};

/// An element representing a part of a crosswords grid: an element of the cell's borders, a cell
/// and its contents or a line break. It should be converted to a textual or graphical
/// representation.
///
/// The variants specifying borders contain a boolean value specifying whether the border should be
/// displayed as thick or thin, i. e. whether it separates different words or letters of a single
/// word.
pub enum PrintItem {
    /// A vertical border.
    VertBorder(bool),
    /// A horizontal border.
    HorizBorder(bool),
    /// A crossing point of borders. It is considered thick (value `true`) if at least two of the
    /// four lines crossing here are thick.
    Cross(bool),
    /// A solid block that is left empty in the crossword's solution. It does not belong to a word.
    Block,
    /// A cell that belongs to one or two words and contains the given character. If one or two
    /// words begin in this cell, the second value will be `n`, where this is the `n`-th cell
    /// containing the beginning of a word.
    CharHint(char, Option<u32>),
    /// A line break. This follows after every row of borders or cells.
    LineBreak,
}

/// An iterator over all `PrintItem`s representing a crosswords grid.
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
                if self.cw.get_border(self.point, Dir::Down) {
                    count += 1
                }
                if self.cw.get_border(self.point, Dir::Right) {
                    count += 1
                }
                if self.cw
                       .get_border(self.point + Point::new(1, 0), Dir::Down) {
                    count += 1
                }
                if self.cw
                       .get_border(self.point + Point::new(0, 1), Dir::Right) {
                    count += 1
                }
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
                    c => {
                        PrintItem::CharHint(c,
                                            if self.cw.has_hint_at(self.point) {
                                                self.hint_count += 1;
                                                Some(self.hint_count)
                                            } else {
                                                None
                                            })
                    }
                };
            }
            self.between_chars = true;
        }
        Some(result)
    }
}
