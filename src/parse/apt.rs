use super::SoupParse;
use crate::soup::model::{Soup, SoupSourceParseError};
use serde_json::{Map, Value};
use std::collections::BTreeSet;


pub struct Apt {}

impl SoupParse for Apt {
    fn soups(
        &self,
        content: &str,
        default_meta: &Map<String, Value>,
    ) -> Result<BTreeSet<Soup>, SoupSourceParseError> {
        todo!()
    }
}