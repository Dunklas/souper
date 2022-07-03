use crate::soup::model::{Soup, SoupContexts};

impl SoupContexts {
    pub fn combine(first: SoupContexts, second: SoupContexts) -> SoupContexts {
        if first.contexts == second.contexts {
            return SoupContexts {
                contexts: first.contexts,
            };
        }
        let mut contexts = first.contexts.clone();
        for context_path in second.contexts.keys() {
            if !first.contexts.contains_key(context_path) {
                let soups = second.contexts.get(context_path).unwrap();
                contexts.insert(context_path.clone(), soups.clone());
                continue;
            }
        }

        for context_path in first.contexts.keys() {
            if !second.contexts.contains_key(context_path) {
                contexts.remove(context_path);
                continue;
            }
            let soups = first.contexts.get(context_path).unwrap();
            let other_soups = second.contexts.get(context_path).unwrap();
            let target_soups = contexts.get_mut(context_path).unwrap();
            if soups.len() < other_soups.len() {
                let missing: Vec<&Soup> = other_soups
                    .iter()
                    .filter(|soup| soups.iter().find(|x| x == soup) == None)
                    .collect();
                for s in missing {
                    target_soups.push(s.clone());
                }
            }
            if soups.len() > other_soups.len() {
                target_soups.retain(|x| other_soups.iter().find(|y| x == *y) != None);
            }
        }
        // Update soup versions
        SoupContexts { contexts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use serde_json::json;

    #[test]
    fn combine_add_context() {
        let first = SoupContexts {
            contexts: BTreeMap::new(),
        };
        let mut second_contexts: BTreeMap<String, Vec<Soup>> = BTreeMap::new();
        second_contexts.insert(
            "src/package.json".to_owned(),
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: json!("{}")
            }],
        );
        let second = SoupContexts {
            contexts: second_contexts,
        };
        let result = SoupContexts::combine(first, second);
        assert_eq!(1, result.contexts.len());
        assert_eq!(true, result.contexts.contains_key("src/package.json"));
    }

    #[test]
    fn combine_remove_context() {
        let mut first_contexts: BTreeMap<String, Vec<Soup>> = BTreeMap::new();
        first_contexts.insert(
            "src/package.json".to_owned(),
            vec![Soup {
                name: "some-dep".to_owned(),
                version: "1.0.0".to_owned(),
                meta: json!("{}")
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

    #[test]
    fn combine_added_soup() {
        let first = SoupContexts {
            contexts: [(
                "src/package.json".to_owned(),
                vec![Soup {
                    name: "some-dep".to_owned(),
                    version: "1.0.0".to_owned(),
                    meta: json!("{}")
                }],
            )]
            .iter()
            .cloned()
            .collect(),
        };
        let second = SoupContexts {
            contexts: [(
                "src/package.json".to_owned(),
                vec![
                    Soup {
                        name: "some-dep".to_owned(),
                        version: "1.0.0".to_owned(),
                        meta: json!("{}")
                    },
                    Soup {
                        name: "some-other-dep".to_owned(),
                        version: "1.0.0".to_owned(),
                        meta: json!("{}")
                    },
                ],
            )]
            .iter()
            .cloned()
            .collect(),
        };

        let result = SoupContexts::combine(first, second);
        assert_eq!(1, result.contexts.len());
        let soups = result
            .contexts
            .get("src/package.json")
            .unwrap();
        assert_eq!(2, soups.len());
    }

    #[test]
    fn combine_removed_soup() {
        let first = SoupContexts {
            contexts: [(
                "src/package.json".to_owned(),
                vec![
                    Soup {
                        name: "some-dep".to_owned(),
                        version: "1.0.0".to_owned(),
                        meta: json!("{}")
                    },
                    Soup {
                        name: "some-other-dep".to_owned(),
                        version: "1.0.0".to_owned(),
                        meta: json!("{}")
                    },
                ],
            )]
            .iter()
            .cloned()
            .collect(),
        };
        let second = SoupContexts {
            contexts: [(
                "src/package.json".to_owned(),
                vec![Soup {
                    name: "some-dep".to_owned(),
                    version: "1.0.0".to_owned(),
                    meta: json!("{}")
                }],
            )]
            .iter()
            .cloned()
            .collect(),
        };

        let result = SoupContexts::combine(first, second);
        assert_eq!(1, result.contexts.len());
        let soups = result
            .contexts
            .get("src/package.json")
            .unwrap();
        assert_eq!(1, soups.len());
    }
}
