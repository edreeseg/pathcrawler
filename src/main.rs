use crate::robots::CrawlerConfig;
use crate::spells::parse_spell;
use pathcrawler::{
    fetch_url, get_links_from_html, get_links_from_xml, replace_file, write_file, CrawlerResult,
};
use regex::Regex;
use std::{collections::HashSet, time::Instant};
use std::{thread, time};

mod robots;
mod spells;

#[derive(Debug)]
enum PageType {
    Race,
    Class,
    _ClassSpell,
    Spell,
    Feat,
    Skill,
    Default,
}

fn main() -> CrawlerResult<()> {
    let now = Instant::now();

    let client = reqwest::blocking::Client::new();
    let robots_url = "https://www.d20pfsrd.com/robots.txt";

    let robots_body = retry_fetch(&client, robots_url).unwrap();
    // write_file("", &body);

    let mut visited = HashSet::new();
    visited.insert(robots_url.to_string());

    let config = CrawlerConfig::new(&robots_body);

    let origin_url = config.sitemap_url.unwrap_or("https://www.d20pfsrd.com");
    let origin_is_xml = is_xml(origin_url);

    let origin_body = retry_fetch(&client, origin_url).unwrap();

    let found_urls = if origin_is_xml {
        get_links_from_xml(&origin_body)
    } else {
        get_links_from_html(&origin_body)
    };
    let mut new_urls = found_urls
        .difference(&visited)
        .map(|x| x.to_string())
        .collect::<HashSet<String>>();

    println!("{:#?}", new_urls);

    let spell_regex = Regex::new(&format!(
        r"^https://www.d20pfsrd.com/magic/((all-spells/[^/]+/[^/]+)|(3rd-party-spells/[^/]+/[^/]+))/?$"
    ))
    .unwrap();

    while !new_urls.is_empty() {
        let found_urls: HashSet<String> = new_urls
            .iter()
            .filter_map(|url| {
                for path in &config.disallowed_paths {
                    if url.contains(path) {
                        return None;
                    }
                }
                let xml_link = is_xml(url);
                if !xml_link && !url.starts_with(&format!("https://www.d20pfsrd.com/magic")) {
                    // Focus only on the magic pages
                    return None;
                }
                let page_type = if url.starts_with(&format!("{}classes/", origin_url)) {
                    PageType::Class
                } else if url.starts_with(&format!("{}races/", origin_url)) {
                    PageType::Race
                } else if url.starts_with(&format!("{}feats/", origin_url)) {
                    PageType::Feat
                } else if spell_regex.is_match(url) {
                    PageType::Spell
                } else if url.starts_with(&format!("{}skill/", origin_url)) {
                    PageType::Skill
                } else {
                    PageType::Default
                };
                let fetch_start = Instant::now();
                println!("Navigating to {}", url);
                let body = retry_fetch(&client, url).unwrap();
                // write_file(&url[origin_url.len() - 1..], &body);
                if let PageType::Spell = page_type {
                    let spell = parse_spell(&body);
                    if let None = spell {
                        match write_file("log/broken_links.txt", url) {
                            Ok(_) => (),
                            Err(_) => println!("Failed to write broken link {}", url),
                        }
                    }
                }
                let links = if xml_link {
                    get_links_from_xml(&body)
                } else {
                    get_links_from_html(&body)
                };
                // If fetch was faster than one second, sleep for the remaining time to avoid DoS
                let fetch_duration = fetch_start.elapsed().as_millis();
                let delay_as_millis: u128 = config.crawl_delay.into();
                let delay_as_millis = delay_as_millis * 1000;
                if fetch_start.elapsed().as_millis() < delay_as_millis {
                    let sleep_time: u64 = (delay_as_millis - fetch_duration).try_into().unwrap();
                    thread::sleep(time::Duration::from_millis(sleep_time));
                }
                Some(links)
            })
            .fold(HashSet::new(), |mut acc, x| {
                acc.extend(x);
                acc
            });
        visited.extend(new_urls);

        new_urls = found_urls
            .difference(&visited)
            .map(|x| x.to_string())
            .collect::<HashSet<String>>();
        let mut sorted_urls: Vec<&String> = new_urls.iter().collect();
        sorted_urls.sort_by_key(|name| name.to_lowercase());
        match replace_file(
            "log/current_urls_object.txt",
            &format!("{:#?}", sorted_urls),
        ) {
            Ok(_) => (),
            Err(e) => println!("Failed to write URLs object {}", e),
        };
        println!("New urls: {}", new_urls.len());
    }

    // let path = "spells.txt";
    // let mut output = File::create(path)?;

    // .for_each(|x| {
    //     println!("{}", x);
    //     write!(output, "{}\n", x).unwrap();
    // });
    Ok(())
}

fn retry_fetch(client: &reqwest::blocking::Client, url: &str) -> CrawlerResult<String> {
    // Retry three times on failure
    for n in 1..=3 {
        let response = fetch_url(client, url);
        if response.is_ok() || n == 3 {
            return response;
        }
        thread::sleep(time::Duration::from_secs(n));
    }
    panic!();
}

fn is_xml(url: &str) -> bool {
    url.ends_with("xml")
}
