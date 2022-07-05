use std::{
    collections::{BTreeSet},
    io
};
use super::SoupSource;

pub struct DockerBase {}

impl<R> SoupSource<R> for DockerBase
where
    R: io::BufRead,
{
    fn soups(reader: R) -> Result<std::collections::BTreeSet<crate::soup::model::Soup>, crate::soup::model::SoupSourceParseError> {
        Ok(BTreeSet::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://docs.docker.com/engine/reference/builder/#from
    fn simple_base() {

    }
}