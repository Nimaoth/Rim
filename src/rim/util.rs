use std::path::*;


pub fn get_absolute_path(path: &Path) -> PathBuf {
    let path = path.canonicalize().unwrap();
    // if path.starts_with("\\\\?\\") {
    PathBuf::from(path.to_str().unwrap().replace("\\\\?\\", ""))
}