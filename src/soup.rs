use std::{
    collections,
    path
};
use serde::{
    Deserialize,
    Serialize
};

#[derive(Serialize, Deserialize)]
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
