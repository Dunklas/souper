use super::SoupParse;
use crate::soup::model::{Soup, SoupSourceParseError};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub struct DockerBase {}

lazy_static! {
    static ref BASE_PATTERN: Regex = Regex::new(r"^ *FROM +(?:--platform=[\w/]+ +)?(?P<name>[\w\-\./]+):(?P<version>[\w\.-]+) *(?:AS +[\w\-]+)? *$").unwrap();
}

impl SoupParse for DockerBase {
    fn soups(
        &self,
        content: &str,
        default_meta: &Map<String, Value>,
    ) -> Result<BTreeSet<Soup>, SoupSourceParseError> {
        let mut result: BTreeSet<Soup> = BTreeSet::new();
        let lines = content.lines();
        for line in lines {
            if let Some(captures) = BASE_PATTERN.captures(line) {
                let name = captures.name("name").unwrap().as_str();
                let version = captures.name("version").unwrap().as_str();
                result.insert(Soup {
                    name: name.to_owned(),
                    version: version.to_owned(),
                    meta: default_meta.clone(),
                });
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_base() {
        let content = r#"
FROM postgres:14.4
        "#;

        let result = DockerBase {}.soups(content, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        let expected_soup = Soup {
            name: "postgres".to_owned(),
            version: "14.4".to_owned(),
            meta: Map::new(),
        };
        assert_eq!(true, soups.contains(&expected_soup));
    }

    #[test]
    fn named_base() {
        let content = r#"
FROM mcr.microsoft.com/dotnet/sdk:6.0 AS build-env
        "#;

        let result = DockerBase {}.soups(content, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        let expected_soup = Soup {
            name: "mcr.microsoft.com/dotnet/sdk".to_owned(),
            version: "6.0".to_owned(),
            meta: Map::new(),
        };
        assert_eq!(true, soups.contains(&expected_soup));
    }

    #[test]
    fn with_platform() {
        let content = r#"
FROM --platform=linux/x86_64 mcr.microsoft.com/dotnet/sdk:6.0 AS build-env
        "#;

        let result = DockerBase {}.soups(content, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        let expected_soup = Soup {
            name: "mcr.microsoft.com/dotnet/sdk".to_owned(),
            version: "6.0".to_owned(),
            meta: Map::new(),
        };
        assert_eq!(true, soups.contains(&expected_soup));
    }

    #[test]
    fn no_from_statement() {
        let content = r#"
COPY --chown app:app . ./
        "#;
        let result = DockerBase {}.soups(content, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(0, soups.len());
    }
}
