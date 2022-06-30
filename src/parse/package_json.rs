use std::{
    collections,
    io,
};
use serde::{Deserialize};
use crate::soup;
use super::SoupSource;

pub struct PackageJson {}

#[derive(Deserialize)]
struct Content {
    dependencies: collections::HashMap<String, String>
}

impl <R> SoupSource<R> for PackageJson where R: io::Read {
    fn soups(reader: R) -> Vec<soup::Soup> {
        let content: Content = serde_json::from_reader(reader).unwrap();
        let soups = content.dependencies.into_iter()
            .map(|(key, value)| soup::Soup {
                name: key,
                version: value,
                meta: collections::HashMap::new()
            })
            .collect::<Vec<soup::Soup>>();
        return soups;
    }
}
