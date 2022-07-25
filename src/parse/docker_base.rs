use super::SoupParse;
use crate::soup::model::{Soup, SoupSourceParseError};
use lazy_static::lazy_static;
use regex::{Regex, RegexSet};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub struct DockerBase {}

static PATTERNS: [&str; 2] = [
    r"^(?i)FROM(?-i) (?:--platform=[\w/]+ )?(?P<name>(?:[a-z0-9\.\-_]+){1}(?:/[a-z0-9\.\-_]+)*)[:@](?P<tag>[a-zA-Z0-9\.\-_]+)(?: (?i)AS(?-i) [\w\-]+)?$",
    r"^(?i)FROM(?-i) (?:--platform=[\w/]+ )?(?P<name>(?:[a-z0-9\.\-_]+){1}:[0-9]+(?:/[a-z0-9\.\-_]+)*)[:@](?P<tag>[a-zA-Z0-9\.\-_]+)(?: (?i)AS(?-i) [\w\-]+)?$",
];

lazy_static! {
    static ref PATTERN_SET: RegexSet = RegexSet::new(&PATTERNS).unwrap();
    static ref REGEXES: Vec<Regex> = PATTERN_SET
        .patterns()
        .iter()
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
            let matching_patterns = PATTERN_SET
                .matches(line)
                .into_iter()
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
    use test_case::test_case;

    #[test_case("FROM postgres:14.4")]
    #[test_case("FROM --platform=linux/x86_64 postgres:14.4")]
    #[test_case("FROM postgres:14.4 AS build-env")]
    fn simple_image_name(input: &str) {
        let result = DockerBase {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(
            true,
            soups.contains(&Soup {
                name: "postgres".to_owned(),
                version: "14.4".to_owned(),
                meta: Map::new()
            })
        );
    }

    #[test_case("FROM fedora/httpd:v1.6.2")]
    #[test_case("FROM --platform=linux/i686 fedora/httpd:v1.6.2")]
    #[test_case("FROM fedora/httpd:v1.6.2 AS some-name")]
    fn multiple_parts_image_name(input: &str) {
        let result = DockerBase {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(
            true,
            soups.contains(&Soup {
                name: "fedora/httpd".to_owned(),
                version: "v1.6.2".to_owned(),
                meta: Map::new()
            })
        )
    }

    #[test_case("FROM mcr.microsoft.com/dotnet/sdk:6.0")]
    #[test_case("FROM --platform=linux/x86_64 mcr.microsoft.com/dotnet/sdk:6.0")]
    #[test_case("FROM mcr.microsoft.com/dotnet/sdk:6.0 AS build-env")]
    fn hostname_image_name(input: &str) {
        let result = DockerBase {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(
            true,
            soups.contains(&Soup {
                name: "mcr.microsoft.com/dotnet/sdk".to_owned(),
                version: "6.0".to_owned(),
                meta: Map::new()
            })
        );
    }

    #[test_case("FROM mcr.microsoft.com:443/dotnet/sdk:6.0")]
    #[test_case("FROM --platform=linux/x86_64 mcr.microsoft.com:443/dotnet/sdk:6.0")]
    #[test_case("FROM mcr.microsoft.com:443/dotnet/sdk:6.0 AS build-env")]
    fn hostname_port_image_name(input: &str) {
        let result = DockerBase {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(
            true,
            soups.contains(&Soup {
                name: "mcr.microsoft.com:443/dotnet/sdk".to_owned(),
                version: "6.0".to_owned(),
                meta: Map::new()
            })
        );
    }

    #[test_case("FROM fedora@ca468b84b84846e84", "fedora")]
    #[test_case("FROM fedora/httpd@ca468b84b84846e84", "fedora/httpd")]
    #[test_case(
        "FROM mcr.microsoft.com/dotnet/sdk@ca468b84b84846e84",
        "mcr.microsoft.com/dotnet/sdk"
    )]
    #[test_case(
        "FROM mcr.microsoft.com:443/dotnet/sdk@ca468b84b84846e84",
        "mcr.microsoft.com:443/dotnet/sdk"
    )]
    fn with_digest(input: &str, expected_name: &str) {
        let result = DockerBase {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(
            true,
            soups.contains(&Soup {
                name: expected_name.to_owned(),
                version: "ca468b84b84846e84".to_owned(),
                meta: Map::new()
            })
        );
    }

    #[test_case("from postgres:14.4 as build-env", "postgres", "14.4")]
    #[test_case("from fedora/httpd:v1.6.2 as some-name", "fedora/httpd", "v1.6.2")]
    #[test_case(
        "from mcr.microsoft.com/dotnet/sdk:6.0 as build-env",
        "mcr.microsoft.com/dotnet/sdk",
        "6.0"
    )]
    #[test_case(
        "from mcr.microsoft.com:443/dotnet/sdk:6.0 as build-env",
        "mcr.microsoft.com:443/dotnet/sdk",
        "6.0"
    )]
    fn lower_case(input: &str, expected_name: &str, expected_version: &str) {
        let result = DockerBase {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(
            true,
            soups.contains(&Soup {
                name: expected_name.to_owned(),
                version: expected_version.to_owned(),
                meta: Map::new()
            })
        )
    }

    #[test_case("COPY --chown app:app . ./")]
    #[test_case("")]
    fn no_from_statement(input: &str) {
        let result = DockerBase {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(0, soups.len());
    }
}
