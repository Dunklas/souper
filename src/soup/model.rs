use std::{
    collections,
    path
};
use serde::{
    Deserialize,
    Serialize
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Soup {
    pub name: String,
    pub version: String,
    pub meta: collections::HashMap<String, serde_json::Value>
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
    pub contexts: collections::BTreeMap<path::PathBuf, Vec<Soup>>
}

impl SoupContexts {
    pub fn empty() -> SoupContexts {
        SoupContexts { contexts: collections::BTreeMap::new() }
    }

    
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use collections::HashMap;

    use super::*;

    #[test]
    fn soup_equal() {
        let s1 = Soup{
            name: "some-dependency".to_owned(),
            version: "1.0.0".to_owned(),
            meta: HashMap::new()
        };
        let mut meta = HashMap::new();
        meta.insert("requirement".to_owned(), json!("should do this and that"));
        let s2 = Soup{
            name: "some-dependency".to_owned(),
            version: "1.0.0".to_owned(),
            meta
        };
        assert_eq!(s1, s2);
    }

    #[test]
    fn soup_not_equal() {
        let s1 = Soup{
            name: "some-dependency".to_owned(),
            version: "1.0.0".to_owned(),
            meta: HashMap::new()
        };
        let s2 = Soup{
            name: "some-dependency".to_owned(),
            version: "1.0.1".to_owned(),
            meta: HashMap::new()
        };
        assert_ne!(s1, s2);
    }
}
