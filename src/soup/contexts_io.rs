use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use serde_json;

use crate::soup::model::{Soup, SoupContexts, SouperIoError};
use crate::parse::{
    SoupSource,
    package_json::PackageJson,
    csproj::CsProj,
    docker_base::DockerBase
};
use crate::utils;

impl SoupContexts {
    pub fn from_paths<P: AsRef<Path>>(
        paths: Vec<PathBuf>,
        source_dir: P,
        default_meta: serde_json::Value
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
                Some(filename) => filename,
                None => {
                    return Err(SouperIoError{
                        message: format!("Not able to obtain filename for path: {}", path.display())
                    });
                }
            };
            let parse_result = match filename.to_str() {
                    Some("package.json") => PackageJson::soups(reader, &default_meta),
                    Some(x) if x.contains(".csproj") => CsProj::soups(reader, &default_meta),
                    Some(x) if x.contains("Dockerfile") => DockerBase::soups(reader, &default_meta),
                    _ => {
                        panic!("No parser found for: {:?}", filename)
                    }
            };
            let soups = match parse_result {
                Ok(soups) => soups,
                Err(e) => {
                    return Err(SouperIoError{
                        message: format!("Unable to parse {} due to: {}", path.display(), e)
                    });
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
                        message: "Not able to convert relative path to string".to_string()
                    })
                }
            };
            let relative_path = relative_path.replace('\\', "/");
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

    pub fn write_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<(), SouperIoError> {
        let mut output_file = match File::create(&file_path) {
            Ok(file) => file,
            Err(e) => return Err(SouperIoError{
                message: format!("Not able to create file: {} ({})", file_path.as_ref().display(), e)
            })
        };
        let json = match serde_json::to_string_pretty(&self.contexts) {
            Ok(json) => json,
            Err(e) => return Err(SouperIoError{
                message: format!("Not able to serialize to json: {}", e)
            })
        };
        match output_file.write_all(json.as_bytes()) {
            Ok(_x) => Ok(()),
            Err(e) => return Err(SouperIoError{
                message: format!("Not able to write to file: {} ({})", file_path.as_ref().display(), e)
            }),
        }
    }
}
