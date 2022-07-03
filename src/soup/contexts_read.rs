use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use crate::soup::model::{Soup, SoupContexts};
use crate::parse::{
    SoupSource,
    package_json::PackageJson
};
use crate::utils;

impl SoupContexts {
    pub fn from_paths<P: AsRef<Path>>(
        paths: Vec<PathBuf>,
        source_dir: P,
    ) -> SoupContexts {
        let mut soup_contexts: BTreeMap<String, Vec<Soup>> =
            BTreeMap::new();
        for path in paths {
            let file = File::open(&path).unwrap();
            let reader = BufReader::new(file);
            let mut soups = match path.file_name() {
                None => {
                    panic!("No filename for path: {:?}", path);
                }
                Some(filename) => match filename.to_str().unwrap() {
                    "package.json" => PackageJson::soups(reader),
                    _ => {
                        panic!("No parser found for: {:?}", filename)
                    }
                },
            };
            let path = utils::relative_path(path.as_ref(), source_dir.as_ref()).unwrap();
            let path = path.into_os_string().into_string().unwrap();
            let path = path.replace("\\", "/");
            soups.sort();
            soup_contexts.insert(path, soups);
        }
        SoupContexts {
            contexts: soup_contexts,
        }
    }

    pub fn from_output_file<P: AsRef<Path>>(file_path: P) -> SoupContexts {
        let output_file = File::open(file_path).unwrap();
        let reader = BufReader::new(output_file);
        let contexts: BTreeMap<String, Vec<Soup>> =
            serde_json::from_reader(reader).unwrap();
        SoupContexts { contexts }
    }
}
