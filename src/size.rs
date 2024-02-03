pub trait SizeOf {
    /// Get current value size in bytes
    fn size_of(&self) -> usize;
}

#[cfg(all(feature = "dyn-size-of-crate", not(feature = "size-of-crate")))]
impl<T> SizeOf for T where T: dyn_size_of::GetSize {
    #[inline]
    fn size_of(&self) -> usize {
        self.size_bytes()
    }
}

#[cfg(all(feature = "size-of-crate", not(feature = "dyn-size-of-crate")))]
impl<T> SizeOf for T where T: size_of::SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        self.size_of().total_bytes()
    }
}

#[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
impl<T> SizeOf for &[T] where T: SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        self.iter().map(T::size_of).sum()
    }
}

#[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
impl<T> SizeOf for Vec<T> where T: SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        self.iter().map(T::size_of).sum()
    }
}

#[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
impl<T> SizeOf for std::sync::Weak<T> where T: SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        match self.upgrade() {
            Some(value) => std::mem::size_of_val(self) + value.size_of(),
            None => std::mem::size_of_val(self)
        }
    }
}

macro_rules! impl_for_type {
    ($t:ty) => {
        #[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
        impl SizeOf for $t {
            #[inline]
            fn size_of(&self) -> usize {
                std::mem::size_of_val(self)
            }
        }
    };

    (len $t:ty) => {
        #[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
        impl SizeOf for $t {
            #[inline]
            fn size_of(&self) -> usize {
                self.len()
            }
        }
    };

    (capacity $t:ty) => {
        #[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
        impl SizeOf for $t {
            #[inline]
            fn size_of(&self) -> usize {
                self.capacity()
            }
        }
    };
}

impl_for_type!(i8);
impl_for_type!(i16);
impl_for_type!(i32);
impl_for_type!(i64);
impl_for_type!(i128);

impl_for_type!(u8);
impl_for_type!(u16);
impl_for_type!(u32);
impl_for_type!(u64);
impl_for_type!(u128);

impl_for_type!(len &str);
impl_for_type!(len &std::ffi::OsStr);

impl_for_type!(capacity String);
impl_for_type!(capacity std::ffi::OsString);
impl_for_type!(capacity std::path::PathBuf);
