// TODO: Use the regex! macro once that feature is stable.
// TODO: Consider using external software for that, e. g.:
//       https://github.com/bwbaugh/wikipedia-extractor/blob/master/WikiExtractor.py
//       It might be best to move the whole hint generation out of crosswords-rs. Instead, you
//       could specify a hint generation program at the command line.
use regex::Regex;
use std::ascii::AsciiExt;
use std::collections::HashMap;

use std::io::Read;

use hyper::Client;

fn replace_all<'a>(text: String, replacements: Vec<(&'a str, &'a str)>) -> String {
    let mut new_text = text;
    for (re, repl) in replacements.into_iter() {
        let temp_text = Regex::new(re).unwrap().replace_all(&new_text, repl);
        new_text = temp_text;
    }
    new_text
}

fn get_hint_from_article(article: String, word: &str, lang: &str) -> String {
    let clean_article = replace_all(article, vec!(
        // Remove quotations.
        (r#"<ref>.*</ref>"#, ""),
        // Remove monospace.
        (r#"<tt>.*</tt>"#, ""),
        // Remove Image and File references:
        (r#"\[\[(Image|File).*\n"#, ""),
        // Replace the convert template using the first unit.
        (r#"\{\{((?i)convert)\|(?P<number>[^\|\}]*)\|(?P<unit>[^\|\}]*)[^\}]*\}\}"#,
            "$number $unit"),
        // Replace other templates with their first parameter ...
        (r#"\{\{[^\|\}]*\|(?P<firstparam>[^\|\}]*)[^\}]*\}\}"#, "$firstparam"),
        // ... or remove them. TODO: Sometimes the template name may be more appropriate.
        (r#"\{\{([^\}]*\|)?[^\|\}]*\}\}"#, ""),
        // Replace links with their link text.
        (r#"\[\[([^\]]*\|)?(?P<link>[^\|\]]*)\]\]"#, "$link"),
        // Display bold text as plain text.
        (r#"'''(?P<bold>[^']*)'''"#, "$bold"))).trim().to_string();
    let descr_init = match lang {
        "de" => " ist | bezeichnet | war | sind | waren ",
        "en" => " is | are | was | were ",
        _ => unimplemented!(),
    };
    let word_re = format!(r#"((?i){})"#, word);
    // Disambiguations:
    let ex_re1 = Regex::new(&format!(
            r#"{}\S* (or [^\.\n]* )?may refer to:\n(\s*((=|;).*|.*:)?\n)*\*(?P<excerpt>.*)\n"#,
            word_re))
        .unwrap();
    // Sentences starting with "<word> is ...":
    let ex_re0 = Regex::new(
            &format!(r#"({}(\([^\)]*\))?({})(?P<excerpt>[^\."\n]*)(\.|"|\n))"#,
                     word_re, descr_init))
        .unwrap();
    // Any sentence containing the word.
    let ex_re2 = Regex::new(
        &format!(r#"(\n|\*|\. )\s*(?P<excerpt>[^\.\n]*{}[^\.\n\*]*(\.|\n))"#, word_re)).unwrap();
    // If all else fails, any sentence.
    let ex_re3 = Regex::new(r#"(\n|\. )\s*(?P<excerpt>[^\.\n]+(\.|\n))"#).unwrap();
    let excerpt = match ex_re0.captures(&clean_article.clone())
            .or(ex_re1.captures(&clean_article.clone()))
            .or(ex_re2.captures(&clean_article.clone()))
            .or(ex_re3.captures(&clean_article.clone())) {
        Some(captures) => captures.name("excerpt").unwrap().to_string(),
        None => clean_article,
    };
    replace_all(excerpt, vec!(
        // Replace the word from the crosswords with ellipses.
        (&format!(r#"(?i){}"#, &word), "..."),
        // Replace any sequence of whitespace with a single space.
        (r#"\s+"#, " "))).trim().to_string()
}

fn download_from(url: String) -> String {
    let client = Client::new();
    let mut res = client.get(&url[..]).send().unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    body
}

fn download_article(word: &String, lang: &String) -> String {
    let mut cased_word = String::new();
    cased_word.extend(word[..1].chars());
    cased_word.extend(word[1..].to_ascii_lowercase().chars());
    let url = format!("http://{}.wikipedia.org/w/index.php?title={}&action=raw", lang, cased_word);
    let body = download_from(url);
    // TODO: Check whether the redirection is just because of capitalization. Otherwise ... ??
    if let Some(captures) =
        Regex::new(r#"^#((?i)REDIRECT|WEITERLEITUNG)\s*\[\[(?P<redir>[^\]]*)\]\]"#)
            .unwrap().captures(&body) {
        let url = format!("http://{}.wikipedia.org/w/index.php?title={}&action=raw",
                          lang, captures.name("redir").unwrap().replace(" ", "_"));
        return download_from(url);
    }
    body
}

fn get_hint(word: &String, lang: &String) -> String {
    let article = download_article(word, lang);
    // TODO: Remove markup. Or better: Find some external software that removes markup.
    // TODO: Escape HTML
    // TODO: Handle disambiguations.
    // TODO: Do something (like, an anagram?) if the article doesn't exist.
    // TODO: Handle all errors without panic!
    // TODO: Restore umlauts.
    get_hint_from_article(article, word, lang)
}

pub fn get_hints<T: Iterator<Item = String>>(words: T, lang: String) -> HashMap<String, String> {
    words.map(|word| {
        let hint = get_hint(&word, &lang);
        (word, hint)
    }).collect()
}

#[test]
fn test_get_hint_from_article() {
    let article = concat!(r#"
        {{Infobox Software
        | Name                              = Servo<!-- Nur falls abweichend vom Artikelnamen -->
        | Logo                              = 
        | Screenshot                        = [[Datei:Servo rendering de wikipedia.png|320px]]
        | Website                           = [http://github.com/mozilla/servo]
        }}

        "#,
        r#"'''Servo''' ist eine [[Layout-Engine]], welche von [[Mozilla]] und '''Samsung''' "#,
        r#"entwickelt wird.<ref>[http://arstechnica.com] (englisch) – Artikel vom "#,
        r#"{{Datum|3|4|2013}} <small></ref> Der Prototyp zielt darauf ab, eine hochparallele "#,
        r#"Umgebung zu erschaffen."#).to_string();
    let description = r#"eine Layout-Engine, welche von Mozilla und Samsung entwickelt wird"#
        .to_string();
    assert_eq!(description, get_hint_from_article(article, "Servo", "de"));
    let convert = r#"distance of {{convert|2,900|km|mi}}"#.to_string();
    assert_eq!(r#"distance of 2,900 km"#.to_string(), get_hint_from_article(convert, "Foo", "en"));
}
