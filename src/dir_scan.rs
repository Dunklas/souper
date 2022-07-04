use std::{fs, io, path};

const GLOBAL_EXCLUDE_DIRS: [&str; 3] = [
    "node_modules",
    "bin",
    "obj"
];

pub fn scan(dir: &path::PathBuf, exclude_dirs: &Vec<path::PathBuf>) -> Result<Vec<path::PathBuf>, io::Error> {
    let mut files: Vec<path::PathBuf> = Vec::new();
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
                    files.push(path);
                },
                Some(file_name_str) => {
                    if file_name_str.contains(".csproj") {
                        files.push(path);
                    }
                },
                _ => {}
            }
        }
    }
    Ok(files)
}
