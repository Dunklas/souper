use std::{
    cmp,
    collections,
    fs,
    io,
    path
};
use serde::{
    Deserialize,
    Serialize
};
use crate::parse::{
    SoupSource,
    package_json
};

#[derive(Serialize, Deserialize, Debug, cmp::Eq, cmp::PartialEq)]
pub struct Soup {
    pub name: String,
    pub version: String,
    pub meta: collections::HashMap<String, serde_json::Value>
}

#[derive(Serialize, Deserialize)]
pub struct SoupContext {
    pub path: path::PathBuf,
    pub soups: Vec<Soup>
}

#[derive(Serialize, Deserialize)]
pub struct SoupContexts {
    contexts: Vec<SoupContext>
}

impl SoupContexts {
    pub fn from_paths(paths: Vec<path::PathBuf>) -> SoupContexts {
        let mut soup_contexts: Vec<SoupContext> = Vec::new();
        for path in paths {
            let file = fs::File::open(&path).unwrap();
            let reader = io::BufReader::new(file);
            let soups = package_json::PackageJson::soups(reader);
            soup_contexts.push(SoupContext { path: path, soups: soups })
        }
        SoupContexts{
            contexts: soup_contexts
        }
    }

    pub fn vec(&self) -> &Vec<SoupContext> {
        &self.contexts
    } 
}
