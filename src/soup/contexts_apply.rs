use crate::soup::model::{Soup, SoupContexts};
use serde_json::{Map, Value};
use std::collections::btree_map::Entry;
use std::collections::{BTreeSet, HashMap};

impl SoupContexts {
    pub fn apply(&mut self, other: SoupContexts) {
        self.contexts
            .retain(|path, _| other.contexts().contains_key(path));

        let mut other_contexts = other.contexts.into_iter().collect::<Vec<(_, _)>>();
        while let Some((path, other_soups)) = other_contexts.pop() {
            let (path, self_soups) = match self.contexts.entry(path) {
                Entry::Vacant(entry) => {
                    entry.insert(other_soups);
                    continue;
                }
                Entry::Occupied(entry) => entry.remove_entry(),
            };
            self.contexts.insert(path, combine_soups(self_soups, other_soups));
        }
    }
}

fn combine_soups(base: BTreeSet<Soup>, other: BTreeSet<Soup>) -> BTreeSet<Soup> {
    let mut base_meta = base.into_iter()
        .map(|soup| (soup.name, soup.meta))
        .collect::<HashMap<String, Map<String, Value>>>();
    other.into_iter()
        .map(|other_soup| {
            let meta = match base_meta.remove(&other_soup.name) {
                Some(self_meta) => combine_meta(self_meta, other_soup.meta),
                None => other_soup.meta,
            };
            Soup {
                name: other_soup.name,
                version: other_soup.version,
                meta,
            }
        })
        .collect::<BTreeSet<Soup>>()
}

fn combine_meta(mut base: Map<String, Value>, other: Map<String, Value>) -> Map<String, Value> {
    let mut patch = other.into_iter().collect::<Vec<(String, Value)>>();
    while let Some((key, value)) = patch.pop() {
        if let serde_json::map::Entry::Vacant(entry) = base.entry(key) {
            entry.insert(value);
        }
    }
    base
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn create_contexts(path: &str, soups: Vec<Soup>) -> SoupContexts {
        SoupContexts {
            contexts: [(path.to_owned(), soups.into_iter().collect())]
                .into_iter()
                .collect(),
        }
    }

    fn meta(key_values: Vec<(&str, &str)>) -> Map<String, Value> {
        key_values
            .iter()
            .map(|(key, value)| (String::from(*key), serde_json::to_value(value).unwrap()))
            .collect::<Map<String, Value>>()
    }

    fn empty_contexts() -> SoupContexts {
        SoupContexts {
            contexts: BTreeMap::new(),
        }
    }

    #[test]
    fn combine_add_context() {
        let mut base = empty_contexts();
        let other = create_contexts(
            "src/package.json",
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: meta(vec![]),
            }],
        );

        base.apply(other);
        assert_eq!(1, base.contexts.len());
        assert_eq!(true, base.contexts.contains_key("src/package.json"));
    }

    #[test]
    fn combine_remove_context() {
        let mut base = create_contexts(
            "src/package.json",
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: meta(vec![]),
            }],
        );
        let other = empty_contexts();

        base.apply(other);
        assert_eq!(true, base.contexts.is_empty());
    }

    #[test]
    fn combine_added_soup() {
        let mut base = create_contexts(
            "src/package.json",
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: meta(vec![]),
            }],
        );
        let other = create_contexts(
            "src/package.json",
            vec![
                Soup {
                    name: "some-dep".to_owned(),
                    version: "1.0.0".to_owned(),
                    meta: meta(vec![]),
                },
                Soup {
                    name: "some-other-dep".to_owned(),
                    version: "1.0.0".to_owned(),
                    meta: meta(vec![]),
                },
            ],
        );

        base.apply(other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        assert_eq!(2, soups.len());
    }

    #[test]
    fn combine_removed_soup() {
        let mut base = create_contexts(
            "src/package.json",
            vec![
                Soup {
                    name: "some-dep".to_owned(),
                    version: "1.0.0".to_owned(),
                    meta: meta(vec![]),
                },
                Soup {
                    name: "some-other-dep".to_owned(),
                    version: "1.0.0".to_owned(),
                    meta: meta(vec![]),
                },
            ],
        );
        let other = create_contexts(
            "src/package.json",
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: meta(vec![]),
            }],
        );

        base.apply(other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        assert_eq!(1, soups.len());
    }

    #[test]
    fn no_update_preserves_meta() {
        let mut base = create_contexts(
            "src/package.json",
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: meta(vec![("some-meta", "some-value")]),
            }],
        );
        let other = create_contexts(
            "src/package.json",
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: meta(vec![]),
            }],
        );

        base.apply(other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        let soup = soups.iter().find(|s| s.name == "some-dep").unwrap();
        assert_eq!("some-dep", soup.name);
        assert_eq!("1.0.0", soup.version);
        assert_eq!(meta(vec![("some-meta", "some-value")]), soup.meta);
    }

    #[test]
    fn update_soup_version_preserves_meta() {
        let mut base = create_contexts(
            "src/package.json",
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: meta(vec![("some-meta", "some-value")]),
            }],
        );
        let other = create_contexts(
            "src/package.json",
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.2.0".to_owned(),
                meta: meta(vec![]),
            }],
        );

        base.apply(other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        let soup = soups.iter().find(|s| s.name == "some-dep").unwrap();
        assert_eq!("some-dep", soup.name);
        assert_eq!("1.2.0", soup.version);
        assert_eq!(meta(vec![("some-meta", "some-value")]), soup.meta);
    }

    #[test]
    fn append_meta_from_other() {
        let mut base = create_contexts(
            "src/package.json",
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: meta(vec![("some-meta", "some-value")]),
            }],
        );
        let other = create_contexts(
            "src/package.json",
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: meta(vec![("requirements", "")]),
            }],
        );
        base.apply(other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        let soup = soups.iter().find(|s| s.name == "some-dep").unwrap();
        assert_eq!("some-dep", soup.name);
        assert_eq!("1.0.0", soup.version);
        assert_eq!(
            meta(vec![("some-meta", "some-value"), ("requirements", "")]),
            soup.meta
        );
    }

    #[test]
    fn append_meta_from_other_no_overwrite() {
        let mut base = create_contexts(
            "src/package.json",
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: meta(vec![("requirements", "a-requirement")]),
            }],
        );
        let other = create_contexts(
            "src/package.json",
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: meta(vec![("requirements", "")]),
            }],
        );

        base.apply(other);
        assert_eq!(1, base.contexts.len());
        let soups = base.contexts.get("src/package.json").unwrap();
        let soup = soups.iter().find(|s| s.name == "some-dep").unwrap();
        assert_eq!("some-dep", soup.name);
        assert_eq!("1.0.0", soup.version);
        assert_eq!(meta(vec![("requirements", "a-requirement")]), soup.meta);
    }
}
