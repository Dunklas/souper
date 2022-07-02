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
use crate::parse::{
    SoupSource,
    package_json::{PackageJson}
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Soup {
    pub name: String,
    pub version: String,
    pub meta: collections::HashMap<String, serde_json::Value>
}

impl PartialEq for Soup {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version
    }
}
impl Eq for Soup {}

#[derive(Serialize, Deserialize, Debug)]
pub struct SoupContext {
    pub path: path::PathBuf,
    pub soups: Vec<Soup>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SoupContexts {
    contexts: Vec<SoupContext>
}

impl SoupContexts {
    pub fn empty() -> SoupContexts {
        SoupContexts { contexts: Vec::new() }
    }

    pub fn from_paths<>(paths: Vec<path::PathBuf>) -> SoupContexts {
        let mut soup_contexts: Vec<SoupContext> = Vec::new();
        for path in paths {
            let file = fs::File::open(&path).unwrap();
            let reader = io::BufReader::new(file);
            let soups = match path.file_name() {
                None => {
                    panic!("No filename for path: {:?}", path);
                },
                Some(filename) => match filename.to_str().unwrap() {
                    "package.json" => PackageJson::soups(reader),
                    _ => {
                        panic!("No parser found for: {:?}", filename)
                    }
                }
            };
            soup_contexts.push(SoupContext {
                path,
                soups
            })
        }
        SoupContexts{
            contexts: soup_contexts
        }
    }

    pub fn from_output_file<P: AsRef<path::Path>>(file_path: P) -> SoupContexts {
        let output_file = fs::File::open(file_path).unwrap();
        let reader = io::BufReader::new(output_file);
        let contexts: Vec<SoupContext> = serde_json::from_reader(reader).unwrap();
        SoupContexts {
            contexts
        }
    }

    pub fn vec(&self) -> &Vec<SoupContext> {
        &self.contexts
    } 
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn soup_equal() {
        let s1 = Soup{
            name: "some-dependency".to_owned(),
            version: "1.0.0".to_owned(),
            meta: collections::HashMap::new()
        };
        let mut meta = collections::HashMap::new();
        meta.insert("requirement".to_owned(), json!("should do this and that"));
        let s2 = Soup{
            name: "some-dependency".to_owned(),
            version: "1.0.0".to_owned(),
            meta
        };
        assert_eq!(s1, s2);
    }

    #[test]
    fn soup_not_equal() {
        let s1 = Soup{
            name: "some-dependency".to_owned(),
            version: "1.0.0".to_owned(),
            meta: collections::HashMap::new()
        };
        let s2 = Soup{
            name: "some-dependency".to_owned(),
            version: "1.0.1".to_owned(),
            meta: collections::HashMap::new()
        };
        assert_ne!(s1, s2);
    }
}
