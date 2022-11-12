#[derive(Debug)]
pub struct CrawlerConfig<'a> {
    pub allowed_paths: Vec<&'a str>,
    pub disallowed_paths: Vec<&'a str>,
    pub crawl_delay: u8,
    pub sitemap_url: Option<&'a str>,
}

impl<'a> CrawlerConfig<'a> {
    pub fn new<'b>(html: &'b str) -> CrawlerConfig<'b> {
        let mut lines = html.split("\r\n").filter(|x| {
            let s = *x;
            !s.eq("\n") && !s.eq("")
        });
        let remove_pattern = |pattern| {
            move |line: &'b str| {
                if !line.starts_with(pattern) {
                    return None;
                }
                Some(line.trim_start_matches(pattern))
            }
        };
        let allowed_paths: Vec<&str> = lines
            .clone()
            .filter_map(remove_pattern("Allow: "))
            .collect();
        let disallowed_paths: Vec<&str> = lines
            .clone()
            .filter_map(remove_pattern("Disallow: "))
            .collect();
        let crawl_delay = if let Some(line) = lines.find(|line| line.starts_with("crawl-delay: ")) {
            let n = line.trim_start_matches("crawl-delay: ").parse::<u8>();
            if let Ok(delay) = n {
                delay
            } else {
                1
            }
        } else {
            1
        };
        let sitemap_url = if let Some(line) = lines.find(|line| line.starts_with("Sitemap: ")) {
            Some(line.trim_start_matches("Sitemap: "))
        } else {
            None
        };
        let created = CrawlerConfig {
            allowed_paths,
            disallowed_paths,
            crawl_delay,
            sitemap_url,
        };
        println!("{:#?}", created);
        created
    }
}
