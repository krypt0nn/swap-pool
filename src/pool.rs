use std::path::PathBuf;
use std::sync::Arc;
use std::hash::Hash;

use super::size::SizeOf;
use super::uuid;
use super::error::SwapResult;
use super::entity::SwapEntity;
use super::handle::SwapHandle;
use super::manager::{SwapManager, SwapLastUseManager};
use super::transformer::{SwapTransformer, SwapIdentityTransformer};

pub struct SwapPoolBuilder {
    thread_safe: bool,
    manager: Box<dyn SwapManager>,
    transformer: Box<dyn SwapTransformer>
}

impl Default for SwapPoolBuilder {
    #[inline]
    fn default() -> Self {
        Self {
            thread_safe: true,
            manager: Box::<SwapLastUseManager>::default(),
            transformer: Box::new(SwapIdentityTransformer)
        }
    }
}

impl SwapPoolBuilder {
    #[inline]
    /// Change swap pool thread safety
    /// 
    /// See `InplaceCell` docs for details
    pub fn with_thread_safe(self, thread_safe: bool) -> Self {
        Self {
            thread_safe,
            manager: self.manager,
            transformer: self.transformer
        }
    }

    #[inline]
    /// Change default swap pool entities manager
    pub fn with_manager(self, manager: impl SwapManager + 'static) -> Self {
        Self {
            thread_safe: self.thread_safe,
            manager: Box::new(manager),
            transformer: self.transformer
        }
    }

    #[inline]
    /// Change default swap pool entities' values transformer
    pub fn with_transformer(self, transformer: impl SwapTransformer + 'static) -> Self {
        Self {
            thread_safe: self.thread_safe,
            manager: self.manager,
            transformer: Box::new(transformer)
        }
    }

    #[inline]
    /// Build swap pool
    pub fn build<T>(self, allocated: usize, folder: impl Into<PathBuf>) -> SwapPool<T> {
        SwapPool {
            handle: Arc::new(SwapHandle::new(allocated, self.manager, self.transformer, self.thread_safe)),
            folder: folder.into(),
            thread_safe: self.thread_safe
        }
    }
}

pub struct SwapPool<T> {
    handle: Arc<SwapHandle<T>>,
    folder: PathBuf,
    thread_safe: bool
}

impl<T> SwapPool<T> {
    #[inline]
    /// Create new swap pool with default params
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
    pub fn new(allocated: usize, folder: impl Into<PathBuf>) -> Self {
        SwapPoolBuilder::default().build(allocated, folder)
    }

    #[inline]
    /// Get swap pool builder
    /// 
    /// ```rust,no_run
    /// use swap_pool::prelude::*;
    /// 
    /// // Create the pool with a custom entities manager
    /// // Unfortunately you have to specify SwapPool's type
    /// // even if you just want to get its builder, so you
    /// // can use SwapPoolBuilder::default() instead
    /// let mut pool = SwapPool::<()>::builder()
    ///     .with_manager(SwapUpgradeCountManager::default())
    ///     .with_transformer(SwapIdentityTransformer)
    ///     .build(128, "/tmp");
    /// 
    /// // Spawn new entity
    /// pool.spawn(vec![0; 128]).unwrap();
    /// ```
    pub fn builder() -> SwapPoolBuilder {
        SwapPoolBuilder::default()
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
        let path = self.folder.join(name.as_ref());

        let entity = SwapEntity::create(value, self.handle.clone(), path, self.thread_safe)?;

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
