use cw::{Crosswords, Dir, Point, PrintItem};
use std::collections::HashMap;
use std::io::{Result, Write};

const CSS: &'static str = r#"
.solution {
    font: 22px monospace;
    text-align: center;
    position: absolute;
    left: 0px;
    right: 0px;
    bottom: 0px;
}
.hint {
    font: 8px monospace;
    color: Gray;
    position: absolute;
}
.row {
    overflow: hidden;
    float: left;
}
.row > div:nth-child(even) { width: 30px; }
.row > div:nth-child(odd) { width: 2px; }
.row > div {
    float: left;
    position: relative;
}
.low { height: 2px; }
.high { height: 30px; }
.dark { background-color: DarkBlue; }
.light { background-color: LightGray; }
.blockcol { background-color: DarkBlue; }
"#;

fn get_border_class(border: bool) -> &'static str {
    if border { "dark" } else { "light" }
}

fn string_for(item: &PrintItem, solution: bool) -> String {
    match *item {
        PrintItem::HorizBorder(b) |
        PrintItem::Cross(b) => format!(r#"<div class="low {}"></div>"#, get_border_class(b)),
        PrintItem::VertBorder(b) => format!(r#"<div class="high {}"></div>"#, get_border_class(b)),
        PrintItem::Block => r#"<div class="high blockcol"></div>"#.to_string(),
        PrintItem::CharHint(c, hint) => {
            format!(concat!(r#"<div class = "high">"#,
                            r#"<span class="hint">{}</span>"#,
                            r#"<span class="solution">{}</span>"#,
                            r#"</div>"#),
                    hint.map(|h| h.to_string())
                        .unwrap_or_else(|| "".to_owned()),
                    if solution {
                        c.to_string()
                    } else {
                        "&nbsp;".to_owned()
                    })
        }
        PrintItem::LineBreak => r#"</div><div class="row">"#.to_owned(),
    }
}

fn write_grid<T: Write, I: Iterator<Item = PrintItem>>(writer: &mut T,
                                                       items: I,
                                                       solution: bool)
                                                       -> Result<()> {
    try!(writeln!(writer, r#"<div class="row">"#));
    for item in items {
        try!(writer.write_all(string_for(&item, solution).as_bytes()))
    }
    try!(writeln!(writer, "</div>"));
    Ok(())
}

fn write_hints<T: Write>(writer: &mut T,
                         cw: &Crosswords,
                         dir: Dir,
                         hint_text: &HashMap<String, String>)
                         -> Result<()> {
    try!(writeln!(writer,
                  "<p><br><b>{}:</b>&nbsp;",
                  match dir {
                      Dir::Right => "Horizontal",
                      Dir::Down => "Vertical",
                  }));
    let mut hint_count = 0;
    for y in 0..cw.get_height() {
        for x in 0..cw.get_width() {
            let p = Point::new(x as i32, y as i32);
            if cw.has_hint_at(p) {
                hint_count += 1;
            }
            if cw.has_hint_at_dir(p, dir) {
                let word: String = cw.chars_at(p, dir).collect();
                let hint = hint_text
                    .get(&word)
                    .cloned()
                    .unwrap_or_else(|| format!("[{}]", word));
                try!(write!(writer, "<b>{}.</b> {} &nbsp;", hint_count, hint));
            }
        }
    }
    try!(writeln!(writer, "</p>"));
    Ok(())
}

/// Write the crosswords to the given writer as an HTML page.
pub fn write_html<T: Write>(writer: &mut T,
                            cw: &Crosswords,
                            solution: bool,
                            hint_text: &HashMap<String, String>)
                            -> Result<()> {
    try!(writeln!(writer, r#"<!doctype html>"#));
    try!(writeln!(writer, r#"<head>"#));
    try!(writeln!(writer, r#"<meta charset="utf-8" />"#));
    try!(writeln!(writer, r#"<style type="text/css">{}</style>"#, CSS));
    try!(writeln!(writer, r#"<title>Crosswords</title>"#));
    try!(writeln!(writer, r#"</head><body>"#));
    try!(writeln!(writer,
                  r#"<div style="width: {}px">"#,
                  cw.get_width() * 32 + 2));
    try!(write_grid(writer, cw.print_items(), solution));
    try!(writeln!(writer, r#"</div><br><div style="clear: both"></div>"#));
    try!(write_hints(writer, cw, Dir::Right, hint_text));
    try!(write_hints(writer, cw, Dir::Down, hint_text));
    try!(writeln!(writer, "<br></body>"));
    Ok(())
}
