use pathcrawler::{fetch_url, get_links_from_html};
use std::io::Error as IoErr;
use std::{collections::HashSet, time::Instant};

// fn write_file(path: &str, content: &str) {
//     fs::create_dir_all(format!("static{}", path)).unwrap();
//     fs::write(format!("static{}/index.html", path), content).unwrap();
// }
#[derive(Debug)]
enum Error {
    Write { _url: String, _e: IoErr },
    Fetch { _url: String, _e: reqwest::Error },
}

type Result<T> = std::result::Result<T, Error>;

impl<S: AsRef<str>> From<(S, IoErr)> for Error {
    fn from((url, e): (S, IoErr)) -> Self {
        Error::Write {
            _url: url.as_ref().to_string(),
            _e: e,
        }
    }
}

impl<S: AsRef<str>> From<(S, reqwest::Error)> for Error {
    fn from((url, e): (S, reqwest::Error)) -> Self {
        Error::Fetch {
            _url: url.as_ref().to_string(),
            _e: e,
        }
    }
}

fn main() -> Result<()> {
    let now = Instant::now();

    let client = reqwest::blocking::Client::new();
    let origin_url = "https://www.d20pfsrd.com/";

    let body = fetch_url(&client, origin_url);
    // write_file("", &body);

    let mut visited = HashSet::new();
    visited.insert(origin_url.to_string());
    let found_urls = get_links_from_html(&body);
    let mut new_urls = found_urls
        .difference(&visited)
        .map(|x| x.to_string())
        .collect::<HashSet<String>>();

    while !new_urls.is_empty() {
        let found_urls: HashSet<String> = new_urls
            .iter()
            .map(|url| {
                let body = fetch_url(&client, url);
                // write_file(&url[origin_url.len() - 1..], &body);
                let links = get_links_from_html(&body);
                println!("Visited: {} found {} links.", url, links.len());
                links
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
