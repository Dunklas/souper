use crate::{
    parse::{
        apt::Apt, cargo::Cargo, csproj::CsProj, docker_base::DockerBase, package_json::PackageJson,
        SoupParse,
    },
    soup::model::{Soup, SoupContexts, SouperIoError},
    utils,
};
use serde_json::{Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Error,
    path::{Path, PathBuf},
};

const GLOBAL_EXCLUDE_DIRS: [&str; 3] = ["node_modules", "bin", "obj"];

pub type SoupParsers = Vec<Box<dyn SoupParse>>;

pub fn scan(
    dir: &PathBuf,
    exclude_dirs: &Vec<PathBuf>,
    default_meta: Map<String, Value>,
) -> Result<SoupContexts, SouperIoError> {
    let path_parsers = match scan_dirs_recursively(dir, exclude_dirs) {
        Ok(path_parsers) => path_parsers,
        Err(e) => {
            return Err(SouperIoError {
                message: format!("Error while scanning directory: {}", e),
            });
        }
    };
    let contexts = path_parsers
        .into_iter()
        .map(|(path, parsers)| {
            let file_content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(e) => {
                    return Err(SouperIoError {
                        message: format!("Not able to read file: {} ({})", path.display(), e),
                    })
                }
            };
            let parse_results: Result<Vec<_>, _> = parsers
                .into_iter()
                .map(|y| y.soups(&file_content, &default_meta))
                .collect();
            let soups = match parse_results {
                Ok(soups_iter) => soups_iter.into_iter().flatten().collect(),
                Err(e) => {
                    return Err(SouperIoError {
                        message: format!("Unable to parse {} due to: {}", path.display(), e),
                    });
                }
            };
            let context_path = relative_path(&path, dir)?;
            Ok((context_path, soups))
        })
        .collect::<Result<BTreeMap<String, BTreeSet<Soup>>, SouperIoError>>()?;
    Ok(SoupContexts { contexts })
}

fn scan_dirs_recursively(
    root: &PathBuf,
    exclude_dirs: &Vec<PathBuf>,
) -> Result<Vec<(PathBuf, SoupParsers)>, Error> {
    let mut sources: Vec<(PathBuf, Vec<Box<dyn SoupParse>>)> = Vec::new();
    'entries: for entry in fs::read_dir(root)? {
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
            let mut content = scan_dirs_recursively(&path, exclude_dirs)?;
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
                    sources.push((path, vec![Box::new(DockerBase {}), Box::new(Apt {})]));
                }
                _ => {}
            }
        }
    }
    Ok(sources)
}

fn relative_path<P: AsRef<Path>>(full_path: P, root_path: P) -> Result<String, SouperIoError> {
    let relative_path = match utils::relative_path(full_path.as_ref(), root_path.as_ref()) {
        Ok(relative_path) => relative_path,
        Err(_e) => {
            return Err(SouperIoError {
                message: format!(
                    "Not able to obtain relative path for: {} (from {})",
                    full_path.as_ref().display(),
                    root_path.as_ref().display()
                ),
            });
        }
    };
    let relative_path = match relative_path.into_os_string().into_string() {
        Ok(path_string) => path_string,
        Err(_) => {
            return Err(SouperIoError {
                message: "Not able to convert relative path to string".to_string(),
            })
        }
    };
    let relative_path = relative_path.replace('\\', "/");
    Ok(relative_path)
}
