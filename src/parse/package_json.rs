use std::{
    collections,
    io,
};
use serde::Deserialize;
use crate::soup::model::Soup;
use super::SoupSource;

pub struct PackageJson {}

#[derive(Deserialize)]
struct Content {
    dependencies: collections::HashMap<String, String>
}

impl <R> SoupSource<R> for PackageJson where R: io::Read {
    fn soups(reader: R) -> Vec<Soup> {
        let content: Content = serde_json::from_reader(reader).unwrap();
        let soups = content.dependencies.into_iter()
            .map(|(key, value)| Soup {
                name: key,
                version: value,
                meta: collections::HashMap::new()
            })
            .collect::<Vec<Soup>>();
        return soups;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_dependency() {
        let content = "{
            \"dependencies\": {
                \"some-lib\": \"^1.0.0\"
            }
        }".as_bytes();
        let soups = PackageJson::soups(content);
        assert_eq!(1, soups.len());
        let expected_soup = Soup {
            name: "some-lib".to_owned(),
            version: "^1.0.0".to_owned(),
            meta: collections::HashMap::new()
        };
        assert_eq!(expected_soup, soups[0]);
    }

    #[test]
    fn multiple_dependencies() {
        let content = "{
            \"dependencies\": {
                \"some-lib\": \"^1.0.0\",
                \"another-lib\": \"6.6.6\"
            }
        }".as_bytes();
        let soups = PackageJson::soups(content);
        assert_eq!(2, soups.len());
        let expected_soups = vec![
            Soup { name: "some-lib".to_owned(), version: "^1.0.0".to_owned(), meta: collections::HashMap::new() },
            Soup { name: "another-lib".to_owned(), version: "6.6.6".to_owned(), meta: collections::HashMap::new() }
        ];
        assert_eq!(true, soups.contains(&expected_soups[0]));
        assert_eq!(true, soups.contains(&expected_soups[1]));
    }
}
