use std::{
    env,
    io,
    path,
    fs
};
use clap::Parser;

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
    for r in result {
        println!("{:?}", r);
    }
}

fn walk_dir(dir: path::PathBuf) -> Result<Vec<path::PathBuf>, io::Error> {
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
