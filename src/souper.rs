use std::path::PathBuf;
use serde_json::{Map, Value};

use crate::scan::dir_scan;
use crate::soup::model::{
    SoupContexts,
    SouperIoError
};

pub fn run(output_file: PathBuf, root_dir: PathBuf, exclude_dirs: Vec<PathBuf>, default_meta: Map<String, Value>) -> Result<(), SouperIoError> {
    let mut current_contexts = match output_file.is_file() {
        true => SoupContexts::read_from_file(&output_file)?,
        false => SoupContexts::empty(),
    };
    let scanned_contexts = dir_scan::scan(&root_dir, &exclude_dirs, default_meta)?;
    current_contexts.apply(scanned_contexts);
    current_contexts.write_to_file(&output_file)?;
    Ok(())
}
