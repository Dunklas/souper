use std::{
    collections::{BTreeSet},
    io
};
use crate::soup::model::Soup;
use super::SoupSource;

pub struct CsProj {}

impl <R> SoupSource<R> for CsProj where R: io::Read {
    fn soups(reader: R) -> BTreeSet<Soup> {
        todo!()
    }
}