use crate::soup::model::{Soup, SoupContexts};
use std::path::PathBuf;

impl SoupContexts {
    pub fn combine(first: SoupContexts, second: SoupContexts) -> SoupContexts {
        if first.contexts == second.contexts {
            return SoupContexts {
                contexts: first.contexts,
            };
        }
        let mut contexts = first.contexts.clone();
        let keys_to_add: Vec<&PathBuf> = second
            .contexts
            .iter()
            .map(|(p, _soups)| p)
            .filter(|key| !first.contexts.contains_key(*key))
            .collect();
        let keys_to_remove: Vec<&PathBuf> = first
            .contexts
            .iter()
            .map(|(p, _soups)| p)
            .filter(|key| !second.contexts.contains_key(*key))
            .collect();
        for key in keys_to_remove {
            contexts.remove(key);
        }
        for key in keys_to_add {
            let soups = second.contexts.get(key).unwrap();
            contexts.insert(key.to_owned(), soups.clone());
        }
        // Update soup versions
        // Add and remove individual soups
        SoupContexts { contexts }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};
    use super::*;

    #[test]
    fn combine_add_context() {
        let first = SoupContexts {
            contexts: BTreeMap::new(),
        };
        let mut second_contexts: BTreeMap<PathBuf, Vec<Soup>> = BTreeMap::new();
        second_contexts.insert(
            ["src", "package.json"].iter().collect(),
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: HashMap::new(),
            }],
        );
        let second = SoupContexts {
            contexts: second_contexts,
        };
        let result = SoupContexts::combine(first, second);
        let expected_key: PathBuf = ["src", "package.json"].iter().collect();
        assert_eq!(true, result.contexts.contains_key(&expected_key));
    }

    #[test]
    fn combine_remove_context() {
        let mut first_contexts: BTreeMap<PathBuf, Vec<Soup>> = BTreeMap::new();
        first_contexts.insert(
            ["src", "package.json"].iter().collect(),
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: HashMap::new(),
            }],
        );
        let first = SoupContexts {
            contexts: first_contexts,
        };
        let second = SoupContexts {
            contexts: BTreeMap::new(),
        };
        let result = SoupContexts::combine(first, second);
        assert_eq!(true, result.contexts.is_empty());
    }
}
