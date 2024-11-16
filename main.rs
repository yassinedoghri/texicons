use convert_case::{Case, Casing};
use regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, fs};

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

#[derive(Debug, Deserialize)]
struct IconSet {
    name: String,
    font_folder: String,
    font_name: String,
    reference_path: String,
    regex: String,
}

fn main() {
    let icon_set_index = fs::read_to_string("./icon-sets/index.json")
        .expect("Should have been able to read the file");

    let icon_sets: HashMap<String, IconSet> =
        serde_json::from_str(icon_set_index.as_str()).expect("JSON was not well-formatted");

    for (icon_set_key, icon_set) in icon_sets.into_iter() {
        println!("{:?}{:?}", icon_set_key, icon_set.regex);
        let re = Regex::new(&icon_set.regex).unwrap();

        let style_contents =
            fs::read_to_string(["./icon-sets", &icon_set.name, &icon_set.reference_path].join("/"))
                .expect("Should have been able to read the file");

        let mut results = vec![];

        for (_, [name, symbol]) in re.captures_iter(&style_contents).map(|c| c.extract()) {
            results.push((name.to_case(Case::Kebab), symbol.to_uppercase()));
        }

        println!("{:?}", results);

        let heading = format!(
            "\\NeedsTeXFormat{{LaTeX2e}}
\\ProvidesPackage{{{}}}[2024/11/13 TeXicons set for {}]
\\usepackage{{fontspec}}

\\newfontfamily{{\\{}Font}}{{{}}}

",
            ["texicons", &icon_set_key].join("-"),
            &icon_set.name,
            &icon_set_key.to_case(Case::Camel),
            &icon_set.font_name
        );

        let mut mappings: String = String::from("");
        for (name, symbol) in results {
            mappings = format!(
                r#"{}\expandafter\def\csname icon@{}:{}\endcsname {{\{}Font \symbol{{"{}}}}}{}"#,
                mappings.to_string(),
                &icon_set_key,
                name,
                &icon_set_key.to_case(Case::Camel),
                symbol,
                LINE_ENDING
            );
        }

        fs::write(
            format!("./packages/{}.sty", ["texicons", &icon_set_key].join("-")),
            [heading.to_string(), mappings].join(LINE_ENDING),
        )
        .expect("Unable to write file");
    }
}
