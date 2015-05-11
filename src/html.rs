use crosswords_rs::{Crosswords, Dir, Point, PrintItem};
use std::io::{Result, Write};

const HTML_START: &'static str = r#"
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN" "http://www.w3.org/TR/html4/loose.dtd">
<html><head><style type="text/css">
div {
    display: inline-block;
    position: relative;
}
.solution {
    font-size: 22px;
    font-family: "monospace",monospace;
    text-align: center;
    position: absolute;
    left: 0px;
    right: 0px;
    bottom: 0px;
}
.hint {
    font-size: 8px;
    font-family: "monospace",monospace;
    text-color: light-gray;
    position: absolute;
}
.gridframe {
    overflow: hidden;
    white-space: nowrap;
}
.border {
    background-color: #000088;
}
.line {
    background-color: #DDDDDD;
}
.borderwidth {
    width: 2px;
}
.borderheight {
    height: 2px;
}
.cellwidth {
    width: 30px;
}
.cellheight {
    height: 30px;
}
.blockcol {
    background-color: #8888CC;
}
</style><title>CW</title></head><body>
"#;
const HTML_END: &'static str = "<br></body>";

fn get_border_class(border: bool) -> &'static str {
    if border { "border" } else { "line" }
}

fn string_for(item: PrintItem, solution: bool) -> String {
    match item {
        PrintItem::VertBorder(b) =>
            format!(r#"<div class="borderwidth cellheight {}"></div>"#, get_border_class(b)),
        PrintItem::HorizBorder(b) =>
            format!(r#"<div class="cellwidth borderheight {}"></div>"#, get_border_class(b)),
        PrintItem::Cross(b) =>
            format!(r#"<div class="borderwidth borderheight {}"></div>"#, get_border_class(b)),
        PrintItem::Block => 
            format!(r#"<div class="cellwidth cellheight blockcol"></div>"#),
        PrintItem::CharHint(c, hint) =>
            format!(concat!(r#"<div class = "cellheight cellwidth">"#,
                            r#"<span class="hint">{}</span>"#,
                            r#"<span class="solution">{}</span>"#,
                            r#"</div>"#),
                    hint.map(|h| h.to_string()).unwrap_or("".to_string()),
                    if solution { c.to_string() } else { "&nbsp;".to_string() }),
        PrintItem::LineBreak => "<br>".to_string(),
    }
}

fn write_hints<T: Write>(writer: &mut T, cw: &Crosswords, dir: Dir) -> Result<()> {
    try!(writeln!(writer, "<p><br><b>{}:</b>&nbsp;", match dir {
        Dir::Right => "Horizontal",
        Dir::Down => "Vertical",
    }));
    let mut hint_count = 0;
    for y in 0..cw.get_height() {
        for x in 0..cw.get_width() {
            let p = Point::new(x as i32, y as i32);
            if cw.has_hint_at(p) { hint_count += 1; }
            if cw.has_hint_at_dir(p, dir) {
                let word: String = cw.chars_at(p, dir).collect();
                try!(write!(writer, "<b>{}.</b> [{}] &nbsp;", hint_count, word));
            }
        }
    }
    try!(writeln!(writer, "</p>"));
    Ok(())
}

fn write_grid<T: Write, I: Iterator<Item = PrintItem>>(writer: &mut T, items: I, solution: bool)
        -> Result<()> {
    try!(writeln!(writer, "<div class=\"gridframe\">"));
    for item in items {
        try!(writer.write_all(&string_for(item, solution).as_bytes()))
    }
    try!(writeln!(writer, "</div>"));
    Ok(())
}

pub fn write_html<T: Write>(writer: &mut T, cw: &Crosswords, solution: bool) -> Result<()> {
    try!(writer.write_all(HTML_START.as_bytes()));
    try!(write_grid(writer, cw.print_items(), solution));
    try!(write_hints(writer, &cw, Dir::Right));
    try!(write_hints(writer, &cw, Dir::Down));
    try!(writer.write_all(HTML_END.as_bytes()));
    Ok(())
}
