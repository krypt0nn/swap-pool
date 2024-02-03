use std::path::PathBuf;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::io::Result;

use super::size::SizeOf;
use super::handle::SwapHandle;
use super::entity::SwapEntity;

pub struct SwapPool<T> {
    handle: Arc<SwapHandle<T>>,
    path: PathBuf
}

impl<T> SwapPool<T> {
    #[inline]
    /// Create new swap pool
    pub fn new(allocated: usize, path: impl Into<PathBuf>) -> Self {
        Self {
            handle: Arc::new(SwapHandle::new(allocated)),
            path: path.into()
        }
    }

    #[inline]
    /// Get swap pool's handle
    pub fn handle(&self) -> &Arc<SwapHandle<T>> {
        &self.handle
    }
}

impl<T> SwapPool<T> where T: From<Vec<u8>> + Into<Vec<u8>> + Clone + SizeOf {
    #[inline]
    /// Spawn new entity in the swap pool with a given file name
    pub fn spawn_named(&mut self, name: impl AsRef<str>, value: T) -> Result<Arc<SwapEntity<T>>> {
        let path = self.path.join(name.as_ref());

        let entity = SwapEntity::create(value, self.handle.clone(), path)?;

        Ok(self.handle.push_entity(entity))
    }
}

impl<T> SwapPool<T> where T: From<Vec<u8>> + Into<Vec<u8>> + Clone + SizeOf + Hash {
    #[inline]
    /// Spawn new entity in the swap pool
    pub fn spawn(&mut self, value: T) -> Result<Arc<SwapEntity<T>>> {
        let mut hasher = DefaultHasher::new();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();

        value.hash(&mut hasher);
        timestamp.hash(&mut hasher);

        self.spawn_named(format!("{:x}.swap", hasher.finish()), value)
    }
}
