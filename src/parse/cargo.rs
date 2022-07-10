use super::SoupParse;
use crate::soup::model::{Soup, SoupSourceParseError};
use serde_json::{Map, Value};
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap, BTreeMap};

pub struct Cargo {}

#[derive(Deserialize)]
struct Content {
    dependencies: HashMap<String, toml::Value>
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
                return Err(SoupSourceParseError{
                    message: format!("Invalid Cargo.toml ({})", e)
                });
            }
        };
        Ok(content.dependencies.into_iter()
            .map(|(dependency, value)| {
                match value {
                    toml::Value::String(str) => Soup {
                        name: dependency,
                        version: str.to_owned(),
                        meta: default_meta.clone()
                    },
                    _ => {
                        panic!("hej");
                    }
                }
            })
            .collect::<BTreeSet<Soup>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_dependency() {
        let content = r#"
[dependencies]
some-dep = "6.6.6"
        "#;
        let result = Cargo {}.soups(content, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(
            true,
            soups.contains(&Soup {
                name: "some-dep".to_owned(),
                version: "6.6.6".to_owned(),
                meta: Map::new()
            })
        );
    }
}
