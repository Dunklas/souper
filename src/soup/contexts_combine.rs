use std::collections::{BTreeSet, HashMap};
use serde_json::{json, Value};
use crate::soup::model::{Soup, SoupContexts};

impl SoupContexts {
    pub fn combine(base: SoupContexts, other: SoupContexts) -> SoupContexts {
        if base.contexts == other.contexts {
            return SoupContexts {
                contexts: base.contexts,
            };
        }
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
                .map(|soup| (soup.name.clone(), soup.meta.clone()))
                .collect::<HashMap<String, Value>>();
            let other_soups = other.contexts.get(path).unwrap();
            let mut desired_soups = other_soups.iter()
                .map(|soup| Soup{
                    name: soup.name.clone(),
                    version: soup.version.clone(),
                    meta: match meta_by_name.get(&soup.name) {
                        Some(meta) => meta.clone(),
                        None => json!({}),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use serde_json::json;

    fn create_contexts(path: &str, soups: Vec<Soup>) -> SoupContexts {
        SoupContexts{
            contexts: [(
                path.to_owned(),
                soups.into_iter().collect()
            )].into_iter().collect()
        }
    }

    fn empty_contexts() -> SoupContexts {
        SoupContexts{
            contexts: BTreeMap::new()
        }
    }

    #[test]
    fn combine_add_context() {
        let first = empty_contexts();
        let second = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: json!({}) }
        ]);

        let result = SoupContexts::combine(first, second);
        assert_eq!(1, result.contexts.len());
        assert_eq!(true, result.contexts.contains_key("src/package.json"));
    }

    #[test]
    fn combine_remove_context() {
        let first = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: json!({}) }
        ]);
        let second = empty_contexts();

        let result = SoupContexts::combine(first, second);
        assert_eq!(true, result.contexts.is_empty());
    }

    #[test]
    fn combine_added_soup() {
        let first = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: json!({}) }
        ]);
        let second = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: json!({}) },
            Soup { name: "some-other-dep".to_owned(), version: "1.0.0".to_owned(), meta: json!({}) }
        ]);
        
        let result = SoupContexts::combine(first, second);
        assert_eq!(1, result.contexts.len());
        let soups = result.contexts.get("src/package.json").unwrap();
        assert_eq!(2, soups.len());
    }

    #[test]
    fn combine_removed_soup() {
        let first = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: json!({}) },
            Soup { name: "some-other-dep".to_owned(), version: "1.0.0".to_owned(), meta: json!({}) }
        ]);
        let second = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: json!({}) }
        ]);
        
        let result = SoupContexts::combine(first, second);
        assert_eq!(1, result.contexts.len());
        let soups = result.contexts.get("src/package.json").unwrap();
        assert_eq!(1, soups.len());
    }

    #[test]
    fn update_soup_version_preserves_meta() {
        let first = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.0.0".to_owned(), meta: json!("{\"some-meta\": \"some-value\"}")}
        ]);
        let second = create_contexts("src/package.json", vec![
            Soup { name: "some-dep".to_owned(), version: "1.2.0".to_owned(), meta: json!({}) }
        ]);

        let result = SoupContexts::combine(first, second);
        assert_eq!(1, result.contexts.len());
        let soups = result.contexts.get("src/package.json").unwrap();
        let soup = soups.iter().find(|s| s.name == "some-dep").unwrap();
        assert_eq!("some-dep", soup.name);
        assert_eq!("1.2.0", soup.version);
        assert_eq!(json!("{\"some-meta\": \"some-value\"}"), soup.meta);
    }
}
