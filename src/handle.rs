use std::collections::VecDeque;
use std::sync::{Arc, Weak};
use std::io::Result;

use super::size::SizeOf;
use super::inplace_cell::InplaceCell;
use super::entity::SwapEntity;

pub struct SwapHandle<T> {
    allocated: usize,
    entities: InplaceCell<Vec<Weak<SwapEntity<T>>>>,
    fetches: InplaceCell<VecDeque<u64>>
}

impl<T> SwapHandle<T> {
    #[inline]
    /// Create new swap pool handle
    pub fn new(allocated: usize) -> Self {
        Self {
            allocated,
            entities: InplaceCell::new(Vec::new()),
            fetches: InplaceCell::new(VecDeque::new())
        }
    }

    #[inline]
    /// Register an entity in the swap pool
    pub fn push_entity(&self, entity: SwapEntity<T>) -> Arc<SwapEntity<T>> {
        let entity = Arc::new(entity);

        self.entities.update(|entities| entities.push(Arc::downgrade(&entity)));
        self.fetches.update(|fetches| fetches.push_back(entity.uuid()));

        entity
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
        self.entities.get()
            .into_iter()
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

    #[inline]
    /// Mark entity with given UUID as "keep alive".
    /// This means that this entity will be flushed by the pool
    /// only if other entities cannot be
    pub fn keep_alive(&self, uuid: u64) {
        self.fetches.update(|fetches| {
            // Remove given uuid from the fetches history
            fetches.retain(|fetch| fetch != &uuid);

            let shift = fetches.len()
                .checked_sub(self.entities.get().len())
                .unwrap_or_default();

            // Remove old keep alive records
            if shift > 0 {
                fetches.drain(..shift);
            }

            // Push uuid to the fetches history
            fetches.push_back(uuid);
        });
    }
}

impl<T> SwapHandle<T> where T: From<Vec<u8>> + Into<Vec<u8>> + Clone + SizeOf {
    #[inline]
    /// Flush all the stored entities to the disk
    pub fn flush(&self) -> Result<()> {
        for weak in self.entities.get() {
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
    pub fn free(&self, mut memory: usize) -> Result<bool> {
        // Remove unused entities
        self.collect_garbage();

        // Get history of entities fetches
        let fetches = self.fetches.get();

        // Prepare list of entities and their keep alive rank
        let mut entities = self.entities.get()
            .into_iter()
            .flat_map(|entity| entity.upgrade())
            .map(|entity| {
                let position = fetches.iter().position(|fetch| fetch == &entity.uuid());

                (position, entity)
            })
            .collect::<Vec<_>>();

        // Sort entities by their keep alive rank in descending order
        entities.sort_by(|a, b| b.0.cmp(&a.0));

        // Flush entities one by one until we free enough memory
        while memory > 0 {
            let Some((_, entity)) = entities.pop() else {
                return Ok(false);
            };

            // Flush entity if it's hot
            if entity.is_hot() {
                // Read its size before flushing because it will change afterwards
                let size = entity.size_of();

                // Flush the entity
                entity.flush()?;

                // We can free more memory than needed so use checked sub here
                memory = memory.checked_sub(size).unwrap_or_default();
            }
        }

        Ok(true)
    }
}
