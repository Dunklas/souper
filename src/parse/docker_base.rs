use super::SoupParse;
use crate::soup::model::{Soup, SoupSourceParseError};
use lazy_static::lazy_static;
use regex::{Regex, RegexSet};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub struct DockerBase {}

static PATTERNS: [&'static str; 1] = [
    r"^FROM (?:--platform=[\w/]+ )?(?P<name>(?:[a-z0-9\.\-_]+){1}(?:/[a-z0-9\.\-_]+)*)[:@](?P<tag>[a-zA-Z0-9\.-_]+)(?: AS [\w\-]+)?$"
];

lazy_static! {
    static ref PATTERN_SET: RegexSet = RegexSet::new(&PATTERNS).unwrap();
    static ref REGEXES: Vec<Regex> = PATTERN_SET.patterns().iter()
        .map(|pat| Regex::new(pat).unwrap())
        .collect();
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
            let matching_patterns = PATTERN_SET.matches(line).into_iter()
                .map(|match_id| &REGEXES[match_id])
                .collect::<Vec<&Regex>>();
            if let Some(pattern) = matching_patterns.first() {
                if let Some(captures) = pattern.captures(line) {
                    result.insert(Soup {
                        name: named_capture(&captures, "name")?,
                        version: named_capture(&captures, "tag")?,
                        meta: default_meta.clone(),
                    });
                }
            }
        }
        Ok(result)
    }
}

fn named_capture(captures: &regex::Captures, name: &str) -> Result<String, SoupSourceParseError> {
    match captures.name(name) {
        Some(value) => Ok(value.as_str().to_owned()),
        None => Err(SoupSourceParseError {
            message: "Unable to parse FROM statement in dockerfile".to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_name() {
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
    fn multiple_parts_name() {
        let content = r#"
FROM fedora/httpd:v1.0.0
        "#;
        
        let result = DockerBase {}.soups(content, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(
            true,
            soups.contains(&Soup { name: "fedora/httpd".to_owned(), version: "v1.0.0".to_owned(), meta: Map::new() })
        )
    }

    #[test]
    fn with_digest() {
        let content = r#"
FROM fedora/httpd@ca468b84b84846e84
        "#;
        
        let result = DockerBase {}.soups(content, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(
            true,
            soups.contains(&Soup { name: "fedora/httpd".to_owned(), version: "ca468b84b84846e84".to_owned(), meta: Map::new() })
        );
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
