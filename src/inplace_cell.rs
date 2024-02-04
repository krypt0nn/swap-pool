use std::cell::{RefCell, Ref};

use super::size::SizeOf;

/// Inplace cells are standard rust `RefCell`-s that
/// are updated "in place", meaning they won't loose
/// their value until the update is finished.
/// 
/// ### Inplace cell (not thread safe):
/// 
/// Foundamentally the same as the standard `RefCell`.
/// 
/// ```text
/// Thread 1      Thread 2
/// cell.take()
///               cell.take()
///               ^^^^^^^^^^^ - this operation will
///                             return default value
///                             instead of what we
///                             initially had
/// cell.set()
///               cell.get()
///               ^^^^^^^^^^ - this operation will
///                            return updated value
/// 
/// ```
/// 
/// ### Inplace cell (thread safe):
/// 
/// Thread same variant clones inner value for mutating it
/// so you will get the old variant in a parallel
/// thread before it was updated. This costs some
/// performance, which is important for e.g. `SwapManager`-s.
/// 
/// ```text
/// Thread 1      Thread 2
/// cell.take()
///               cell.take()
///               ^^^^^^^^^^^ - this operation will
///                             return the same value
///                             as we initially had
/// cell.set()
///               cell.get()
///               ^^^^^^^^^^ - this operation will
///                            return updated value
/// 
/// ```
pub struct InplaceCell<T> {
    value: RefCell<T>,

    /// If true, then the cell's value
    /// will be cloned before updating
    thread_safe: bool
}

impl<T> InplaceCell<T> {
    #[inline]
    /// Create new inplace cell
    pub fn new(value: T, thread_safe: bool) -> Self {
        Self {
            value: RefCell::new(value),
            thread_safe
        }
    }

    #[inline]
    /// Replace stored value by a new one
    pub fn replace_by(&self, value: T) {
        self.value.replace(value);
    }

    #[inline]
    /// Clone stored value and return it
    pub fn get_ref(&self) -> Ref<'_, T> {
        self.value.borrow()
    }
}

impl<T> InplaceCell<T> where T: Default + Clone {
    #[inline]
    /// Update stored value using updater,
    /// catching and returning error from the updater
    /// if it happens
    pub fn update_result<R, E>(&self, updater: impl FnOnce(&mut T) -> Result<R, E>) -> Result<R, E> {
        let mut value = self.value.take();

        if self.thread_safe {
            self.value.replace(value.clone());
        }

        let result = updater(&mut value)?;

        self.value.replace(value);

        Ok(result)
    }

    #[inline]
    /// Replace stored value by updater's result,
    /// catching and returning error from the updater
    /// if it happens
    pub fn replace_result<E>(&self, updater: impl FnOnce(T) -> Result<T, E>) -> Result<(), E> {
        let value = self.value.take();

        if self.thread_safe {
            self.value.replace(value.clone());
        }

        self.value.replace(updater(value)?);

        Ok(())
    }

    #[inline]
    /// Update stored value using updater
    pub fn update(&self, updater: impl FnOnce(&mut T)) {
        let _ = self.update_result(move |value| {
            updater(value);

            Ok::<_, ()>(())
        });
    }

    #[inline]
    /// Replace stored value by updater's result
    pub fn replace(&self, updater: impl FnOnce(T) -> T) {
        let _ = self.replace_result(move |value| Ok::<_, ()>(updater(value)));
    }

    #[inline]
    /// Clone stored value and return it
    pub fn get_copy(&self) -> T {
        let value = self.value.take();

        self.value.replace(value.clone());

        value
    }
}

impl<T> SizeOf for InplaceCell<T> where T: Default + Clone + SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        let value = self.value.take();
        let size = value.size_of();

        self.value.replace(value);

        std::mem::size_of_val(self) + size
    }
}
