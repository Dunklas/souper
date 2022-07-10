use super::SoupParse;
use crate::soup::model::{Soup, SoupSourceParseError};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub struct Cargo {}

impl SoupParse for Cargo {
    fn soups(
        &self,
        content: &str,
        default_meta: &Map<String, Value>,
    ) -> Result<BTreeSet<Soup>, SoupSourceParseError> {
        Ok(BTreeSet::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_dependency() {
        let content = r#"
[dependencies]
some-dep = "6.6.6"
        "#;
        let result = Cargo {}.soups(content, &Map::new());
        assert_eq!(true, result.is_ok());
        let soups = result.unwrap();
        assert_eq!(
            true,
            soups.contains(&Soup {
                name: "some-dep".to_owned(),
                version: "6.6.6".to_owned(),
                meta: Map::new()
            })
        );
    }
}
