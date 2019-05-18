//! A dynamically allocated storage system. Check it out on [Github][gh], or its capabilities on the
//! [tests][ts]. This is meant to serve as a storage solution for resources in a dynamic context. It
//! supports runtime borrow checking using [`RefCell`][rc]s. It will support a concurrent context in
//! the future.
//!
//! [gh]: https://github.com/OptimisticPeach/restor
//! [ts]: https://github.com/OptimisticPeach/restor
//! [rc]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
//!
//! ## Example
//! ```
//! # use restor::*;
//! # fn main() {
//! let mut storage = DynamicStorage::new();
//! storage.allocate_for::<usize>();
//! storage.allocate_for::<String>();
//! storage.insert::<String>("abc".into());
//! let mut my_string = storage.get_mut::<String>().unwrap();
//! storage.insert_many::<usize>(vec![2, 4, 8, 16, 32]);
//! for i in 0..5 {
//!     *my_string = format!("{}, {:?}", &*my_string, *storage.ind::<usize>(i).unwrap());
//! }
//! assert_eq!("abc, 2, 4, 8, 16, 32", &*my_string);
//! # }
//! ```
//!
mod black_box;
mod concurrent_black_box;

pub type MutexStorage = BlackBox<
    (dyn for<'a> Unit<
        'a,
        Borrowed = MappedMutexGuard<'a, (dyn Any + Send)>,
        MutBorrowed = MappedMutexGuard<'a, (dyn Any + Send)>,
        Owned = Box<(dyn Any + Send)>,
    > + Send),
>;
pub type DynamicStorage = BlackBox<
    (dyn for<'a> Unit<
        'a,
        Borrowed = Ref<'a, (dyn Any + Send)>,
        MutBorrowed = RefMut<'a, (dyn Any + Send)>,
        Owned = Box<(dyn Any + Send)>,
    >),
>;

///
/// Shorthand for forming storage with preallocated types
///
/// # Example
/// ```
/// use restor::{DynamicStorage, make_storage};
/// let x: DynamicStorage = make_storage!(DynamicStorage: usize, String, isize);
/// x.insert(0usize).unwrap();
/// x.insert(String::new()).unwrap();
/// x.insert(1isize).unwrap();
/// ```
///
#[macro_export]
macro_rules! make_storage {
    ($storagetype:ty $(: $($contents:ty),*)? ) => {
        {
            let mut storage = <$storagetype>::new();
            $(
                $(
                    storage.allocate_for::<$contents>();
                )*
            )?
            storage
        }
    };
    (Arc<$storagetype:ty> $(: $($contents:ty),*)? ) => {
        {
            let mut storage = $storagetype::new();
            $(
                $(
                    storage.allocate_for::<$contents>();
                )*
            )?
            ::std::arc::Arc::new(storage)
        }
    }
}

pub use black_box::{
    BlackBox, ErrorDesc, MutexUnitTrait, RefCellUnitTrait, RwLockUnitTrait, Unit, UnitError,
};
pub use concurrent_black_box::{MutexUnit, RwLockStorage, RwLockUnit};
use parking_lot::MappedMutexGuard;
use std::any::Any;
use std::cell::{Ref, RefMut};
