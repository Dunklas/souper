use std::io::Write;
use std::{
    env,
    fs,
    io,
    path
};
use clap::Parser;

mod soup;
mod parse;
mod dir_scan;
mod utils;

use soup::model::SoupContexts;

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
                panic!("baj {:?}", e);
            }
        }
    };
    let path = target_dir.as_path();
    if !path.exists() || !path.is_dir() {
        panic!("Invalid directory: {:?}", path);
    }
    let output_path = args.file.into_boxed_path();
    if output_path.is_dir() {
        panic!("Invalid output file: {:?}", output_path);
    }
    let current_contexts = match output_path.is_file() {
        true => match SoupContexts::from_output_file(&output_path) {
            Ok(contexts) => contexts,
            Err(e) => {
                panic!("{}", e);
            }
        }
        false => SoupContexts::empty()
    };
    let result = match dir_scan::scan(&target_dir) {
        Ok(result) => result,
        Err(e) => {
            panic!("{}", e);
        }
    };
    let scanned_contexts = match SoupContexts::from_paths(result, path) {
        Ok(contexts) => contexts,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let combined_contexts = SoupContexts::combine(current_contexts, scanned_contexts);
    match write_soups(combined_contexts, &output_path) {
        Err(e) => {
            panic!("{}", e);
        },
        _ => {}
    }
}

fn write_soups<P: AsRef<path::Path>>(soup_contexts: SoupContexts, path: P) -> Result<(), io::Error> {
    let mut output_file = fs::File::create(path)?;
    output_file.write_all(serde_json::to_string_pretty(&soup_contexts.contexts).unwrap().as_bytes())?;
    Ok(())
}
