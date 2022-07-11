use super::SoupParse;
use crate::soup::model::{Soup, SoupSourceParseError};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::{BTreeSet, HashMap};

pub struct Cargo {}

#[derive(Deserialize)]
struct Content {
    dependencies: Option<HashMap<String, toml::Value>>,
}

impl SoupParse for Cargo {
    fn soups(
        &self,
        content: &str,
        default_meta: &Map<String, Value>,
    ) -> Result<BTreeSet<Soup>, SoupSourceParseError> {
        let content: Content = match toml::from_str(content) {
            Ok(content) => content,
            Err(e) => {
                return Err(SoupSourceParseError {
                    message: format!("Invalid Cargo.toml ({})", e),
                });
            }
        };
        match content.dependencies {
            None => Ok(BTreeSet::new()),
            Some(dependencies) => dependencies
                .into_iter()
                .map(|(dependency, value)| match value {
                    toml::Value::String(version) => Ok(Soup {
                        name: dependency,
                        version,
                        meta: default_meta.clone(),
                    }),
                    toml::Value::Table(table) => match table.get("version") {
                        Some(version) => Ok(Soup {
                            name: dependency.to_owned(),
                            version: match version.as_str() {
                                Some(v) => v.to_owned(),
                                None => {
                                    return Err(SoupSourceParseError {
                                        message: format!("Invalid version for: {}", dependency),
                                    });
                                }
                            },
                            meta: default_meta.clone(),
                        }),
                        None => {
                            return Err(SoupSourceParseError {
                                message: format!("Missing version for: {}", dependency),
                            });
                        }
                    },
                    _ => Err(SoupSourceParseError {
                        message: format!("Malformed dependency: {}", dependency),
                    }),
                })
                .collect::<Result<BTreeSet<Soup>, _>>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(
        r#"
[dependencies]
serde = "1.0.137"
    "#
    )]
    #[test_case(
        r#"
[dependencies]
serde = { version = "1.0.137", features = ["derive"] }
    "#
    )]
    fn simple_dependency(input: &str) {
        let result = Cargo {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(
            true,
            soups.contains(&Soup {
                name: "serde".to_owned(),
                version: "1.0.137".to_owned(),
                meta: Map::new()
            })
        );
    }

    #[test]
    fn multiple_dependencies() {
        let input = r#"
[dependencies]
serde_json = { version = "1.0.82", features = ["preserve_order"] }
quick-xml = "0.23.0"
        "#;
        let result = Cargo {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(2, soups.len());
        assert_eq!(
            vec![
                Soup {
                    name: "serde_json".to_owned(),
                    version: "1.0.82".to_owned(),
                    meta: Map::new()
                },
                Soup {
                    name: "quick-xml".to_owned(),
                    version: "0.23.0".to_owned(),
                    meta: Map::new()
                }
            ]
            .into_iter()
            .collect::<BTreeSet<_>>(),
            soups
        );
    }

    #[test_case("[dependencies]")]
    #[test_case("")]
    fn no_dependencies(input: &str) {
        let result = Cargo {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(0, soups.len());
    }

    #[test_case(
        r#"
[dependencies]
serde_json = { verzion = "1.0.82", features = ["preserve_order"] }
    "#
    )]
    #[test_case(
        r#"
[dependencies]
serde_json = { features = ["preserve_order"] }
    "#
    )]
    #[test_case(
        r#"
[dependencies]
serde_json = 4.42
    "#
    )]
    #[test_case(
        r#"
[dependencies]
serde_json = { version = { verzion = "1.0.82" }, features = ["preserve_order"] }
    "#
    )]
    #[test_case(r#"dependencies = "serde""#)]
    fn missing_or_invalid_version(input: &str) {
        let result = Cargo {}.soups(input, &Map::new());
        assert_eq!(true, result.is_err());
    }
}
