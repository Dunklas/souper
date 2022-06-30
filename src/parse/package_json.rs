use std::{
    collections,
    fs,
    io,
    path
};
use serde::{
    Deserialize,
    Serialize
};
use crate::soup;
use super::DependencyParse;

pub struct PackageJsonParser {
    path: path::PathBuf
}

impl PackageJsonParser {
    pub fn new(path: path::PathBuf) -> PackageJsonParser {
        PackageJsonParser {
            path
        }
    }
}

impl DependencyParse for PackageJsonParser {
    fn parse(&self) -> crate::soup::SoupContext {
        let file = fs::File::open(&self.path).unwrap();
        let reader = io::BufReader::new(file);
        let p: PackageJson = serde_json::from_reader(reader).unwrap();
        let soups = p.dependencies.into_iter()
            .map(|(key, value)| soup::Soup {
                name: key,
                version: value,
                meta: collections::HashMap::new()
            })
            .collect::<Vec<soup::Soup>>();
        return soup::SoupContext {
            path: path::PathBuf::from(&self.path),
            soups
        };
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct PackageJson {
    dependencies: collections::HashMap<String, String>
}

