use super::SoupParse;
use crate::soup::model::{Soup, SoupSourceParseError};
use lazy_static::lazy_static;
use regex::{Regex, RegexSet};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub struct Apt {}

static PATTERNS: [&str; 2] = [
    r"^apt(?:\-get)? install (?:\-[a-zA-Z\-]{1} )*(?:\-\-[a-zA-Z\-]+ )*(?P<name>[a-zA-Z0-9\-\._]+)=(?P<version>[a-zA-Z0-9\.\-_]+)$",
    r"^apt(?:\-get)? install (?:\-[a-zA-Z\-]{1} )*(?:\-\-[a-zA-Z\-]+ )*(?P<name>[a-zA-Z0-9\-\._]+)$",
];
lazy_static! {
    static ref PATTERN_SET: RegexSet = RegexSet::new(&PATTERNS).unwrap();
    static ref REGEXES: Vec<Regex> = PATTERN_SET
        .patterns()
        .iter()
        .map(|pat| Regex::new(pat).unwrap())
        .collect();
    static ref LINE_CONTINUATION: Regex = Regex::new(r"\\.*\n|\r\n").unwrap();
    static ref MULTI_SPACE: Regex = Regex::new(r"[ \t]+").unwrap();
}

impl SoupParse for Apt {
    fn soups(
        &self,
        content: &str,
        default_meta: &Map<String, Value>,
    ) -> Result<BTreeSet<Soup>, SoupSourceParseError> {
        let mut result: BTreeSet<Soup> = BTreeSet::new();
        let content = normalize(content);
        for line in content.lines() {
            let matching_patterns = PATTERN_SET
                .matches(line)
                .into_iter()
                .map(|match_id| &REGEXES[match_id])
                .collect::<Vec<&Regex>>();
            if let Some(pattern) = matching_patterns.first() {
                if let Some(captures) = pattern.captures(line) {
                    result.insert(Soup {
                        name: named_capture(&captures, "name")?,
                        version: match named_capture(&captures, "version") {
                            Ok(version) => version,
                            Err(_e) => "unknown".to_owned()
                        },
                        meta: default_meta.clone(),
                    });
                }
            }
        }
        Ok(result)
    }
}

fn normalize(input: &str) -> String {
    let result = LINE_CONTINUATION.replace_all(input, " ");
    let result = MULTI_SPACE.replace_all(&result, " ");
    result.to_string()
}

fn named_capture(captures: &regex::Captures, name: &str) -> Result<String, SoupSourceParseError> {
    match captures.name(name) {
        Some(value) => Ok(value.as_str().to_owned()),
        None => Err(SoupSourceParseError {
            message: "Unable to parse apt install statement".to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("apt install curl=7.81.0-1ubuntu1.3")]
    #[test_case("apt install -y -q curl=7.81.0-1ubuntu1.3")]
    #[test_case("apt install --asume-yes --quiet curl=7.81.0-1ubuntu1.3")]
    #[test_case("apt install -y --quiet curl=7.81.0-1ubuntu1.3")]
    #[test_case("apt-get install curl=7.81.0-1ubuntu1.3")]
    #[test_case("apt-get install -y -q curl=7.81.0-1ubuntu1.3")]
    #[test_case("apt-get install --asume-yes --quiet curl=7.81.0-1ubuntu1.3")]
    #[test_case("apt-get install -y --quiet curl=7.81.0-1ubuntu1.3")]
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

    #[test_case("apt install curl")]
    #[test_case("apt install -y -q curl")]
    #[test_case("apt install --asume-yes --quiet curl")]
    #[test_case("apt install -y --quiet curl")]
    #[test_case("apt-get install curl")]
    #[test_case("apt-get install -y -q curl")]
    #[test_case("apt-get install --asume-yes --quiet curl")]
    #[test_case("apt-get install -y --quiet curl")]
    fn unspecified_version(input: &str) {
        let result = Apt {}.soups(input, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(1, soups.len());
        let soup = soups.into_iter().last().unwrap();
        assert_eq!(
            Soup {
                name: "curl".to_owned(),
                version: "unknown".to_owned(),
                meta: Map::new()
            },
            soup
        )
    }

    #[test_case("apt install --assume-yes \\\n\t--quiet curl=7.81.0-1ubuntu1.3")]
    #[test_case("apt install --assume-yes \\\r\n\t--quiet curl=7.81.0-1ubuntu1.3")]
    #[test_case("apt install --assume-yes \\ # Some comment\n\t--quiet curl=7.81.0-1ubuntu1.3")]
    #[test_case("apt install --assume-yes \\ # Some comment\r\n\t--quiet curl=7.81.0-1ubuntu1.3")]
    fn multi_line(input: &str) {
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
        );
    }
}
