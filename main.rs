use clap::{Parser, Subcommand};
use convert_case::{Case, Casing};
use regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, fs, path};

#[cfg(windows)]
const LINE_ENDING: &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &'static str = "\n";

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    DownloadIconSets,
    GeneratePackages,
}

#[derive(Debug, Deserialize)]
struct IconSet {
    name: String,
    version: String,
    font_url: String,
    font_name: String,
    codepoints_url: String,
    codepoints_name: String,
    regex: String,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let icon_set_index = fs::read_to_string("./icon-sets/index.json")
        .expect("Should have been able to read the file");

    let icon_sets: HashMap<String, IconSet> =
        serde_json::from_str(icon_set_index.as_str()).expect("JSON was not well-formatted");

    match args.command {
        Commands::DownloadIconSets => download_icon_sets(&icon_sets).await,
        Commands::GeneratePackages => generate_packages(&icon_sets),
    }
}

async fn download_icon_sets(icon_sets: &HashMap<String, IconSet>) {
    for (icon_set_key, icon_set) in icon_sets.into_iter() {
        // download font file
        download_file(
            &icon_set.font_url,
            format!("./icon-sets/{}/{}", &icon_set_key, &icon_set.font_name),
        )
        .await;

        // download codepoints
        download_file(
            &icon_set.codepoints_url,
            format!(
                "./icon-sets/{}/{}",
                &icon_set_key, &icon_set.codepoints_name
            ),
        )
        .await;
    }
}

fn generate_packages(icon_sets: &HashMap<String, IconSet>) {
    for (icon_set_key, icon_set) in icon_sets.into_iter() {
        println!("{:?}{:?}", icon_set_key, icon_set.regex);
        let re = Regex::new(&icon_set.regex).unwrap();

        let style_contents =
            fs::read_to_string(["./icon-sets", &icon_set_key, &icon_set.codepoints_name].join("/"))
                .expect("Should have been able to read the file");

        let mut results = vec![];

        for (_, [name, symbol]) in re.captures_iter(&style_contents).map(|c| c.extract()) {
            results.push((name.to_case(Case::Kebab), symbol.to_uppercase()));
        }

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

        let file_path = format!(
            "./packages/{}/{}.sty",
            &icon_set_key,
            ["texicons", &icon_set_key].join("-")
        );
        let path = path::Path::new(&file_path);

        fs::create_dir_all(path.parent().unwrap()).unwrap();

        fs::write(path, [heading.to_string(), mappings].join(LINE_ENDING))
            .expect("Unable to write file");

        // copy font to package folder
        fs::copy(
            format!("./icon-sets/{}/{}", &icon_set_key, &icon_set.font_name),
            format!("./packages/{}/{}", &icon_set_key, &icon_set.font_name),
        )
        .expect("Unable to copy font file");
    }
}

async fn download_file(url: &String, destination: String) {
    let body = reqwest::get(url).await.unwrap().text().await.unwrap();

    let path = path::Path::new(&destination);

    fs::create_dir_all(path.parent().unwrap()).unwrap();

    fs::write(path, body).expect("Unable to write file");
}
