use std::{
    cmp,
    collections,
    path
};
use serde::{
    Deserialize,
    Serialize
};

#[derive(Serialize, Deserialize, Debug, cmp::Eq, cmp::PartialEq)]
pub struct Soup {
    pub name: String,
    pub version: String,
    pub meta: collections::HashMap<String, serde_json::Value>
}

#[derive(Serialize, Deserialize)]
pub struct SoupContext {
    pub path: path::PathBuf,
    pub soups: Vec<Soup>
}
