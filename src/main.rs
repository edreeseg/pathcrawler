use pathcrawler::{fetch_url, get_links_from_html, parse_spell, CrawlerResult};
use regex::Regex;
use std::{collections::HashSet, time::Instant};
use std::{thread, time};

#[derive(Debug)]
enum PageType {
    Race,
    Class,
    ClassSpell,
    Spell,
    Feat,
    Skill,
    Default,
}

// fn write_file(path: &str, content: &str) {
//     fs::create_dir_all(format!("static{}", path)).unwrap();
//     fs::write(format!("static{}/index.html", path), content).unwrap();
// }

fn main() -> CrawlerResult<()> {
    let now = Instant::now();

    let client = reqwest::blocking::Client::new();
    let origin_url = "https://www.d20pfsrd.com/";

    let body = retry_fetch(&client, origin_url).unwrap();
    // write_file("", &body);

    let mut visited = HashSet::new();
    visited.insert(origin_url.to_string());
    let found_urls = get_links_from_html(&body);
    let mut new_urls = found_urls
        .difference(&visited)
        .map(|x| x.to_string())
        .collect::<HashSet<String>>();

    let spell_regex = Regex::new(&format!(
        r"{}magic/((all-spells/[a-z]/\w+)|(3rd-party-spells/\w+/\w+))",
        origin_url
    ))
    .unwrap();

    while !new_urls.is_empty() {
        let found_urls: HashSet<String> = new_urls
            .iter()
            .filter_map(|url| {
                if !url.starts_with(&format!("{}magic", origin_url)) {
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
                let body = retry_fetch(&client, url).unwrap();
                // write_file(&url[origin_url.len() - 1..], &body);
                if let PageType::Spell = page_type {
                    let _spell = parse_spell(&body);
                }
                let links = get_links_from_html(&body);
                println!(
                    "Visited: {} - {:?} page. Found {} links.",
                    url,
                    page_type,
                    links.len()
                );
                // If fetch was faster than one second, sleep for the remaining time to avoid DoS
                let fetch_duration = fetch_start.elapsed().as_millis();
                if fetch_start.elapsed().as_millis() < 1000 {
                    let sleep_time: u64 = (1000 - fetch_duration).try_into().unwrap();
                    println!("Sleeping for {}ms", sleep_time);
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
        println!("New urls: {}", new_urls.len());
    }

    println!("URLs: {:#?}", found_urls);
    println!("{}", now.elapsed().as_secs());

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
