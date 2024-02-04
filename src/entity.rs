use std::path::PathBuf;
use std::sync::Arc;

use super::size::SizeOf;
use super::inplace_cell::InplaceCell;
use super::uuid;
use super::error::{SwapResult, SwapError};
use super::handle::SwapHandle;

pub struct SwapEntity<T> {
    value: InplaceCell<Option<T>>,
    handle: Arc<SwapHandle<T>>,
    uuid: u64,
    path: PathBuf
}

impl<T> SwapEntity<T> {
    #[inline]
    /// Get entity's swap pool handle
    pub fn handle(&self) -> &Arc<SwapHandle<T>> {
        &self.handle
    }

    #[inline]
    /// Get entity's unique id
    pub fn uuid(&self) -> u64 {
        self.uuid
    }

    #[inline]
    /// Upgrade entity's rank
    pub fn upgrade(&self) -> u64 {
        self.handle.upgrade_entity(self.uuid)
    }

    #[inline]
    /// Get entity's rank
    pub fn rank(&self) -> u64 {
        self.handle.rank_entity(self.uuid)
    }
}

impl<T> SwapEntity<T> where T: Clone {
    #[inline]
    /// Check if the inner value is stored in the RAM right now
    pub fn is_hot(&self) -> bool {
        self.value.get_ref().is_some()
    }

    #[inline]
    /// Check if the inner value is stored on the disk right now
    pub fn is_cold(&self) -> bool {
        self.value.get_ref().is_none()
    }
}

impl<T> SwapEntity<T> where T: Clone + SizeOf {
    #[inline]
    /// Get size of the entity's value
    pub fn value_size(&self) -> SwapResult<usize> {
        match self.value.get_ref().as_ref() {
            Some(value) => Ok(value.size_of()),
            None => Ok(usize::try_from(self.path.metadata()?.len()).unwrap())
        }
    }
}

impl<T> SwapEntity<T>
where
    T: TryFrom<Vec<u8>> + TryInto<Vec<u8>> + Clone + SizeOf,
    <T as TryFrom<Vec<u8>>>::Error: std::error::Error + 'static,
    <T as TryInto<Vec<u8>>>::Error: std::error::Error + 'static
{
    /// Create new entity and flush it to the disk if there's no space available
    pub fn create(value: T, handle: Arc<SwapHandle<T>>, path: impl Into<PathBuf>) -> SwapResult<Self> {
        let path: PathBuf = path.into();

        // We expect the path to be unique for each entity
        let uuid = uuid::get(&path);

        if value.size_of() > handle.available() {
            let value: Vec<u8> = value.try_into()
                .map_err(|err| SwapError::Serialize(Box::new(err)))?;

            std::fs::write(&path, value)?;

            Ok(SwapEntity {
                value: InplaceCell::new(None),
                handle,
                uuid,
                path
            })
        } else {
            Ok(SwapEntity {
                value: InplaceCell::new(Some(value)),
                handle,
                uuid,
                path
            })
        }
    }

    #[inline]
    /// Get entity's value from the RAM or read it from the disk
    /// 
    /// This method will make the entity hot if the pool has
    /// enough memory available, or keep it cold otherwise
    pub fn value(&self) -> SwapResult<T> {
        self.upgrade();

        let value = self.value.update_result(|value| {
            let raw_value = match value.take() {
                Some(value) => value,
                None => T::try_from(std::fs::read(&self.path)?)
                    .map_err(|err| SwapError::Deserialize(Box::new(err)))?
            };

            // Calculate amount of memory which is needed to be freed to store the value
            let free = raw_value.size_of()
                .checked_sub(self.handle.available())
                .unwrap_or_default();

            // Free some memory if it's needed, and store the value
            // if we have enough space available
            if free == 0 || self.handle.free(free)? {
                *value = Some(raw_value.clone());
            }

            Ok::<_, SwapError>(raw_value)
        })?;

        Ok(value)
    }

    #[inline]
    /// Get entity's value from the RAM or read it from the disk,
    /// and flush the value afterwards
    /// 
    /// This method is needed to keep the entity cold.
    /// It also will not increment the entity's keep alive rank
    /// 
    /// Use it if you need to access value once
    pub fn value_unallocate(&self) -> SwapResult<T> {
        self.value.update_result(|value| {
            match value.take() {
                Some(value) => Ok(value),
                None => Ok(T::try_from(std::fs::read(&self.path)?)
                    .map_err(|err| SwapError::Deserialize(Box::new(err)))?)
            }
        })
    }

    #[inline]
    /// Get entity's value from the RAM or read it from the disk
    /// 
    /// This method will make the entry hot, even if there's no
    /// free space available in the pool
    /// 
    /// Use it if you need to access value frequently
    pub fn value_allocate(&self) -> SwapResult<T> {
        self.upgrade();

        self.value.update_result(|value| {
            if value.is_none() {
                *value = Some(T::try_from(std::fs::read(&self.path)?)
                    .map_err(|err| SwapError::Deserialize(Box::new(err)))?);
            }

            Ok::<_, SwapError>(())
        })?;

        unsafe {
            Ok(self.value.get_copy().unwrap_unchecked())
        }
    }

    #[inline]
    /// Update current entity's value
    /// 
    /// This method will try to free enough memory
    /// for the updated value. It also can fail to update
    /// the value if it can't free needed amount of memory
    /// and return `Ok(false)`
    /// 
    /// Use `replace` instead if you're sure that
    /// it will take less or equal amount of memory
    // TODO: don't flush the value before we're sure that it's needed
    pub fn update(&self, value: T) -> SwapResult<bool> {
        // Flush the entity making it cold
        self.flush()?;

        // Calculate amount of memory which is needed to be freed to store the value
        let free = value.size_of()
            .checked_sub(self.handle.available() + self.size_of())
            .unwrap_or_default();

        // Free some memory if it's needed, and store the value
        // if we have enough space available
        if free == 0 || self.handle.free(free)? {
            // Replace the value
            self.value.replace_by(Some(value));

            // This is technically not needed but I do this anyway
            // for some ideological consistency
            if self.path.exists() {
                std::fs::remove_file(&self.path)?;
            }

            Ok(true)
        }

        else {
            Ok(false)
        }
    }

    #[inline]
    /// Replace current entity's value
    /// 
    /// This method will not check if there's enough memory available
    /// so it works faster than `update`
    pub fn replace(&self, value: T) -> SwapResult<()> {
        self.value.update(move |old_value| *old_value = Some(value));

        // This is technically not needed but I do this anyway
        // for some ideological consistency
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }

        Ok(())
    }

    #[inline]
    /// Flush stored value to the disk, making current entity cold
    pub fn flush(&self) -> SwapResult<()> {
        self.value.update_result(|value| {
            if let Some(value) = value.take() {
                let value: Vec<u8> = value.try_into()
                    .map_err(|err| SwapError::Serialize(Box::new(err)))?;

                std::fs::write(&self.path, value)?;
            }

            Ok(())
        })
    }
}

impl<T> SizeOf for SwapEntity<T> where T: Clone + SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        self.value.size_of() + std::mem::size_of_val(&self.handle) + self.path.capacity()
    }
}

impl<T> Drop for SwapEntity<T> {
    #[inline]
    fn drop(&mut self) {
        if self.path.exists() {
            // TODO: panic?
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
