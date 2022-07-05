use std::{
    collections::BTreeSet,
    io
};
use serde_json::json;
use regex::Regex;
use lazy_static::lazy_static;
use super::SoupSource;
use crate::soup::model::{Soup, SoupSourceParseError};

pub struct DockerBase {}

lazy_static! {
    static ref BASE_PATTERN: Regex = Regex::new(r"^ *FROM +(?P<name>[a-zA-Z0-9\-/\.]+):(?P<version>[a-zA-Z0-9\._-]+) *(?:AS +[a-zA-Z0-9\-]+)? *$").unwrap();
}

impl<R> SoupSource<R> for DockerBase
where
    R: io::BufRead,
{
    fn soups(reader: R) -> Result<BTreeSet<Soup>, SoupSourceParseError> {
        Ok(reader.lines()
            .filter_map(|line| line.ok())
            .filter(|line| BASE_PATTERN.is_match(&line))
            .map(|line| {
                match BASE_PATTERN.captures(&line) {
                    Some(captures) => {
                        let name = captures.name("name").unwrap().as_str();
                        let version = captures.name("version").unwrap().as_str();
                        Some(Soup{
                            name: name.to_owned(),
                            version: version.to_owned(),
                            meta: json!({})
                        })
                    },
                    None => None
                }
            })
            .filter(|soup| soup.is_some())
            .map(|soup| soup.unwrap())
            .collect::<BTreeSet<Soup>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn simple_base() {
        let content = r#"
FROM postgres:14.4
        "#.as_bytes();

        let result = DockerBase::soups(content);
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        let expected_soup = Soup {
            name: "postgres".to_owned(),
            version: "14.4".to_owned(),
            meta: json!({})
        };
        assert_eq!(1, soups.len());
        assert_eq!(true, soups.contains(&expected_soup));
    }

    #[test]
    fn named_base() {
        let content = r#"
FROM mcr.microsoft.com/dotnet/sdk:6.0 AS build-env
        "#.as_bytes();

        let result = DockerBase::soups(content);
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(1, soups.len());
        let expected_soup = Soup {
            name: "mcr.microsoft.com/dotnet/sdk".to_owned(),
            version: "6.0".to_owned(),
            meta: json!({})
        };
        assert_eq!(true, soups.contains(&expected_soup));
    }
}