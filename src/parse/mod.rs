use crate::soup::model::{Soup, SoupSourceParseError};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub trait SoupParse {
    fn soups(
        &self,
        content: &str,
        default_meta: &Map<String, Value>,
    ) -> Result<BTreeSet<Soup>, SoupSourceParseError>;
}

pub mod apt;
pub mod cargo;
pub mod csproj;
pub mod docker_base;
pub mod package_json;
