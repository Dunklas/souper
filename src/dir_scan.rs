use std::{fs, io, path};

const EXCLUDE_DIRS: [&'static str; 3] = [
    "node_modules",
    "bin",
    "obj"
];

pub fn scan(dir: &path::PathBuf) -> Result<Vec<path::PathBuf>, io::Error> {
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
            let mut content = scan(&path)?;
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
