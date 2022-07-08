use serde_json::{Map, Value};
use crate::soup::model::{Soup, SoupContexts};

impl SoupContexts {
    pub fn combine2(&mut self, other: SoupContexts) {
        self.contexts.retain(|path,_| other.contexts().contains_key(path));

        let mut other_contexts = other.contexts.into_iter().collect::<Vec<(_, _)>>();
        while let Some((path, soups)) = other_contexts.pop() {
            if !self.contexts.contains_key(&path) {
                self.contexts.insert(path, soups);
                continue;
            }
        }
    }

    pub fn combine(&mut self, other: &SoupContexts) {
        for (path, soups) in other.contexts() {
            if !self.contexts.contains_key(path) {
                self.contexts.insert(path.clone(), soups.clone());
            }
        }
        self.contexts.retain(|path,_| other.contexts().contains_key(path));
        
        for (path, other_soups) in other.contexts() {
            let self_soups = self.contexts.get_mut(path).unwrap();

            for other_soup in other_soups {
                let soup = Soup {
                    name: other_soup.name.clone(),
                    version: other_soup.version.clone(),
                    meta: match self_soups.iter().find(|x| x.name == other_soup.name) {
                        Some(soup) => combine_meta(&soup.meta, &other_soup.meta),
                        None => other_soup.meta.clone()
                    }
                };
                self_soups.remove(&soup);
                self_soups.insert(soup);
            }

            self_soups.retain(|soup| other_soups.contains(soup));
        }
    }
}


fn combine_meta(base: &Map<String, Value>, patch: &Map<String, Value>) -> Map<String, Value> {
    let mut result = base.clone();
    for (key, value) in patch {
        if !base.contains_key(key) {
            result.insert(key.clone(), value.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn create_contexts(path: &str, soups: Vec<Soup>) -> SoupContexts {
        SoupContexts{
            contexts: [(
                path.to_owned(),
                soups.into_iter().collect()
            )].into_iter().collect()
        }
    }

    fn meta(key_values: Vec<(&str, &str)>) -> Map<String, Value> {
        key_values.iter()
            .map(|(key, value)| (String::from(*key), serde_json::to_value(value).unwrap()))
            .collect::<Map<String, Value>>()
    }

    fn empty_contexts() -> SoupContexts {
        SoupContexts{
            contexts: BTreeMap::new()
        }
    }

    #[test]
    fn combine_add_context() {
        let mut base = empty_contexts();
        let other = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) }
        ]);

        base.combine2(other);
        assert_eq!(1, base.contexts.len());
        assert_eq!(true, base.contexts.contains_key("src/package.json"));
    }

    #[test]
    fn combine_remove_context() {
        let mut base = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) }
        ]);
        let other = empty_contexts();

        base.combine2(other);
        assert_eq!(true, base.contexts.is_empty());
    }

    #[test]
    fn combine_added_soup() {
        let mut base = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) }
        ]);
        let other = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) },
            Soup { name: "some-other-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) }
        ]);
        
        base.combine(&other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        assert_eq!(2, soups.len());
    }

    #[test]
    fn combine_removed_soup() {
        let mut base = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) },
            Soup { name: "some-other-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) }
        ]);
        let other = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) }
        ]);
        
        base.combine(&other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        assert_eq!(1, soups.len());
    }

    #[test]
    fn no_update_preserves_meta() {
        let mut base = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![("some-meta", "some-value")]) }
        ]);
        let other = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) }
        ]);

        base.combine(&other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        let soup = soups.iter().find(|s| s.name == "some-dep").unwrap();
        assert_eq!("some-dep", soup.name);
        assert_eq!("1.0.0", soup.version);
        assert_eq!(meta(vec![("some-meta", "some-value")]), soup.meta);
    }

    #[test]
    fn update_soup_version_preserves_meta() {
        let mut base = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![("some-meta", "some-value")]) }
        ]);
        let other = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.2.0".to_owned(), meta: meta(vec![]) }
        ]);

        base.combine(&other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        let soup = soups.iter().find(|s| s.name == "some-dep").unwrap();
        assert_eq!("some-dep", soup.name);
        assert_eq!("1.2.0", soup.version);
        assert_eq!(meta(vec![("some-meta", "some-value")]), soup.meta);
    }

    #[test]
    fn append_meta_from_other() {
        let mut base = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![("some-meta", "some-value")]) }
        ]);
        let other = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![("requirements", "")]) }
        ]);
        base.combine(&other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        let soup = soups.iter().find(|s| s.name == "some-dep").unwrap();
        assert_eq!("some-dep", soup.name);
        assert_eq!("1.0.0", soup.version);
        assert_eq!(meta(vec![
            ("some-meta", "some-value"),
            ("requirements", "")
        ]), soup.meta);
    }

    #[test]
    fn append_meta_from_other_no_overwrite() {
        let mut base = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![("requirements", "a-requirement")]) }
        ]);
        let other = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![("requirements", "")]) }
        ]);

        base.combine(&other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        let soup = soups.iter().find(|s| s.name == "some-dep").unwrap();
        assert_eq!("some-dep", soup.name);
        assert_eq!("1.0.0", soup.version);
        assert_eq!(meta(vec![("requirements", "a-requirement")]), soup.meta);
    }
}
