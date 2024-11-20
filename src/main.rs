use clap::{Parser, Subcommand};
use convert_case::{Case, Casing};
use regex::Regex;
use serde::Deserialize;
use std::{fs, path};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    GeneratePackages,
}

#[derive(Debug, Deserialize)]
struct IconSet {
    prefix: String,
    info: IconSetInfo,
}

#[derive(Debug, Deserialize)]
struct IconSetInfo {
    name: String,
    version: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::GeneratePackages => generate_packages(),
    }
}

fn generate_packages() {
    let icon_set_paths = fs::read_dir("./icon-sets/json").unwrap();

    for icon_set_path in icon_set_paths {
        let icon_set_json = fs::read_to_string(icon_set_path.unwrap().path())
            .expect("Should have been able to read the file");

        let icon_set: IconSet =
            serde_json::from_str(&icon_set_json.as_str()).expect("JSON was not well-formatted");

        if ["devicon-plain", "emblemicons"].contains(&icon_set.prefix.as_str()) {
            continue;
        }

        let package_heading = format!(
            "\\NeedsTeXFormat{{LaTeX2e}}
\\ProvidesPackage{{{}}}[2024/11/13 TeXicons set for {}]
\\usepackage{{fontspec}}

\\newfontfamily{{\\{}Font}}{{{}.ttf}}",
            ["texicons", &icon_set.prefix].join("-"),
            match &icon_set.info.version {
                None => icon_set.info.name,
                Some(version) => format!("{} v{}", &icon_set.info.name, version),
            },
            &icon_set.prefix.to_case(Case::Camel),
            &icon_set.prefix
        );

        println!("{:?}", icon_set.prefix);

        let re = Regex::new("([a-z0-9-]+) ([a-z0-9]+)").unwrap();

        let codepoints_contents =
            fs::read_to_string(format!("./fonts/{}.codepoints", &icon_set.prefix))
                .expect("Should have been able to read the file");

        let mut results = vec![];

        for (_, [name, symbol]) in re.captures_iter(&codepoints_contents).map(|c| c.extract()) {
            results.push((name, symbol.to_uppercase()));
        }

        let mut mappings = vec![];
        for (name, symbol) in results {
            mappings.push(format!(
                r#"\expandafter\def\csname icon@{}:{}\endcsname {{\{}Font \symbol{{"{}}}}}"#,
                &icon_set.prefix,
                name,
                &icon_set.prefix.to_case(Case::Camel),
                symbol,
            ));
        }

        let file_path = format!(
            "./packages/texicons-{}/{}.sty",
            &icon_set.prefix,
            ["texicons", &icon_set.prefix].join("-")
        );
        let path = path::Path::new(&file_path);

        fs::create_dir_all(path.parent().unwrap()).unwrap();

        // copy font to package folder
        fs::copy(
            format!("./fonts/{}.ttf", &icon_set.prefix),
            format!(
                "./packages/texicons-{}/{}.ttf",
                &icon_set.prefix, &icon_set.prefix
            ),
        )
        .expect("Unable to copy font file");

        fs::write(
            path,
            [package_heading.to_string(), mappings.join("\n")].join("\n\n"),
        )
        .expect("Unable to write file");
    }
}
