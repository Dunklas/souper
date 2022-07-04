use std::collections::BTreeSet;
use std::io;
use crate::soup::model::Soup;

pub trait SoupSource<R: io::Read> {
    fn soups(reader: R) -> BTreeSet<Soup>;
}

pub mod package_json;
pub mod csproj;
