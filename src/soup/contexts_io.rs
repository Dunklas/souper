use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufReader, Write, Read};
use std::path::{Path, PathBuf};

use crate::parse::SoupParse;
use crate::soup::model::{Soup, SoupContexts, SouperIoError};
use crate::utils;

impl SoupContexts {
    pub fn from_paths<P: AsRef<Path>>(
        paths: Vec<(PathBuf, Vec<Box<dyn SoupParse>>)>,
        source_dir: P,
        default_meta: Map<String, Value>,
    ) -> Result<SoupContexts, SouperIoError> {
        let mut soup_contexts: BTreeMap<String, BTreeSet<Soup>> = BTreeMap::new();
        for (path, parsers) in paths {
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
            let context_path = relative_path(path.as_ref(), source_dir.as_ref())?;
            soup_contexts.insert(context_path, soups);
        }
        Ok(SoupContexts {
            contexts: soup_contexts,
        })
    }

    pub fn read_from_file(file_path: &PathBuf) -> Result<SoupContexts, SouperIoError> {
        let output_file = match fs::File::open(file_path) {
            Ok(file) => file,
            Err(e) => {
                return Err(SouperIoError {
                    message: format!(
                        "Not able to open file: {} ({})",
                        file_path.display(),
                        e
                    ),
                });
            }
        };
        let reader = BufReader::new(output_file);
        SoupContexts::read(reader)
    }

    fn read<R>(reader: R) -> Result<SoupContexts, SouperIoError> where R: Read {
        let contexts: BTreeMap<String, BTreeSet<Soup>> = match serde_json::from_reader(reader) {
            Ok(contexts) => contexts,
            Err(e) => {
                return Err(SouperIoError {
                    message: format!(
                        "Not able to read output-file: {} ",
                        e
                    ),
                });
            }
        };
        Ok(SoupContexts { contexts })
    }

    pub fn write_to_file(&self, file_path: &PathBuf) -> Result<(), SouperIoError> {
        let mut output_file = match fs::File::create(file_path) {
            Ok(file) => file,
            Err(e) => {
                return Err(SouperIoError {
                    message: format!(
                        "Not able to create file: {} ({})",
                        file_path.display(),
                        e
                    )
                });
            }
        };
        self.write(&mut output_file)
    }

    fn write<W>(&self, writer: &mut W) -> Result<(), SouperIoError> where W: Write {
        let json = match serde_json::to_string_pretty(&self.contexts()) {
            Ok(json) => json,
            Err(e) => {
                return Err(SouperIoError {
                    message: format!("Not able to serialize to json: {}", e),
                })
            }
        };
        match writer.write_all(json.as_bytes()) {
            Ok(_x) => Ok(()),
            Err(e) => {
                return Err(SouperIoError {
                    message: format!(
                        "Not able to write output-file: {}",
                        e
                    )
                })
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read() {
        let input = r#"
{
    "src/package.json": [
        {
            "name": "some-dependency",
            "version": "6.6.6",
            "meta": {}
        },
        {
            "name": "another-dependency",
            "version": "42",
            "meta": {
                "rationale": "Do this and that"
            }
        }
    ],
    "src/Dockerfile": [
        {
            "name": "some-image",
            "version": "6.0-jammy",
            "meta": {}
        }
    ]
}
        "#;
        let result = SoupContexts::read(input.as_bytes());

        assert_eq!(true, result.is_ok());
        let contexts = result.unwrap().contexts;
        assert_eq!(true, contexts.contains_key("src/package.json"));
        assert_eq!(
            vec![
                Soup { name: "some-dependency".to_owned(), version: "6.6.6".to_owned(), meta: Map::new() },
                Soup { name: "another-dependency".to_owned(), version: "42".to_owned(), meta: serde_json::json!({ "rationale": "Do this and that" }).as_object().unwrap().clone()}
            ].into_iter().collect::<BTreeSet<Soup>>(),
            *contexts.get("src/package.json").unwrap()
        );
        assert_eq!(true, contexts.contains_key("src/Dockerfile"));
        assert_eq!(
            vec![
                Soup { name: "some-image".to_owned(), version: "6.0-jammy".to_owned(), meta: Map::new() }
            ].into_iter().collect::<BTreeSet<Soup>>(),
            *contexts.get("src/Dockerfile").unwrap()
        )
    }

    #[test]
    fn write() {
        let input = SoupContexts {
            contexts: vec![
                ("src/package.json".to_owned(), vec![
                    Soup { name: "some-dependency".to_owned(), version: "6.6.6".to_owned(), meta: Map::new() }
                ].into_iter().collect::<BTreeSet<Soup>>()),
                ("src/Dockerfile".to_owned(), vec![
                    Soup { name: "some-image".to_owned(), version: "6.0-jammy".to_owned(), meta: serde_json::json!({"rationale": "Do this and that" }).as_object().unwrap().clone() }
                ].into_iter().collect::<BTreeSet<Soup>>())
            ].into_iter().collect::<BTreeMap<String, BTreeSet<Soup>>>()
        };

        let mut buffer = Vec::<u8>::new();
        input.write(&mut buffer).unwrap();
        assert_eq!(
            r#"{
  "src/Dockerfile": [
    {
      "name": "some-image",
      "version": "6.0-jammy",
      "meta": {
        "rationale": "Do this and that"
      }
    }
  ],
  "src/package.json": [
    {
      "name": "some-dependency",
      "version": "6.6.6",
      "meta": {}
    }
  ]
}"#.to_owned().trim(),
            String::from_utf8(buffer).unwrap().trim()
        );
    }
}
