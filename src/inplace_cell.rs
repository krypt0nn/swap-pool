use std::cell::Cell;

/// Inplace cells are standard rust `Cell`-s that
/// are updated "in place", meaning they won't loose
/// their value until the update is finished.
/// 
/// ### Standard cell:
/// 
/// ```text
/// Thread 1      Thread 2
/// cell.take()
///               cell.take()
/// cell.set()    ^^^^^^^^^^^ - this operation will
///                             return default value
///                             instead of what we
///                             initially had
/// 
/// ```
/// 
/// ### Inplace cell:
/// 
/// Inplace cell doesn't have take/set methods.
/// They are used here for better understanding.
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
    value: Cell<T>
}

impl<T> InplaceCell<T> {
    #[inline]
    /// Create new inplace cell
    pub fn new(value: T) -> Self {
        Self {
            value: Cell::new(value)
        }
    }
}

impl<T> InplaceCell<T> where T: Default + Clone {
    #[inline]
    /// Update stored value using updater,
    /// catching and returning error from the updater
    /// if it happens
    pub fn update_result<R, E>(&self, updater: impl FnOnce(&mut T) -> Result<R, E>) -> Result<R, E> {
        let mut value = self.value.take();

        self.value.set(value.clone());

        let result = updater(&mut value)?;

        self.value.set(value);

        Ok(result)
    }

    #[inline]
    /// Replace stored value by updater's result,
    /// catching and returning error from the updater
    /// if it happens
    pub fn replace_result<E>(&self, updater: impl FnOnce(T) -> Result<T, E>) -> Result<(), E> {
        let value = self.value.take();

        self.value.set(value.clone());
        self.value.set(updater(value)?);

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
    /// Replace stored value by a new one
    pub fn replace_by(&self, value: T) {
        self.value.replace(value);
    }

    #[inline]
    /// Clone stored value and return it
    pub fn get(&self) -> T {
        let value = self.value.take();

        self.value.set(value.clone());

        value
    }
}
