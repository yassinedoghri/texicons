use regex::Regex;
use std::{fs};

#[cfg(windows)]
const LINE_ENDING : &'static str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING : &'static str = "\n";

fn main() {
    let re = Regex::new(r#"\.ri-([a-zA-Z0-9-]*):before[\S\s]\{[\S\s]content:[\S\s]"\\([a-z0-9]*)""#).unwrap();
    let style_contents = fs::read_to_string("./RemixIcon/fonts/remixicon.css")
        .expect("Should have been able to read the file");

    let mut results = vec![];
 
    for (_, [name, symbol]) in re.captures_iter(&style_contents).map(|c| c.extract()) {
        results.push((name, symbol.to_uppercase()));
    }

    println!("{:?}", results);

    let heading = "\\NeedsTeXFormat{LaTeX2e}
\\ProvidesPackage{remixicon}[2024/11/13 RemixIcon package]
\\usepackage{fontspec}

\\newfontfamily{\\riFont}{remixicon.ttf}

\\newcommand{\\ri}[1]{{
    \\csname ri@#1\\endcsname
}}

";

    let mut mappings:String = String::from("");
    for (name, symbol) in results {
        mappings = format!(r#"{}\expandafter\def\csname ri@{}\endcsname {{\riFont \symbol{{"{}}}}}{}"#, mappings.to_string(), name, symbol, LINE_ENDING);
    }

    fs::write("./remixicon.sty",  [heading.to_string(), mappings].join(LINE_ENDING)).expect("Unable to write file");
}
