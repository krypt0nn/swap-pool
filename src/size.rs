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
impl<T, const N: usize> SizeOf for [T; N] where T: SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        self.iter().map(T::size_of).sum::<usize>()
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
        std::mem::size_of_val(self) + self.iter().map(T::size_of).sum::<usize>()
    }
}

#[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
impl<T> SizeOf for &Vec<T> where T: SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        std::mem::size_of_val(*self) + self.iter().map(T::size_of).sum::<usize>()
    }
}

#[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
impl<T> SizeOf for Option<T> where T: SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        match self {
            Some(value) => std::mem::size_of_val(self) + value.size_of(),
            None => std::mem::size_of_val(self)
        }
    }
}

#[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
impl<T, E> SizeOf for Result<T, E> where T: SizeOf, E: SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        match self {
            Ok(value) => std::mem::size_of_val(self) + value.size_of(),
            Err(err) => std::mem::size_of_val(self) + err.size_of()
        }
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

#[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
impl<T> SizeOf for std::cell::Cell<T> where T: Default + SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        let value = self.take();
        let size = value.size_of();

        self.set(value);

        std::mem::size_of_val(self) + size
    }
}

#[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
impl<T> SizeOf for std::cell::RefCell<T> where T: SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        std::mem::size_of_val(self) + self.borrow().size_of()
    }
}

#[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
impl<T> SizeOf for std::cell::OnceCell<T> where T: SizeOf {
    #[inline]
    fn size_of(&self) -> usize {
        match self.get() {
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

    (deref $t:ty) => {
        #[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
        impl<T> SizeOf for $t where T: SizeOf {
            #[inline]
            fn size_of(&self) -> usize {
                use std::ops::Deref;

                std::mem::size_of_val(self) + self.deref().size_of()
            }
        }
    };

    ($t:ty$(,$f:ty)+) => {
        impl_for_type!($t);
        $(impl_for_type!($f);)+
    }
}

#[cfg(all(not(feature = "size-of-crate"), not(feature = "dyn-size-of-crate")))]
use std::{
    net::{Ipv4Addr, Ipv6Addr, IpAddr},
    sync::atomic::{
        AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize,
        AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize,
        AtomicBool
    }
};

impl_for_type!(i8, i16, i32, i64, i128, isize);
impl_for_type!(u8, u16, u32, u64, u128, usize);
impl_for_type!(f32, f64);
impl_for_type!(bool, char);
impl_for_type!(());

impl_for_type!(Ipv4Addr, Ipv6Addr, IpAddr);

impl_for_type!(AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize);
impl_for_type!(AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize);
impl_for_type!(AtomicBool);

impl_for_type!(len &str);
impl_for_type!(len &std::ffi::OsStr);

impl_for_type!(capacity String);
impl_for_type!(capacity std::ffi::OsString);
impl_for_type!(capacity std::path::PathBuf);

impl_for_type!(deref Box<T>);
impl_for_type!(deref std::rc::Rc<T>);
impl_for_type!(deref std::sync::Arc<T>);
