use crate::soup;

pub trait DependencyParse {
    fn parse(&self) -> soup::SoupContext;
}

pub mod package_json;
