use quick_xml::events::Event as XMLEvent;
use quick_xml::reader::Reader as XMLReader;
use reqwest::Url;

use scraper::{Html, Selector};
use std::collections::HashSet;
use std::fs;
use std::io::Error as IoErr;
use std::io::Read;
use std::io::Write;
use std::path::Path;

// TODO: Sanitize

pub fn write_file(path: &str, content: &str) -> std::io::Result<()> {
    let path_result = Path::new(path);
    if let Some(prefix) = path_result.parent() {
        fs::create_dir_all(prefix).expect("Failed to create dir in write_file");
    }
    let mut file = fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;
    writeln!(file, "{}", content)?;
    Ok(())
}

pub fn replace_file(path: &str, content: &str) -> std::io::Result<()> {
    let path_result = Path::new(path);
    if let Some(prefix) = path_result.parent() {
        fs::create_dir_all(prefix).expect("Failed to create dir in replace_file");
    }
    let mut file = if path_result.exists() {
        fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?
    } else {
        fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)?
    };
    writeln!(file, "{}", content)?;
    Ok(())
}

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

pub fn get_links_from_xml(xml: &str) -> HashSet<String> {
    let mut reader = XMLReader::from_str(xml);
    reader.trim_text(true);

    let mut urls = Vec::new();
    let mut buf: Vec<u8> = Vec::new();
    let mut reading_loc = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!(
                "Error reading XML page at position {} {:#?}",
                reader.buffer_position(),
                e
            ),
            Ok(XMLEvent::Eof) => break,
            Ok(XMLEvent::Start(e)) if e.name().as_ref() == b"loc" => reading_loc = true,
            Ok(XMLEvent::End(e)) if e.name().as_ref() == b"loc" => reading_loc = false,
            Ok(XMLEvent::Text(e)) if reading_loc => urls.push(e.unescape().unwrap().into_owned()),
            _ => (),
        }
    }

    let urls = urls
        .into_iter()
        .filter(|s| allowed_extension(&&s[..]))
        .filter_map(|s| normalize_url(&&s[..]))
        .collect::<HashSet<String>>();

    // let document = Html::parse_fragment(xml);
    // let anchor_tags: Vec<&str> = document.tree.values().fold(vec![], |mut a, b| {
    //     if let Node::Element(el) = b {
    //         let tag_name = &(*el.name.local);
    //         if tag_name.eq("loc") {
    //             let text = b.as_text();
    //             if text.is_some() {
    //                 let text = &(**text.unwrap());
    //                 println!("Text: {}", text);
    //             }
    //         }
    //     }
    //     a
    // });
    // println!("Anchor tags: {:#?}", anchor_tags);
    urls
}

/* ['', '<div class="content-right-videobox hidden-xs hidden-sm">', '<div style="width:180px;max-width:180px;margin-lef…n-right:auto;" class="ogn-npa-container videoad">', '<div class="ogn-npa ogn-npa-video" id="nitropay-d20pfsrd-video"></div>', '</div>\x3Cscript type="text/javascript">', 'ognCreateVideoAdSpot("nitropay-d20pfsrd-video");', '\x3C/script>    </div>', '<div class="breadcrumbs">', '<a parentid="52278" href="https://www.d20pfsrd.com…                                           </div>', '<h1>Spellblight Jinx</h1>', '<p><b>School</b> <a href="https://www.d20pfsrd.com…d.com/classes/base-classes/witch">witch</a> 5</p>', '<p class="divider">CASTING</p>', '<p><b>Casting Time</b> 1 <a href="https://www.d20p…tandard action</a><br> <b>Components</b> V, S</p>', '<p class="divider">EFFECT</p>', '<p><b>Range</b> <a href="https://www.d20pfsrd.com/…Will</a> negates; <b>Spell Resistance</b> yes</p>', '<p class="divider">DESCRIPTION</p>', '<p>You inflict a curse similar to the spell burn s…ons#TOC-Staggered">staggered</a> for a round.</p>', '<p>Unlike with the spell burn spellblight, the bur…eal</a>, violet flame surrounding the caster.</p>', '<div class="section15">', '<div>Section 15: Copyright Notice</div>', '<div>', '<p><a href="https://www.amazon.com/gp/product/1601…n T. Helt, Thurston Hillman, and Ron Lundeen.</p>', '</div>', '</div>                                            …              \x3C!--div style="clear:both"></div-->', ''] */

pub fn get_links_from_html(html: &str) -> HashSet<String> {
    let document = Html::parse_fragment(html);
    let selector = Selector::parse("a").expect("Not Not Boi");
    let result = document
        .select(&selector)
        .filter_map(|el| el.value().attr("href"))
        .filter(allowed_extension)
        .filter_map(normalize_url)
        .collect::<HashSet<String>>();
    result
}

fn allowed_extension(url: &&str) -> bool {
    let ext = Path::new(url).extension();
    if let Some(s) = ext {
        match s.to_str() {
            Some("xml") => true,
            _ => false,
        }
    } else {
        true
    }
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
