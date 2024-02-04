use std::sync::{Arc, Weak};

use super::size::SizeOf;
use super::inplace_cell::InplaceCell;
use super::error::SwapResult;
use super::entity::SwapEntity;
use super::manager::SwapManager;
use super::transformer::SwapTransformer;

pub struct SwapHandle<T> {
    allocated: usize,
    entities: InplaceCell<Vec<Weak<SwapEntity<T>>>>,
    manager: Box<dyn SwapManager>,
    transformer: Box<dyn SwapTransformer>
}

impl<T> SwapHandle<T> {
    #[inline]
    /// Create new swap pool handle
    pub fn new(allocated: usize, manager: Box<dyn SwapManager>, transformer: Box<dyn SwapTransformer>, thread_safe: bool) -> Self {
        Self {
            allocated,
            entities: InplaceCell::new(Vec::new(), thread_safe),
            manager,
            transformer
        }
    }

    #[inline]
    /// Register an entity in the swap pool
    pub fn push_entity(&self, entity: SwapEntity<T>) -> Arc<SwapEntity<T>> {
        let entity = Arc::new(entity);

        self.entities.update(|entities| entities.push(Arc::downgrade(&entity)));
        self.manager.upgrade(entity.uuid());

        entity
    }

    #[inline]
    /// Upgrade pool entity's rank and return new value
    pub fn upgrade_entity(&self, uuid: u64) -> u64 {
        self.manager.upgrade(uuid)
    }

    #[inline]
    /// Get pool entity's rank
    pub fn rank_entity(&self, uuid: u64) -> u64 {
        self.manager.rank(uuid)
    }

    #[inline]
    /// Get list of entities registered in the pool
    pub fn entities(&self) -> Vec<Weak<SwapEntity<T>>> {
        self.entities.get_copy()
    }

    #[inline]
    /// Get swap pool manager
    pub fn manager(&self) -> &dyn SwapManager {
        self.manager.as_ref()
    }

    #[inline]
    /// Get swap pool transformer
    pub fn transformer(&self) -> &dyn SwapTransformer {
        self.transformer.as_ref()
    }

    #[inline]
    /// Get maximum amount of memory which can be allocated by the pool items
    pub fn allocated(&self) -> usize {
        self.allocated
    }

    #[inline]
    /// Remove references to the unused entities
    pub fn collect_garbage(&self) {
        self.entities.update(|entities| entities.retain(|entity| entity.strong_count() > 0));
    }
}

impl<T> SwapHandle<T> where T: Clone + SizeOf {
    #[inline]
    /// Calculate total amount of memory which is allocated now by the entities
    /// 
    /// This method iterates over all the stored entities
    pub fn used(&self) -> usize {
        self.entities.get_ref()
            .iter()
            .flat_map(|weak| weak.upgrade())
            .filter(|entity| entity.is_hot())
            .map(|entity| entity.size_of())
            .sum()
    }

    #[inline]
    /// Calculate memory which is not used to store entities in the RAM
    /// and available for new allocations
    /// 
    /// This method iterates over all the stored entities
    pub fn available(&self) -> usize {
        self.allocated().checked_sub(self.used()).unwrap_or_default()
    }
}

impl<T> SwapHandle<T>
where
    T: TryFrom<Vec<u8>> + TryInto<Vec<u8>> + Clone + SizeOf,
    <T as TryFrom<Vec<u8>>>::Error: std::error::Error + 'static,
    <T as TryInto<Vec<u8>>>::Error: std::error::Error + 'static
{
    #[inline]
    /// Flush all the stored entities to the disk
    pub fn flush(&self) -> SwapResult<()> {
        for weak in self.entities.get_ref().iter() {
            if let Some(entity) = weak.upgrade() {
                entity.flush()?;
            }
        }

        Ok(())
    }

    /// Free given amount of memory by flushing hot entities
    /// 
    /// If the function returned `Ok(false)` - then the method
    /// failed to free required amount of memory but there's also
    /// no hot entities remained so nothing to unallocate
    pub fn free(&self, mut memory: usize) -> SwapResult<bool> {
        // Prepare list of entities and their ranks
        let mut entities = self.entities.get_ref()
            .iter()
            .flat_map(|entity| entity.upgrade())
            .map(|entity| (self.manager.rank(entity.uuid()), entity))
            .collect::<Vec<_>>();

        // Sort entities by their ranks in descending order
        entities.sort_by(|a, b| b.0.cmp(&a.0));

        // Flush entities one by one until we free enough memory
        while memory > 0 {
            let Some((_, entity)) = entities.pop() else {
                return Ok(false);
            };

            // Flush entity if it's hot
            if entity.is_hot() {
                // Read its size before flushing because it will change after flushing
                let mut size = entity.size_of();

                // Flush the entity
                entity.flush()?;

                // Decrement flushed entity size to find freed memory size
                size -= entity.size_of();

                // We can free more memory than needed so use checked sub here
                memory = memory.checked_sub(size).unwrap_or_default();
            }
        }

        Ok(true)
    }
}
