use std::path::PathBuf;
use std::sync::Arc;
use std::hash::Hash;

use super::size::SizeOf;
use super::uuid;
use super::error::SwapResult;
use super::entity::SwapEntity;
use super::handle::SwapHandle;

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

impl<T> SwapPool<T>
where
    T: TryFrom<Vec<u8>> + TryInto<Vec<u8>> + Clone + SizeOf,
    <T as TryFrom<Vec<u8>>>::Error: std::error::Error + 'static,
    <T as TryInto<Vec<u8>>>::Error: std::error::Error + 'static
{
    #[inline]
    /// Spawn new entity in the swap pool with a given file name
    pub fn spawn_named(&mut self, name: impl AsRef<str>, value: T) -> SwapResult<Arc<SwapEntity<T>>> {
        let path = self.path.join(name.as_ref());

        let entity = SwapEntity::create(value, self.handle.clone(), path)?;

        Ok(self.handle.push_entity(entity))
    }
}

impl<T> SwapPool<T>
where
    T: TryFrom<Vec<u8>> + TryInto<Vec<u8>> + Clone + SizeOf + Hash,
    <T as TryFrom<Vec<u8>>>::Error: std::error::Error + 'static,
    <T as TryInto<Vec<u8>>>::Error: std::error::Error + 'static
{
    #[inline]
    /// Spawn new entity in the swap pool
    pub fn spawn(&mut self, value: T) -> SwapResult<Arc<SwapEntity<T>>> {
        self.spawn_named(format!("{:x}.swap", uuid::get(&value)), value)
    }
}
