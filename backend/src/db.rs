use anyhow::Result;
use once_cell::sync::Lazy;
use serde::{Serialize, de::DeserializeOwned};

static DB: Lazy<sled::Db> = Lazy::new(|| {
    // I want this to crash
    sled::open("data").expect("Could not open database")
});

pub struct Tree {
    inner: sled::Tree,
}

impl Tree {
    pub fn insert<S>(&self, key: &str, value: S) -> Result<()>
    where
        S: Serialize,
    {
        let key = key.as_bytes();
        let vec = flexbuffers::to_vec(value)?;

        self.inner.insert(key, vec)?;

        Ok(())
    }

    pub fn all<S>(&self) -> impl Iterator<Item = S> where S: DeserializeOwned + std::fmt::Debug {
        self.inner
            .iter()
            .values()
            .filter_map(|v| v.ok())
            .map(|value| flexbuffers::from_slice(&value))
            .filter_map(|v| v.ok())
    }
}

pub fn tree(name: &str) -> Result<Tree> {
    let key = name.as_bytes();
    let inner = DB.open_tree(key)?;

    Ok(Tree { inner })
}
