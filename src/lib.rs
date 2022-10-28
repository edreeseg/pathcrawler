use reqwest::Url;
use scraper::element_ref::Text;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::io::Error as IoErr;
use std::io::Read;
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    IO { url: String, e: IoErr },
    Fetch { url: String, e: reqwest::Error },
}

pub type CrawlerResult<T> = std::result::Result<T, Error>;

impl<S: AsRef<str>> From<(S, IoErr)> for Error {
    fn from((url, e): (S, IoErr)) -> Self {
        Error::IO {
            url: url.as_ref().to_string(),
            e: e,
        }
    }
}

impl<S: AsRef<str>> From<(S, reqwest::Error)> for Error {
    fn from((url, e): (S, reqwest::Error)) -> Self {
        Error::Fetch {
            url: url.as_ref().to_string(),
            e: e,
        }
    }
}

pub fn fetch_url(client: &reqwest::blocking::Client, url: &str) -> CrawlerResult<String> {
    let mut res = client.get(url).send().map_err(|e| Error::Fetch {
        url: url.to_string(),
        e,
    })?;
    // println!("Status for {}: {}", url, res.status());

    let mut body = String::new();
    res.read_to_string(&mut body).map_err(|e| Error::IO {
        url: url.to_string(),
        e,
    })?;
    Ok(body)
}

// struct Spell {
//     name: &str,
//     description: &str,
//     school: &str,
//     subschool: &str,
//     descriptors: Vec<&str>,
// }

// pub fn parse_class_spells() -> Vec<Spell> {

// }

fn parse_spell_metadata(mut iter: Text) -> () {
    while let Some(text_value) = iter.next() {
        println!("Metadata: {}", text_value);
    }
    ()
}

pub fn parse_spell(html: &str) -> () {
    let document = Html::parse_fragment(html);
    let name_selector = Selector::parse("#article-content > h1").expect("Basic child boi");
    let spell_name = document.select(&name_selector).next().unwrap().text().next().unwrap();
    println!("Spell Name: {}", spell_name);
    let spell_metadata_selector = Selector::parse("#article-content > p:first-of-type").unwrap();
    parse_spell_metadata(document.select(&spell_metadata_selector).next().unwrap().text());
    // let (spell_school, spell_subschool, spell_descriptors) = parse_spell_metadata(document.select(&spell_metadata_selector).next()?.text());
    // let document = Document::from(html);
    // let spell_name = document
    //     .find(Child(Class("article-content"), Name("h1")))
    //     .into_selection()
    //     .first();

    // if let Some(name) = spell_name {
    //     println!("Spell name: {}", name.text());
    // }
    ()
}

pub fn get_links_from_html(html: &str) -> HashSet<String> {
    let document = Html::parse_fragment(html);
    let selector = Selector::parse("a").expect("Not Not Boi");
    document
        .select(&selector)
        .filter_map(|n| n.value().attr("href"))
        .filter(has_no_extension)
        .filter_map(normalize_url)
        .collect::<HashSet<String>>()
}

fn has_no_extension(url: &&str) -> bool {
    Path::new(url).extension().is_none()
}

fn normalize_url(url: &str) -> Option<String> {
    let new_url = Url::parse(url);
    let url_sans_hash: Vec<&str> = url.split("#").collect();
    let url_sans_hash = url_sans_hash[0];
    match new_url {
        Ok(new_url) => {
            if let Some("www.d20pfsrd.com") = new_url.host_str() {
                Some(url_sans_hash.to_string())
            } else {
                None
            }
        }
        Err(_e) => {
            if url.starts_with('/') {
                Some(format!("https://www.d20pfsrd.com{}", url_sans_hash))
            } else {
                None
            }
        }
    }
}
