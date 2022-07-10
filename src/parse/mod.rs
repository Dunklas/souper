use std::collections::BTreeSet;
use serde_json::{
    Map,
    Value
};
use crate::soup::model::{Soup, SoupSourceParseError};

pub trait SoupParse {
    fn soups(&self, content: &str, default_meta: &Map<String, Value>) -> Result<BTreeSet<Soup>, SoupSourceParseError>;
}

pub mod package_json;
pub mod csproj;
pub mod docker_base;
