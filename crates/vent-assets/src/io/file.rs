use std::{
    env,
    path::{Path, PathBuf},
};

pub(crate) fn get_base_path() -> PathBuf {
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        PathBuf::from(manifest_dir)
    } else {
        env::current_exe()
            .map(|path| path.parent().map(ToOwned::to_owned).unwrap())
            .unwrap()
    }
}

/// I/O implementation for the local filesystem.
///
/// This asset I/O is fully featured but it's not available on `android` and `wasm` targets.
#[allow(dead_code)]
pub struct FileAsset {
    root_path: PathBuf,
}
#[allow(dead_code)]
impl FileAsset {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let root_path = get_base_path().join(path.as_ref());
        Self { root_path }
    }

    pub fn root_path(&self) -> &PathBuf {
        &self.root_path
    }
}
