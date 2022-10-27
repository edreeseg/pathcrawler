use reqwest::Url;
use select::document::Document;
use select::predicate::{And, Class, Descendant, Name, Not, Or};
use std::collections::HashSet;
use std::io::Read;
use std::path::Path;

pub fn fetch_url(client: &reqwest::blocking::Client, url: &str) -> String {
    let mut res = client.get(url).send().unwrap();
    // println!("Status for {}: {}", url, res.status());

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    body
}

pub fn get_links_from_html(html: &str) -> HashSet<String> {
    Document::from(html)
        .find(And(
            Name("a"),
            Not(Descendant(
                Or(Name("nav"), Or(Class("footer-nav"), Name("footer"))),
                Name("a"),
            )),
        ))
        .filter_map(|n| n.attr("href"))
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
