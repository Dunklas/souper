use std::{
    collections::{HashMap, BTreeSet},
    io,
};
use serde::Deserialize;
use serde_json::json;
use crate::soup::model::Soup;
use super::SoupSource;

pub struct PackageJson {}

#[derive(Deserialize)]
struct Content {
    dependencies: HashMap<String, String>
}

impl <R> SoupSource<R> for PackageJson where R: io::BufRead {
    fn soups(reader: R) -> BTreeSet<Soup> {
        let content: Content = serde_json::from_reader(reader).unwrap();
        let soups = content.dependencies.into_iter()
            .map(|(key, value)| Soup {
                name: key,
                version: value,
                meta: json!({})
            })
            .collect::<BTreeSet<Soup>>();
        return soups;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_dependency() {
        let content = r#"{
            "dependencies": {
                "some-lib": "^1.0.0"
            }
        }"#.as_bytes();
        let soups = PackageJson::soups(content);
        assert_eq!(1, soups.len());
        let expected_soup = Soup {
            name: "some-lib".to_owned(),
            version: "^1.0.0".to_owned(),
            meta: json!({})
        };
        assert_eq!(true, soups.contains(&expected_soup));
    }

    #[test]
    fn multiple_dependencies() {
        let content = r#"{
            "dependencies": {
                "some-lib": "^1.0.0",
                "another-lib": "6.6.6"
            }
        }"#.as_bytes();
        let soups = PackageJson::soups(content);
        assert_eq!(2, soups.len());
        let expected_soups = vec![
            Soup { name: "some-lib".to_owned(), version: "^1.0.0".to_owned(), meta: json!({}) },
            Soup { name: "another-lib".to_owned(), version: "6.6.6".to_owned(), meta: json!({}) }
        ].into_iter().collect::<BTreeSet<Soup>>();
        assert_eq!(expected_soups, soups);
    }

    #[test]
    fn no_dependencies() {
        let content = r#"{
            "dependencies": {}
        }"#.as_bytes();
        let soups = PackageJson::soups(content);
        assert_eq!(0, soups.len());
    }
}
