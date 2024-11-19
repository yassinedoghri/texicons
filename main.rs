use clap::{Parser, Subcommand};
use convert_case::{Case, Casing};
use regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, fs, path};

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
    variants: Option<HashMap<String, Variant>>,
}

#[derive(Debug, Deserialize)]
struct Variant {
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

    println!("{:?}", icon_sets);

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

        if icon_set.variants.is_none() {
            continue;
        }

        for (_, icon_set_variant) in icon_set.variants.as_ref().unwrap().iter() {
            download_file(
                &icon_set_variant.font_url,
                format!(
                    "./icon-sets/{}/{}",
                    &icon_set_key, &icon_set_variant.font_name
                ),
            )
            .await;

            download_file(
                &icon_set_variant.codepoints_url,
                format!(
                    "./icon-sets/{}/{}",
                    &icon_set_key, &icon_set_variant.codepoints_name
                ),
            )
            .await;
        }
    }
}

fn generate_packages(icon_sets: &HashMap<String, IconSet>) {
    for (icon_set_key, icon_set) in icon_sets.into_iter() {
        let package_heading = format!(
            "\\NeedsTeXFormat{{LaTeX2e}}
\\ProvidesPackage{{{}}}[2024/11/13 TeXicons set for {}]
\\usepackage{{fonticon_set_variant_keyspec}}",
            ["texicons", &icon_set_key].join("-"),
            &icon_set.name
        );

        println!("{:?}{:?}", icon_set_key, icon_set.regex);
        let re = Regex::new(&icon_set.regex).unwrap();

        let codepoints_contents =
            fs::read_to_string(["./icon-sets", &icon_set_key, &icon_set.codepoints_name].join("/"))
                .expect("Should have been able to read the file");

        let mut results = vec![];

        for (_, [name, symbol]) in re.captures_iter(&codepoints_contents).map(|c| c.extract()) {
            results.push((name.to_case(Case::Kebab), symbol.to_uppercase()));
        }

        let mut font_families = vec![format!(
            "\\newfontfamily{{\\{}Font}}{{{}}}",
            &icon_set_key.to_case(Case::Camel),
            &icon_set.font_name
        )];

        let mut mappings = vec![];
        for (name, symbol) in results {
            mappings.push(format!(
                r#"\expandafter\def\csname icon@{}:{}\endcsname {{\{}Font \symbol{{"{}}}}}"#,
                &icon_set_key,
                name,
                &icon_set_key.to_case(Case::Camel),
                symbol,
            ));
        }

        println!(
            "{:?}",
            format!("./icon-sets/{}/{}", &icon_set_key, &icon_set.font_name),
        );

        let file_path = format!(
            "./packages/texicons-{}/{}.sty",
            &icon_set_key,
            ["texicons", &icon_set_key].join("-")
        );
        let path = path::Path::new(&file_path);

        fs::create_dir_all(path.parent().unwrap()).unwrap();

        // copy font to package folder
        fs::copy(
            format!("./icon-sets/{}/{}", &icon_set_key, &icon_set.font_name),
            format!(
                "./packages/texicons-{}/{}",
                &icon_set_key, &icon_set.font_name
            ),
        )
        .expect("Unable to copy font file");

        if !icon_set.variants.is_none() {
            for (icon_set_variant_key, icon_set_variant) in
                icon_set.variants.as_ref().unwrap().iter()
            {
                let re = Regex::new(&icon_set_variant.regex).unwrap();

                let codepoints_contents = fs::read_to_string(
                    [
                        "./icon-sets",
                        &icon_set_key,
                        &icon_set_variant.codepoints_name,
                    ]
                    .join("/"),
                )
                .expect("Should have been able to read the file");

                let mut results = vec![];

                for (_, [name, symbol]) in
                    re.captures_iter(&codepoints_contents).map(|c| c.extract())
                {
                    results.push((name.to_case(Case::Kebab), symbol.to_uppercase()));
                }

                let camel_cased_font_name =
                    format!("{}-{}", &icon_set_key, &icon_set_variant_key).to_case(Case::Camel);

                font_families.push(format!(
                    "\\newfontfamily{{\\{}Font}}{{{}}}",
                    &camel_cased_font_name, &icon_set_variant.font_name
                ));

                for (name, symbol) in results {
                    mappings.push(format!(
                        r#"\expandafter\def\csname icon@{}:{}\endcsname {{\{}Font \symbol{{"{}}}}}"#,
                        &icon_set_key,
                        [name, icon_set_variant_key.to_string()].join("-"),
                        &camel_cased_font_name,
                        symbol,
                    ));
                }

                // copy variant font to package folder
                fs::copy(
                    format!(
                        "./icon-sets/{}/{}",
                        &icon_set_key, &icon_set_variant.font_name
                    ),
                    format!(
                        "./packages/texicons-{}/{}",
                        &icon_set_key, &icon_set_variant.font_name
                    ),
                )
                .expect("Unable to copy font file");
            }
        }

        fs::write(
            path,
            [
                package_heading.to_string(),
                font_families.join("\n"),
                mappings.join("\n"),
            ]
            .join("\n\n"),
        )
        .expect("Unable to write file");
    }
}

async fn download_file(url: &String, destination: String) {
    let body = reqwest::get(url).await.unwrap().text().await.unwrap();

    let path = path::Path::new(&destination);

    fs::create_dir_all(path.parent().unwrap()).unwrap();

    fs::write(path, body).expect("Unable to write file");
}
