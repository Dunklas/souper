use std::collections::BTreeSet;
use std::io;
use crate::soup::model::{Soup, SoupSourceParseError};

pub trait SoupSource<R: io::BufRead> {
    fn soups(reader: R) -> Result<BTreeSet<Soup>, SoupSourceParseError>;
}

pub mod package_json;
pub mod csproj;
pub mod docker_base;
