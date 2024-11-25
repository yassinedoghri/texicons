use chrono::Utc;
use clap::{Parser, Subcommand};
use convert_case::{Case, Casing};
use log::warn;
use ordermap::OrderMap;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::BufWriter,
    path,
};
use usvg;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    CleanIconSets,
    GeneratePackages,
}

#[derive(Debug, Deserialize)]
struct IconSet {
    prefix: String,
    info: IconSetInfo,
    icons: OrderMap<String, Icon>,
    height: Option<f32>,
    width: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct IconSetInfo {
    name: String,
    version: Option<String>,
    height: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct Icon {
    body: String,
    height: Option<f32>,
    width: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TexIconSet {
    prefix: String,
    name: String,
    font_id: String,
    version: Option<String>,
    icons: OrderMap<String, TexIcon>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TexIcon {
    codepoint: String,
    svg: String,
}

macro_rules! skip_fail {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                warn!("An error: {}; skipped.", e);
                continue;
            }
        }
    };
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let allow_list: Vec<String> = fs::read_to_string(".allow")
        .expect("Failed to read input")
        .split("\n")
        .filter(|&line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();

    let disallow_list: Vec<String> = fs::read_to_string(".disallow")
        .expect("Failed to read input")
        .split("\n")
        .filter(|&line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();

    match args.command {
        Commands::CleanIconSets => clean_icon_sets(&allow_list, &disallow_list),
        Commands::GeneratePackages => generate_packages(&allow_list, &disallow_list),
    }
}

fn clean_icon_sets(allow_list: &Vec<String>, disallow_list: &Vec<String>) {
    println!("{:?}", allow_list);

    let icon_set_paths = fs::read_dir("./icon-sets/json").unwrap();

    for icon_set_path in icon_set_paths {
        let path = icon_set_path.unwrap().path();
        let icon_set_json =
            fs::read_to_string(&path).expect("Should have been able to read the file");

        let icon_set: IconSet =
            serde_json::from_str(&icon_set_json.as_str()).expect("JSON was not well-formatted");

        if !allow_list.is_empty() && !allow_list.contains(&icon_set.prefix) {
            continue;
        }

        if !disallow_list.is_empty() && disallow_list.contains(&icon_set.prefix) {
            continue;
        }

        println!("{:?}", &path);

        let mut tex_icon_set: TexIconSet = TexIconSet {
            prefix: icon_set.prefix.clone(),
            name: icon_set.info.name,
            font_id: replace_numbers_to_letters(&icon_set.prefix).to_case(Case::Camel),
            version: icon_set.info.version,
            icons: OrderMap::new(),
        };

        let opt = usvg::Options::default();
        let write_opt = usvg::WriteOptions::default();
        let mut codepoint: u32 = 0xE000;
        for (name, icon) in icon_set.icons {
            println!("{:?}:{:?}", &icon_set.prefix, name);
            let width = icon
                .width
                .or(icon_set.width)
                .or(icon_set.info.height)
                .or(Some(24.0))
                .unwrap();
            let height = icon
                .height
                .or(icon_set.height)
                .or(icon_set.info.height)
                .or(Some(24.0))
                .unwrap();
            let svg = format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\">{}</svg>",
                width, height, icon.body
            );

            // TODO: flag if error with empty svg (or original malformed svg?)
            let tree = skip_fail!(usvg::Tree::from_str(&svg, &opt));

            tex_icon_set.icons.insert(
                name,
                TexIcon {
                    svg: tree.to_string(&write_opt),
                    codepoint: format!("{:X}", codepoint),
                },
            );
            codepoint = codepoint + 1;
        }

        let file = File::create(format!("./temp/icon-sets/{}.json", &icon_set.prefix)).unwrap();
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &tex_icon_set).unwrap();
    }
}

fn generate_packages(allow_list: &Vec<String>, disallow_list: &Vec<String>) {
    let icon_set_paths = fs::read_dir("./temp/icon-sets").unwrap();

    for icon_set_path in icon_set_paths {
        let path = icon_set_path.unwrap().path();

        if path.ends_with(".gitkeep") {
            continue;
        }

        let icon_set_json =
            fs::read_to_string(&path).expect("Should have been able to read the file");

        let icon_set: TexIconSet =
            serde_json::from_str(&icon_set_json).expect("JSON was not well-formatted");

        if !allow_list.is_empty() && !allow_list.contains(&icon_set.prefix) {
            continue;
        }

        if !disallow_list.is_empty() && disallow_list.contains(&icon_set.prefix) {
            continue;
        }

        println!("{:?}", icon_set.prefix);

        let mut mappings = vec![];
        let mut docs_table_rows = vec![];
        for (name, tex_icon) in &icon_set.icons {
            mappings.push(format!(
                include_str!("./templates/mapping.tmpl"),
                prefix = &icon_set.prefix,
                name = name,
                font_id = &icon_set.font_id,
                codepoint = &tex_icon.codepoint,
            ));
            docs_table_rows.push(format!(
                include_str!("./templates/docs-table_row.tmpl"),
                prefix = &icon_set.prefix,
                name = name,
            ));
        }

        let docs = format!(
            include_str!("./templates/docs.tex.tmpl"),
            prefix = &icon_set.prefix,
            rows = docs_table_rows.join("\n")
        );

        let file_path = format!(
            "./packages/texicons-{}/{}.sty",
            &icon_set.prefix,
            ["texicons", &icon_set.prefix].join("-")
        );
        let path = path::Path::new(&file_path);

        fs::create_dir_all(path.parent().unwrap()).unwrap();

        // copy font to package folder
        fs::copy(
            format!("./temp/fonts/{}.ttf", &icon_set.prefix),
            format!(
                "./packages/texicons-{}/{}.ttf",
                &icon_set.prefix, &icon_set.prefix
            ),
        )
        .expect("Unable to copy font file");

        fs::write(
            format!(
                "./packages/texicons-{}/{}.tex",
                &icon_set.prefix,
                ["texicons", &icon_set.prefix].join("-")
            ),
            docs,
        )
        .expect("Unable to write file");

        fs::write(
            path,
            format!(
                include_str!("./templates/package.sty.tmpl"),
                prefix = &icon_set.prefix,
                date = Utc::now().format("%Y-%m-%d"),
                info = match &icon_set.version {
                    None => icon_set.name,
                    Some(version) => format!("{} v{}", &icon_set.name, version),
                },
                font_id = &icon_set.font_id,
                mappings = mappings.join("\n"),
            ),
        )
        .expect("Unable to write file");
    }
}

fn replace_numbers_to_letters(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        let next_char;
        if c.is_numeric() {
            next_char = match c {
                '1' => 'l',
                '2' => 'a',
                '3' => 't',
                '4' => 'e',
                '5' => 'x',
                '6' => 'i',
                '7' => 'c',
                '8' => 'o',
                '9' => 'n',
                _ => 's',
            };
        } else {
            next_char = c;
        }
        result.push(next_char);
    }

    result
}
