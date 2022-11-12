use std::collections::HashMap;

use pathcrawler::write_file;
use regex::Regex;
use scraper::element_ref::Text;
use scraper::{Html, Selector};

// struct Spell {
//     name: &str,
//     description: &str,
//     school: &str,
//     subschool: &str,
//     descriptors: Vec<&str>,
// }

// pub fn parse_class_spells() -> Vec<Spell> {

// }

pub fn parse_spell(html: &str) -> Option<()> {
    let document = Html::parse_fragment(html);
    let article_content = Selector::parse("#article-content");
    let article = document
        .select(&article_content.unwrap())
        .next()
        .unwrap()
        .inner_html();
    let article: Vec<&str> = article.split("\n").map(|str| str.trim()).collect();
    let mut spell_name = None;
    let mut school = None;
    let mut subschool = None;
    let mut descriptors = vec![];
    let mut levels_hash: HashMap<String, i8> = HashMap::new();
    for line in article {
        if line.starts_with("<h1>") {
            spell_name = get_spell_name(line);
        }
        if line.contains("<b>School</b>") {
            school = get_spell_school(line);
            let subschool_regex = Regex::new(r#"\([\w>< ="'/]+\)"#);
            let descriptors_regex = Regex::new(r#"\[[\w>< ="'/,]+\]"#);
            if let Some(result) = subschool_regex.unwrap().find(line) {
                let parsed_subschool = result.as_str().trim_matches(|c| c == '(' || c == ')');
                subschool = Some(remove_wrapping_anchor(parsed_subschool));
            }
            if let Some(result) = descriptors_regex.unwrap().find(line) {
                descriptors = result
                    .as_str()
                    .trim_matches(|c| c == '[' || c == ']')
                    .split(",")
                    .map(|x| remove_wrapping_anchor(x.trim()))
                    .collect();
            }
        }
        if line.contains("<b>Level</b>") {
            let levels = line.split("<b>Level</b>").collect::<Vec<&str>>()[1];
            let levels = levels.split(";").next().unwrap();
            let levels = levels.trim();
            let levels_regexp =
                Regex::new(r#"((<a[ :\w"/.=-]+>[\w/ ]+</a+>)|([\w/ ]+)) (\d{1,2})(, )?"#).unwrap();

            // Need to account for multiple spells per page (e.g. Inflict Pain and Inflict Pain, Mass)
            // Added spaces to the regexp to try to catch random cases like "hedge witch"
            println!("Capturing: {}", levels);
            let levels = levels_regexp.captures_iter(levels);
            for item in levels {
                let (class, level) = (remove_wrapping_anchor(&item[1]), &item[4]);
                let level = level.parse();
                let level = if let Ok(n) = level { n } else { -1 };
                levels_hash.insert(class, level);
            }
        }
    }
    let spell_name = spell_name?;
    let school = school?;
    let subschool = subschool.unwrap_or(String::from("N/A"));

    match write_file(
        "log/spells.txt",
        &format!(
            "{}, {}, {}, {:?}, {:?}",
            spell_name, school, subschool, descriptors, levels_hash
        ),
    ) {
        Ok(_) => (),
        Err(_) => println!("Failed to write spell {}", spell_name),
    };
    Some(())
}

fn _parse_spell_metadata(mut iter: Text) -> () {
    while let Some(text_value) = iter.next() {
        println!("Metadata: {}", text_value);
    }
    ()
}

fn get_spell_name(line: &str) -> Option<String> {
    let spell_name = Html::parse_fragment(line)
        .select(&Selector::parse("h1").unwrap())
        .next()?
        .text()
        .next()?
        .to_string();
    Some(spell_name)
}

fn get_spell_school(line: &str) -> Option<String> {
    // println!("This is the line: {}", school_line);
    let fragment = Html::parse_fragment(line);
    let schools_of_magic = vec![
        "Abjuration",
        "Conjuration",
        "Divination",
        "Enchantment",
        "Evocation",
        "Illusion",
        "Necromancy",
        "Transmutation",
    ];
    // Account for cases where schools are linked conventionally
    let selector = schools_of_magic
        .iter()
        .map(|s| format!("a[href*=\"#TOC-{}\"]", s))
        .collect::<Vec<String>>()
        .join(", ");
    let parsed_selector = &Selector::parse(&selector).unwrap();
    let mut selection = fragment.select(parsed_selector);
    let school = if let Some(node) = selection.next() {
        node.text().next().unwrap().to_string()
    } else if line.to_lowercase().contains("universal") {
        // The universal string is never wrapped in an anchor
        String::from("universal")
    } else {
        return None;
    };
    Some(school.to_string())
}

fn remove_wrapping_anchor(slice: &str) -> String {
    let potential_anchor = Html::parse_fragment(&slice);
    let anchor_selector = Selector::parse("a").unwrap();
    let potential_anchor = potential_anchor.select(&anchor_selector).next();
    if let Some(node) = potential_anchor {
        String::from(node.text().next().unwrap_or(slice))
    } else {
        String::from(slice)
    }
}
