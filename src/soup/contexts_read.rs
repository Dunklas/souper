use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::soup::model::{Soup, SoupContexts, SouperIoError};
use crate::parse::{
    SoupSource,
    package_json::PackageJson
};
use crate::utils;

impl SoupContexts {
    pub fn from_paths<P: AsRef<Path>>(
        paths: Vec<PathBuf>,
        source_dir: P,
    ) -> Result<SoupContexts, SouperIoError> {
        let mut soup_contexts: BTreeMap<String, BTreeSet<Soup>> = BTreeMap::new();
        for path in paths {
            let file = match File::open(&path) {
                Ok(file) => file,
                Err(e) => return Err(SouperIoError{
                    message: format!("Not able to open file: {} ({})", path.display(), e)
                })
            };
            let reader = BufReader::new(file);
            let filename = match path.file_name() {
                None => {
                    return Err(SouperIoError{
                        message: format!("Not able to obtain filename for path: {}", path.display())
                    });
                }
                Some(filename) => match filename.to_str() {
                    Some(filename) => filename,
                    None => {
                        return Err(SouperIoError{
                            message: format!("Not able to convert filename to string")
                        })
                    }
                }
            };
            let soups = match filename {
                    "package.json" => PackageJson::soups(reader),
                    _ => {
                        panic!("No parser found for: {:?}", filename)
                    }
            };
            let relative_path = match utils::relative_path(path.as_ref(), source_dir.as_ref()) {
                Ok(relative_path) => relative_path,
                Err(_e) => {
                    return Err(SouperIoError {
                        message: format!("Not able to obtain relative path for: {} (from {})", path.display(), source_dir.as_ref().display())
                    });
                }
            };
            let relative_path = relative_path.into_os_string();
            let relative_path = match relative_path.into_string() {
                Ok(path_string) => path_string,
                Err(_) => {
                    return Err(SouperIoError{
                        message: format!("Not able to convert relative path to string")
                    })
                }
            };
            let relative_path = relative_path.replace("\\", "/");
            soup_contexts.insert(relative_path, soups);
        }
        Ok(SoupContexts {
            contexts: soup_contexts,
        })
    }

    pub fn from_output_file<P: AsRef<Path>>(file_path: P) -> Result<SoupContexts, SouperIoError> {
        let output_file = match File::open(file_path.as_ref()) {
            Ok(file) => file,
            Err(e) => {
                return Err(SouperIoError{
                    message: format!("Not able to open file: {} ({})", file_path.as_ref().display(), e)
                });
            }
        };
        let reader = BufReader::new(output_file);
        let contexts: BTreeMap<String, BTreeSet<Soup>> = match serde_json::from_reader(reader) {
            Ok(contexts) => contexts,
            Err(e) => {
                return Err(SouperIoError{
                    message: format!("Not able to parse file: {} ({})", file_path.as_ref().display(), e)
                });
            }
        };
        Ok(SoupContexts { contexts })
    }
}
