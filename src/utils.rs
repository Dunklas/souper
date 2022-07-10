use std::path;

pub fn relative_path<P: AsRef<path::Path>>(
    full_path: P,
    source_dir: P,
) -> Result<path::PathBuf, path::StripPrefixError> {
    match full_path.as_ref().strip_prefix(source_dir) {
        Ok(res) => Ok(res.to_path_buf()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use path::{Path, PathBuf};

    #[test]
    fn some_test() {
        let p1 = Path::new("/home/user/repo/src/package.json");
        let p2 = Path::new("/home/user/repo");
        let result = relative_path(p1, p2);
        assert_eq!(true, result.is_ok());

        let expected_path: PathBuf = ["src", "package.json"].iter().collect();

        assert_eq!(expected_path, result.unwrap());
    }
}
