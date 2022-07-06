use std::{fs, io, path};
use crate::parse::{
    SoupSource,
    package_json::PackageJson
};

const GLOBAL_EXCLUDE_DIRS: [&str; 3] = [
    "node_modules",
    "bin",
    "obj"
];

pub fn scan(dir: &path::PathBuf, exclude_dirs: &Vec<path::PathBuf>) -> Result<Vec<SoupSource<io::BufReader<fs::File>>>, io::Error> {
    let mut files: Vec<SoupSource<io::BufReader<fs::File>>> = Vec::new();
    'entries: for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let file_name = entry.file_name();
        if file_type.is_dir() {
            for ex in GLOBAL_EXCLUDE_DIRS {
                if file_name.eq(ex) {
                    continue 'entries;
                }
            }
            for ex in exclude_dirs {
                if path.eq(ex) {
                    continue 'entries;
                }
            }
            let mut content = scan(&path, exclude_dirs)?;
            files.append(&mut content);
            continue;
        }
        if file_type.is_file() {
            match file_name.to_str() {
                Some("package.json") => {
                    let f = fs::File::open(&path).unwrap();
                    let r = io::BufReader::new(f);
                    let source = SoupSource::new(r);
                    source.append_parsers(vec![Box::new(PackageJson{})]);
                    files.push(source);
                },
                Some(file_name_str) if file_name_str.contains(".csproj") => {
                    let f = fs::File::open(&path).unwrap();
                    let r = io::BufReader::new(f);
                    let source = SoupSource::new(r);
                    source.append_parsers(vec![Box::new(PackageJson{})]);
                    files.push(source);
                },
                Some(file_name_str) if file_name_str.contains("Dockerfile") => {
                    let f = fs::File::open(&path).unwrap();
                    let r = io::BufReader::new(f);
                    let source = SoupSource::new(r);
                    source.append_parsers(vec![Box::new(PackageJson{})]);
                    files.push(source);
                },
                _ => {}
            }
        }
    }
    Ok(files)
}
