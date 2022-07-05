use std::{
    collections::{
        BTreeMap,
        BTreeSet
    },
    fmt
};
use serde::{
    Deserialize,
    Serialize
};
use serde_json;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Soup {
    pub name: String,
    pub version: String,
    pub meta: serde_json::Value
}

impl PartialEq for Soup {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.version == other.version
    }
}
impl Eq for Soup {}

impl PartialOrd for Soup {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Soup {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.name.cmp(&other.name) {
            std::cmp::Ordering::Equal => self.version.cmp(&other.version),
            _ => self.name.cmp(&other.name)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SoupContexts {
    pub contexts: BTreeMap<String, BTreeSet<Soup>>
}

impl SoupContexts {
    pub fn empty() -> SoupContexts {
        SoupContexts { contexts: BTreeMap::new() }
    }
}

pub struct SouperIoError {
    pub message: String
}

impl fmt::Display for SouperIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IO error: {}", self.message)
    }
}

impl fmt::Debug for SouperIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

pub struct SoupSourceParseError {
    pub message: String
}

impl fmt::Display for SoupSourceParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for SoupSourceParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn soup_equal() {
        let s1 = Soup{
            name: "some-dependency".to_owned(),
            version: "1.0.0".to_owned(),
            meta: json!({})
        };
        let s2 = Soup{
            name: "some-dependency".to_owned(),
            version: "1.0.0".to_owned(),
            meta: json!("{\"requirement\": \"should do this and that\"}")
        };
        assert_eq!(s1, s2);
    }

    #[test]
    fn soup_not_equal() {
        let s1 = Soup{
            name: "some-dependency".to_owned(),
            version: "1.0.0".to_owned(),
            meta: json!({})
        };
        let s2 = Soup{
            name: "some-dependency".to_owned(),
            version: "1.0.1".to_owned(),
            meta: json!({})
        };
        assert_ne!(s1, s2);
    }
}
