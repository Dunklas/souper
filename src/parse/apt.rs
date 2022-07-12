use super::SoupParse;
use crate::soup::model::{Soup, SoupSourceParseError};
use lazy_static::lazy_static;
use regex::{Regex, RegexSet};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub struct Apt {}

static PATTERNS: [&str; 1] = [r"^apt install (?:\-[a-zA-Z\-]{1} )*(?P<name>[a-zA-Z0-9]+)=(?P<version>[a-zA-Z0-9\.\-_]+)$"];
lazy_static! {
    static ref PATTERN_SET: RegexSet = RegexSet::new(&PATTERNS).unwrap();
    static ref REGEXES: Vec<Regex> = PATTERN_SET
        .patterns()
        .iter()
        .map(|pat| Regex::new(pat).unwrap())
        .collect();
}

impl SoupParse for Apt {
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
                        version: named_capture(&captures, "version")?,
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

    #[test_case("apt install curl=7.81.0-1ubuntu1.3")]
    #[test_case("apt install -y curl=7.81.0-1ubuntu1.3")]
    fn specific_version(input: &str) {
        let result = Apt {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(1, soups.len());
        let soup = soups.into_iter().last().unwrap();
        assert_eq!(
            Soup {
                name: "curl".to_owned(),
                version: "7.81.0-1ubuntu1.3".to_owned(),
                meta: Map::new()
            },
            soup
        )
    }
}
