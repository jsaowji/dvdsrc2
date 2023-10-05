mod mpeg2;
use std::path::{Path, PathBuf};

pub use mpeg2::*;

mod vobu;
pub use vobu::*;

pub struct IndexManager {
    md: PathBuf,
}

impl IndexManager {
    pub fn new() -> IndexManager {
        let cachedir = home::home_dir().unwrap().join(".cache").join("dvdsrc2");
        std::fs::create_dir_all(&cachedir).unwrap();
        IndexManager { md: cachedir }
    }

    pub fn get_dir<P: AsRef<Path>>(&self, a: P) -> PathBuf {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        a.as_ref().hash(&mut hasher);
        "version0".hash(&mut hasher);

        return self.md.join(format!("{:x}", hasher.finish()));
    }
}
