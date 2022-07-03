use std::io;
use crate::soup::model::Soup;

pub trait SoupSource<R: io::Read> {
    fn soups(reader: R) -> Vec<Soup>;
}

pub mod package_json;
