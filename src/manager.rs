use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::inplace_cell::InplaceCell;

/// Swap manager is needed to rank swap pool entities.
/// Entities with higher rank will be removed after
/// entities with lower rank
pub trait SwapManager {
    // Upgrade given entity's rank and return it
    fn upgrade(&self, uuid: u64) -> u64;

    /// Rank given entity for the pool garbage collector
    fn rank(&self, uuid: u64) -> u64;
}

/// Rank entities based on their last `upgrade()` call
/// 
/// This manager will request `SystemTime::now()` each time
/// you get a value of an entity, and return timestamp
/// in seconds as their ranks
/// 
/// If you have a high load system - consider using `SwapUpgradeCountManager`
/// or implementing your own variant
pub struct SwapLastUseManager {
    ranks: InplaceCell<HashMap<u64, u64>>
}

impl Default for SwapLastUseManager {
    #[inline]
    fn default() -> Self {
        Self::new(true)
    }
}

impl SwapLastUseManager {
    #[inline]
    pub fn new(thread_safe: bool) -> Self {
        Self {
            ranks: InplaceCell::new(HashMap::new(), thread_safe)
        }
    }
}

impl SwapManager for SwapLastUseManager {
    fn upgrade(&self, uuid: u64) -> u64 {
        let rank = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.ranks.update(move |ranks| {
            ranks.insert(uuid, rank);
        });

        rank
    }

    #[inline]
    fn rank(&self, uuid: u64) -> u64 {
        self.ranks.get_ref()
            .get(&uuid)
            .copied()
            .unwrap_or_default()
    }
}

/// Rank entities based on amount of their `upgrade()` calls
/// 
/// Has better performance than `SwapLastUseManager` because
/// it just increments a counter in the `HashMap`
pub struct SwapUpgradeCountManager {
    ranks: InplaceCell<HashMap<u64, u64>>
}

impl Default for SwapUpgradeCountManager {
    #[inline]
    fn default() -> Self {
        Self::new(true)
    }
}

impl SwapUpgradeCountManager {
    #[inline]
    pub fn new(thread_safe: bool) -> Self {
        Self {
            ranks: InplaceCell::new(HashMap::new(), thread_safe)
        }
    }
}

impl SwapManager for SwapUpgradeCountManager {
    fn upgrade(&self, uuid: u64) -> u64 {
        // We always return a value so it's absolutely safe (TM)
        unsafe {
            self.ranks.update_result::<u64, ()>(move |ranks| {
                let rank = ranks.get(&uuid)
                    .copied()
                    .unwrap_or_default() + 1;
    
                ranks.insert(uuid, rank);
    
                Ok(rank)
            }).unwrap_unchecked()
        }
    }

    #[inline]
    fn rank(&self, uuid: u64) -> u64 {
        self.ranks.get_ref()
            .get(&uuid)
            .copied()
            .unwrap_or_default()
    }
}
