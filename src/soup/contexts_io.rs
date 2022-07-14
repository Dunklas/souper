use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;

use crate::soup::model::{Soup, SoupContexts, SouperIoError};

impl SoupContexts {
    pub fn read_from_file(file_path: &PathBuf) -> Result<SoupContexts, SouperIoError> {
        let output_file = match fs::File::open(file_path) {
            Ok(file) => file,
            Err(e) => {
                return Err(SouperIoError {
                    message: format!("Not able to open file: {} ({})", file_path.display(), e),
                });
            }
        };
        let reader = BufReader::new(output_file);
        SoupContexts::read(reader)
    }

    fn read<R>(reader: R) -> Result<SoupContexts, SouperIoError>
    where
        R: Read,
    {
        let contexts: BTreeMap<String, BTreeSet<Soup>> = match serde_json::from_reader(reader) {
            Ok(contexts) => contexts,
            Err(e) => {
                return Err(SouperIoError {
                    message: format!("Not able to read output-file: {} ", e),
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
                    message: format!("Not able to create file: {} ({})", file_path.display(), e),
                });
            }
        };
        self.write(&mut output_file)
    }

    fn write<W>(&self, writer: &mut W) -> Result<(), SouperIoError>
    where
        W: Write,
    {
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
                    message: format!("Not able to write output-file: {}", e),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;

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
                Soup {
                    name: "some-dependency".to_owned(),
                    version: "6.6.6".to_owned(),
                    meta: Map::new()
                },
                Soup {
                    name: "another-dependency".to_owned(),
                    version: "42".to_owned(),
                    meta: serde_json::json!({ "rationale": "Do this and that" })
                        .as_object()
                        .unwrap()
                        .clone()
                }
            ]
            .into_iter()
            .collect::<BTreeSet<Soup>>(),
            *contexts.get("src/package.json").unwrap()
        );
        assert_eq!(true, contexts.contains_key("src/Dockerfile"));
        assert_eq!(
            vec![Soup {
                name: "some-image".to_owned(),
                version: "6.0-jammy".to_owned(),
                meta: Map::new()
            }]
            .into_iter()
            .collect::<BTreeSet<Soup>>(),
            *contexts.get("src/Dockerfile").unwrap()
        )
    }

    #[test]
    fn write() {
        let input = SoupContexts {
            contexts: vec![
                (
                    "src/package.json".to_owned(),
                    vec![Soup {
                        name: "some-dependency".to_owned(),
                        version: "6.6.6".to_owned(),
                        meta: Map::new(),
                    }]
                    .into_iter()
                    .collect::<BTreeSet<Soup>>(),
                ),
                (
                    "src/Dockerfile".to_owned(),
                    vec![Soup {
                        name: "some-image".to_owned(),
                        version: "6.0-jammy".to_owned(),
                        meta: serde_json::json!({"rationale": "Do this and that" })
                            .as_object()
                            .unwrap()
                            .clone(),
                    }]
                    .into_iter()
                    .collect::<BTreeSet<Soup>>(),
                ),
            ]
            .into_iter()
            .collect::<BTreeMap<String, BTreeSet<Soup>>>(),
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
}"#
            .to_owned()
            .trim(),
            String::from_utf8(buffer).unwrap().trim()
        );
    }

    #[test]
    fn write_no_content() {
        let input = SoupContexts{
            contexts: vec![
                (
                    "src/package.json".to_owned(),
                    vec![].into_iter().collect::<BTreeSet<Soup>>()
                )
            ]
            .into_iter()
            .collect::<BTreeMap<String, BTreeSet<Soup>>>()
        };
        let mut buffer = Vec::<u8>::new();
        input.write(&mut buffer).unwrap();
        assert_eq!("{}".to_owned(), String::from_utf8(buffer).unwrap())
    }
}
