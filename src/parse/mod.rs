use std::io;
use crate::soup;

pub trait SoupSource<R: io::Read> {
    fn soups(reader: R) -> Vec<soup::Soup>;
}

pub mod package_json;
