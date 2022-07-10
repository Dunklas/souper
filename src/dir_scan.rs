use std::{
    fs,
    io::{
        Error
    },
    path::{
        PathBuf
    }
};
use crate::parse::{
    SoupSource,
    package_json::PackageJson,
    csproj::CsProj,
    docker_base::DockerBase
};

const GLOBAL_EXCLUDE_DIRS: [&str; 3] = [
    "node_modules",
    "bin",
    "obj"
];

pub fn scan2(dir: &PathBuf, exclude_dirs: &Vec<PathBuf>) -> Result<Vec<(PathBuf, Vec<Box<dyn SoupSource>>)>, Error>{
    let mut sources: Vec<(PathBuf, Vec<Box<dyn SoupSource>>)> = Vec::new();
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
            let mut content = scan2(&path, exclude_dirs)?;
            sources.append(&mut content);
            continue;
        }
        if file_type.is_file() {
            match file_name.to_str() {
                Some("package.json") => {
                    sources.push((path, vec![Box::new(PackageJson{})]));
                },
                Some(file_name_str) if file_name_str.contains(".csproj") => {
                    sources.push((path, vec![Box::new(CsProj{})]));
                },
                Some(file_name_str) if file_name_str.contains("Dockerfile") => {
                    sources.push((path, vec![Box::new(DockerBase{})]));
                },
                _ => {}
            }
        }
    }
    Ok(sources)
}

pub fn scan(dir: &PathBuf, exclude_dirs: &Vec<PathBuf>) -> Result<Vec<PathBuf>, Error> {
    let mut files: Vec<PathBuf> = Vec::new();
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
                Some(file_name_str) if file_name_str.contains(".csproj") => {
                    files.push(path);
                },
                Some(file_name_str) if file_name_str.contains("Dockerfile") => {
                    files.push(path);
                },
                _ => {}
            }
        }
    }
    Ok(files)
}
