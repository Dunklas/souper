use std::io::Write;
use std::{
    env,
    fs,
    io,
    path,
    result
};
use clap::Parser;

use crate::parse::package_json::PackageJson;
use crate::parse::SoupSource;

mod soup;
mod parse;

const EXCLUDE_DIRS: [&'static str; 3] = [
    "node_modules",
    "bin",
    "obj"
];

/// Scans a given repository for software of unknown provenance (SOUP) and outputs them in a file.
#[derive(Parser)]
struct Cli {
    /// Output file to print report in
    #[clap(short = 'o', long = "output-file", parse(from_os_str))]
    file: path::PathBuf,
    /// Directory to scan
    #[clap(short = 'd', long = "directory", parse(from_os_str))]
    target_dir: Option<path::PathBuf>
}

fn main() {
    let args = Cli::parse();
    let target_dir = match args.target_dir {
        Some(target_dir) => target_dir,
        None => match env::current_dir() {
            Ok(current_dir) => current_dir,
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    };
    let path = target_dir.as_path();
    if !path.exists() || !path.is_dir() {
        panic!("Invalid directory: {:?}", path);
    }
    let result = walk_dir(target_dir).unwrap();
    let mut soup_contexts: Vec<soup::SoupContext> = Vec::new();
    for r in result {
        let file = fs::File::open(&r).unwrap();
        let reader = io::BufReader::new(file);
        let soups = PackageJson::soups(reader);
        soup_contexts.push(soup::SoupContext{
            path: r,
            soups
        });
    }
    write_soups(soup_contexts, args.file).unwrap();
}

fn walk_dir(dir: path::PathBuf) -> result::Result<Vec<path::PathBuf>, io::Error> {
    let mut files: Vec<path::PathBuf> = Vec::new();
    'entries: for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let file_name = entry.file_name();
        if file_type.is_dir() {
            for ex in EXCLUDE_DIRS {
                if file_name.eq(ex) {
                    continue 'entries;
                }
            }
            let mut content = walk_dir(path)?;
            files.append(&mut content);
            continue;
        }
        if file_type.is_file() {
            if file_name.eq("package.json") {
                files.push(path);
            }
        }
    }
    return Ok(files);
}

fn write_soups(soup_contexts: Vec<soup::SoupContext>, path: path::PathBuf) -> Result<(), io::Error> {
    let mut output_file = fs::File::create(path)?;
    output_file.write_all(serde_json::to_string_pretty(&soup_contexts).unwrap().as_bytes())?;
    Ok(())
}
