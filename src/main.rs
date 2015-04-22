extern crate crosswords_rs;

use crosswords_rs::{Crosswords, generate_crosswords, PrintItem};
use std::ascii::AsciiExt;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Result, Write};

fn load_dict(filename: &str) -> Result<BTreeSet<String>> {
    let file = try!(File::open(filename));
    let reader = BufReader::new(file);
    let mut dict = BTreeSet::new();
    for line in reader.lines() {
        // TODO: Use to_uppercase() once it's stable.
        if let Ok(lword) = line {
            let word = lword.to_ascii_uppercase().trim()
                           .replace("ä", "AE")
                           .replace("Ä", "AE")
                           .replace("ö", "OE")
                           .replace("Ö", "OE")
                           .replace("ü", "UE")
                           .replace("Ü", "UE")
                           .replace("ß", "SS");
            if word.chars().all(|c| c.is_alphabetic() && c.is_ascii()) && word.len() > 1 {
                dict.insert(word.clone());
            }
        }
    }
    Ok(dict)
}

const HTML_START: &'static str = r#"
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN" "http://www.w3.org/TR/html4/loose.dtd">
<html><head><style type="text/css">
.solution { font-size:25px; font-family:"monospace",monospace; }
.hints { font-size:8px; font-family:"monospace",monospace; color:light-gray }
</style><title>CW</title></head><body>
"#;
const HTML_END: &'static str = "<br></body>";

const TABLE_START: &'static str = "<table border=0 cellspacing=0 cellpadding=0>\n<tr>\n";
const TABLE_END: &'static str = "</tr></table>\n";

const BORDER_COL: &'static str = "bgcolor=#000088";
const LINE_COL: &'static str = "bgcolor=#DDDDDD";
const BLOCK_COL: &'static str = "bgcolor=#8888CC";
const LINE_SIZE: &'static str = "width=2 height=2";
const CELL_SIZE: &'static str = "width = 30 height = 30";
const SOLUTION_ATTR: &'static str = "class=solution align=center";
const HINT_ATTR: &'static str = "valign=top class=hints";

fn write_html(cw: Crosswords) -> Result<()> {
    let file = try!(File::create("cw.html"));
    let mut writer = BufWriter::new(file);
    try!(writer.write_all(HTML_START.as_bytes()));
    try!(writer.write_all(TABLE_START.as_bytes()));
    for item in cw.print_items_solution() {
        try!(writer.write_all(&match item {
            PrintItem::VertBorder(b) | PrintItem::HorizBorder(b) | PrintItem::Cross(b) =>
                format!("<td {} {}></td>\n", if b { BORDER_COL } else { LINE_COL }, LINE_SIZE),
            PrintItem::Block => 
                format!("<td {} {}></td>\n", CELL_SIZE, BLOCK_COL),
            PrintItem::Character(c) =>
                format!("<td {} {}>{}</td>\n", SOLUTION_ATTR, CELL_SIZE, c.to_string()),
            PrintItem::Number(n) =>
                format!("<td {} {}>{}</td>\n", CELL_SIZE, HINT_ATTR, n.to_string()),
            PrintItem::LineBreak => "</tr>\n<tr>".to_string(),
        }.as_bytes()))
    }
    try!(writer.write_all(TABLE_END.as_bytes()));
    try!(writer.write_all(HTML_END.as_bytes()));
    Ok(())
}

fn main() {
    let dict = load_dict("dict/top10000de.txt").unwrap();
    println!("{} words", dict.len());
    write_html(generate_crosswords(&dict, 20, 10)).unwrap();
}
