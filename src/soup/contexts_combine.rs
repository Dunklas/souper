use std::collections::{BTreeSet, HashMap};
use serde_json::{Map, Value};
use crate::soup::model::{Soup, SoupContexts};

impl SoupContexts {
    pub fn combine2(&mut self, other: &SoupContexts) {
        for (path, soups) in other.contexts() {
            if !self.contexts.contains_key(path) {
                self.contexts.insert(path.clone(), soups.clone());
            }
        }
        self.contexts.retain(|path,_| other.contexts().contains_key(path));
        
        for (path, other_soups) in other.contexts() {
            let self_soups = self.contexts.get_mut(path).unwrap();
            self_soups.retain(|soup| other_soups.contains(soup));
        }
    }

    pub fn combine(base: SoupContexts, other: SoupContexts) -> SoupContexts {
        let mut result_contexts = base.contexts.clone();
        other.contexts.iter()
            .filter(|(path, _soups)| !base.contexts.contains_key(*path))
            .for_each(|(path, soups)| {
                result_contexts.insert(path.clone(), soups.clone());
            });
        base.contexts.iter().for_each(|(path, soups)| {
            if !other.contexts.contains_key(path) {
                result_contexts.remove(path);
                return;
            }
            let meta_by_name = soups.iter()
                .map(|soup| (&soup.name, &soup.meta))
                .collect::<HashMap<&String, &Map<String, Value>>>();
            let other_soups = other.contexts.get(path).unwrap();
            let mut desired_soups = other_soups.iter()
                .map(|soup| Soup{
                    name: soup.name.clone(),
                    version: soup.version.clone(),
                    meta: match meta_by_name.get(&soup.name) {
                        Some(meta) => combine_meta(*meta, &soup.meta),
                        None => soup.meta.clone(),
                    }
                })
                .collect::<BTreeSet<Soup>>();
            let target_soups = result_contexts.get_mut(path).unwrap();
            target_soups.clear();
            target_soups.append(&mut desired_soups);
        });
        SoupContexts { contexts: result_contexts }
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

        base.combine2(&other);
        assert_eq!(1, base.contexts.len());
        assert_eq!(true, base.contexts.contains_key("src/package.json"));
    }

    #[test]
    fn combine_remove_context() {
        let mut base = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) }
        ]);
        let other = empty_contexts();

        base.combine2(&other);
        assert_eq!(true, base.contexts.is_empty());
    }

    #[test]
    fn combine_added_soup() {
        let base = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) }
        ]);
        let other = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) },
            Soup { name: "some-other-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![]) }
        ]);
        
        let result = SoupContexts::combine(base, other);
        assert_eq!(1, result.contexts.len());
        let soups = result.contexts.get("src/package.json").unwrap();
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
        
        base.combine2(&other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        assert_eq!(1, soups.len());
    }

    #[test]
    fn update_soup_version_preserves_meta() {
        let base = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![("some-meta", "some-value")]) }
        ]);
        let other = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.2.0".to_owned(), meta: meta(vec![]) }
        ]);

        let result = SoupContexts::combine(base, other);
        assert_eq!(1, result.contexts.len());
        let soups = result.contexts.get("src/package.json").unwrap();
        let soup = soups.iter().find(|s| s.name == "some-dep").unwrap();
        assert_eq!("some-dep", soup.name);
        assert_eq!("1.2.0", soup.version);
        assert_eq!(meta(vec![("some-meta", "some-value")]), soup.meta);
    }

    #[test]
    fn append_meta_from_other() {
        let base = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![("some-meta", "some-value")]) }
        ]);
        let other = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![("requirements", "")]) }
        ]);
        let result = SoupContexts::combine(base, other);
        assert_eq!(1, result.contexts.len());
        let soups = result.contexts.get("src/package.json").unwrap();
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
        let base = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![("requirements", "a-requirement")]) }
        ]);
        let other = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: meta(vec![("requirements", "")]) }
        ]);

        let result = SoupContexts::combine(base, other);
        assert_eq!(1, result.contexts.len());
        let soups = result.contexts.get("src/package.json").unwrap();
        let soup = soups.iter().find(|s| s.name == "some-dep").unwrap();
        assert_eq!("some-dep", soup.name);
        assert_eq!("1.0.0", soup.version);
        assert_eq!(meta(vec![("requirements", "a-requirement")]), soup.meta);
    }
}
