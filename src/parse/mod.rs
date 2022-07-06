use std::collections::BTreeSet;
use std::io;
use crate::soup::model::{Soup, SoupSourceParseError};

pub mod package_json;
pub mod csproj;
pub mod docker_base;

pub trait SoupParse<R: io::BufRead> {
    fn soups(&self, reader: R) -> Result<BTreeSet<Soup>, SoupSourceParseError>;
}

pub struct SoupSource<R: io::BufRead> {
    source: R,
    parsers: Vec<Box<dyn SoupParse<R>>>
}

impl <R: io::BufRead> SoupSource<R> {
    pub fn new(source: R) -> SoupSource<R> {
        SoupSource{
            source,
            parsers: Vec::new()
        }
    }

    pub fn soups(&self) -> Result<BTreeSet<Soup>, SoupSourceParseError> {
        let mut result = BTreeSet::<Soup>::new();
        // If I can't read a BufRead more than once - Load it as string into memory and then run parsers?
        for parser in self.parsers {
            let partial_soups = match *parser.soups(self.source) {
                Ok(s) => s,
                Err(e) => return Err(e)
            };
            result.append(partial_soups);
        }
        Ok(result)
    }
}
