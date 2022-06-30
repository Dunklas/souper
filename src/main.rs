use std::{
    collections,
    env,
    fs,
    io,
    path,
    result
};
use clap::Parser;
use serde::{
    Deserialize,
    Serialize
};
use serde_json::Result;

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
    let mut soup_contexts: Vec<SoupContext> = Vec::new();
    for r in result {
        let soup_context = find_soups(r);
        soup_contexts.push(soup_context);
    }
    let json = serde_json::to_string(&soup_contexts).unwrap();
    println!("{}", json);
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

#[derive(Serialize, Deserialize, Debug)]
struct PackageJson {
    dependencies: collections::HashMap<String, String>
}

fn find_soups(path: path::PathBuf) -> SoupContext {
    let file = fs::File::open(&path).unwrap();
    let reader = io::BufReader::new(file);
    let p: PackageJson = serde_json::from_reader(reader).unwrap();
    let soups = p.dependencies.into_iter()
            .map(|(key, value)| Soup {
                name: key,
                version: value,
                meta: collections::HashMap::new()
            })
            .collect::<Vec<Soup>>();
    return SoupContext {
        path,
        soups
    }
}

#[derive(Serialize, Deserialize)]
struct Soup {
    name: String,
    version: String,
    meta: collections::HashMap<String, serde_json::Value>
}

#[derive(Serialize, Deserialize)]
struct SoupContext {
    path: path::PathBuf,
    soups: Vec<Soup>
}
