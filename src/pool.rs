use std::path::PathBuf;
use std::sync::Arc;
use std::hash::Hash;

use super::size::SizeOf;
use super::uuid;
use super::error::{SwapResult, SwapError};
use super::entity::SwapEntity;
use super::handle::SwapHandle;
use super::manager::{SwapManager, SwapLastUseManager};

pub struct SwapPool<T> {
    handle: Arc<SwapHandle<T>>,
    path: PathBuf
}

impl<T> SwapPool<T> {
    #[inline]
    /// Create new swap pool with a `SwapLastUseManager` entities manager
    /// 
    /// ```rust,no_run
    /// use swap_pool::prelude::*;
    /// 
    /// // Create the pool
    /// let mut pool = SwapPool::new(128, "/tmp");
    /// 
    /// // Spawn new entity
    /// pool.spawn(vec![0; 128]).unwrap();
    /// ```
    pub fn new(allocated: usize, path: impl Into<PathBuf>) -> Self {
        Self::with_manager(allocated, path, SwapLastUseManager::default())
    }

    #[inline]
    /// Create new swap pool with a custom entities manager
    /// 
    /// ```rust,no_run
    /// use swap_pool::prelude::*;
    /// 
    /// // Create the pool with a custom entities manager
    /// let mut pool = SwapPool::with_manager(128, "/tmp", SwapUpgradeCountManager::default());
    /// 
    /// // Spawn new entity
    /// pool.spawn(vec![0; 128]).unwrap();
    /// ```
    pub fn with_manager(allocated: usize, path: impl Into<PathBuf>, manager: impl SwapManager + 'static) -> Self {
        Self {
            handle: Arc::new(SwapHandle::new(allocated, manager)),
            path: path.into()
        }
    }

    #[inline]
    /// Get swap pool's handle
    /// 
    /// ```rust,no_run
    /// use swap_pool::prelude::*;
    /// 
    /// // Create the pool
    /// let mut pool = SwapPool::new(128, "/tmp");
    /// 
    /// // Spawn new entity which will be immediately dropped
    /// pool.spawn(vec![0; 128]).unwrap();
    /// 
    /// // Remove dropped entities from the pool
    /// pool.handle().collect_garbage();
    /// ```
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
    /// 
    /// ```rust,no_run
    /// use swap_pool::prelude::*;
    /// 
    /// // Create the pool
    /// let mut pool = SwapPool::new(128, "/tmp");
    /// 
    /// // Spawn new entity with the given swap file name
    /// let entity = pool.spawn_named("My cool swap file", vec![0; 128]).unwrap();
    /// 
    /// // Flush the file to the swap folder
    /// entity.flush().unwrap();
    /// 
    /// // Check that the swap file exists
    /// assert!(std::path::PathBuf::from("/tmp/My cool swap file").exists());
    /// 
    /// // Drop the entity to delete swap file
    /// drop(entity);
    /// 
    /// assert!(!std::path::PathBuf::from("/tmp/My cool swap file").exists());
    /// ```
    pub fn spawn_named(&mut self, name: impl AsRef<str>, value: T) -> SwapResult<Arc<SwapEntity<T>>> {
        let path = self.path.join(name.as_ref());

        let entity = SwapEntity::create(value, self.handle.clone(), path)?;

        Ok(self.handle.push_entity(entity))
    }

    #[inline]
    /// Spawn new entity in the swap pool from the given file
    /// 
    /// ```rust,no_run
    /// use swap_pool::prelude::*;
    /// 
    /// // Create the pool
    /// let mut pool = SwapPool::<Vec<u8>>::new(128, "/tmp"); // Path doesn't matter here
    /// 
    /// // Try to allocate the file on the RAM
    /// let entity = pool.spawn_from_file("1gb_text_file.txt").unwrap();
    /// 
    /// // Print file's len
    /// println!("File len: {}", entity.value().unwrap().len());
    /// 
    /// // Entity's swap file will be deleted once the entity is dropped
    /// // so you have to consider this if you want to keep this file on the disk
    /// drop(entity); // You don't need to call this function manually
    /// 
    /// // File "1gb_text_file.txt" doesn't exist anymore
    /// ```
    pub fn spawn_from_file(&mut self, file: impl Into<PathBuf>) -> SwapResult<Arc<SwapEntity<T>>> {
        let path: PathBuf = file.into();

        let value = T::try_from(std::fs::read(&path)?)
            .map_err(|err| SwapError::Deserialize(Box::new(err)))?;

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
    /// 
    /// ```rust,no_run
    /// use swap_pool::prelude::*;
    /// 
    /// // Create the pool
    /// let mut pool = SwapPool::new(128, "/tmp");
    /// 
    /// // Spawn new entity with a random swap file name
    /// let entity = pool.spawn(vec![0; 128]).unwrap();
    /// 
    /// // Print the entity's value length
    /// println!("Value len: {}", entity.value().unwrap().len());
    /// ```
    pub fn spawn(&mut self, value: T) -> SwapResult<Arc<SwapEntity<T>>> {
        self.spawn_named(format!("{:x}.swap", uuid::get(&value)), value)
    }
}
