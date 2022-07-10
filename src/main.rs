use clap::Parser;
use serde_json::{json, Map, Value};
use std::{env, path};

mod dir_scan;
mod parse;
mod soup;
mod utils;

use soup::model::SoupContexts;

/// Scans a given repository for software of unknown provenance (SOUP) and outputs them in a file.
#[derive(Parser)]
#[clap(version)]
struct Cli {
    /// Output file to print report in
    #[clap(short = 'o', long = "output-file", parse(from_os_str))]
    file: path::PathBuf,

    /// Directory to scan
    #[clap(short = 'd', long = "directory", parse(from_os_str))]
    target_dir: Option<path::PathBuf>,

    /// Directory to exclude
    #[clap(short = 'e', long = "exclude-directory", parse(from_os_str))]
    exclude_dirs: Vec<path::PathBuf>,

    // Key to add in meta property
    #[clap(short = 'm', long = "meta-key")]
    meta_keys: Vec<String>,
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
        },
    };
    let path = target_dir.as_path();
    if !path.exists() || !path.is_dir() {
        panic!("Invalid directory: {:?}", path);
    }
    let output_path = args.file.into_boxed_path();
    if output_path.is_dir() {
        panic!("Invalid output file: {:?}", output_path);
    }
    let exclude_dirs = args.exclude_dirs;

    let default_meta = args
        .meta_keys
        .into_iter()
        .map(|meta_key| (meta_key, json!("")))
        .collect::<Map<String, Value>>();

    let mut current_contexts = match output_path.is_file() {
        true => match SoupContexts::from_output_file(&output_path) {
            Ok(contexts) => contexts,
            Err(e) => {
                panic!("{}", e);
            }
        },
        false => SoupContexts::empty(),
    };
    let result = match dir_scan::scan(&target_dir, &exclude_dirs) {
        Ok(result) => result,
        Err(e) => {
            panic!("{}", e);
        }
    };
    let scanned_contexts = match SoupContexts::from_paths(result, path, default_meta) {
        Ok(contexts) => contexts,
        Err(e) => {
            panic!("{}", e);
        }
    };

    current_contexts.apply(scanned_contexts);
    if let Err(e) = current_contexts.write_to_file(&output_path) {
        panic!("{}", e);
    }
}
