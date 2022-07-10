use crate::parse::{csproj::CsProj, docker_base::DockerBase, package_json::PackageJson, cargo::Cargo, SoupParse};
use std::{fs, io::Error, path::PathBuf};

const GLOBAL_EXCLUDE_DIRS: [&str; 3] = ["node_modules", "bin", "obj"];

pub type SoupParsers = Vec<Box<dyn SoupParse>>;

pub fn scan(
    dir: &PathBuf,
    exclude_dirs: &Vec<PathBuf>,
) -> Result<Vec<(PathBuf, SoupParsers)>, Error> {
    let mut sources: Vec<(PathBuf, Vec<Box<dyn SoupParse>>)> = Vec::new();
    'entries: for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let file_name = entry.file_name();
        if file_type.is_dir() {
            for ex in GLOBAL_EXCLUDE_DIRS {
                if file_name == ex {
                    continue 'entries;
                }
            }
            for ex in exclude_dirs {
                if path == *ex {
                    continue 'entries;
                }
            }
            let mut content = scan(&path, exclude_dirs)?;
            sources.append(&mut content);
            continue;
        }
        if file_type.is_file() {
            match file_name.to_str() {
                Some("package.json") => {
                    sources.push((path, vec![Box::new(PackageJson {})]));
                }
                Some("Cargo.toml") => {
                    sources.push((path, vec![Box::new(Cargo {})]));
                }
                Some(file_name_str) if file_name_str.contains(".csproj") => {
                    sources.push((path, vec![Box::new(CsProj {})]));
                }
                Some(file_name_str) if file_name_str.contains("Dockerfile") => {
                    sources.push((path, vec![Box::new(DockerBase {})]));
                }
                _ => {}
            }
        }
    }
    Ok(sources)
}
