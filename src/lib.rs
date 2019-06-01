#![deny(bare_trait_objects)]
#![doc(html_logo_url = "https://optimisticpeach.github.io/Logo.svg")]

//!
//! A dynamically allocated storage system. Check it out on [Github][gh], or its capabilities on the
//! [tests][ts]. This is meant to serve as a storage solution for resources in a dynamic context. It
//! supports runtime borrow checking using [`RefCell`][rc]s. It will support a concurrent context in
//! the future.
//!
//! [gh]: https://github.com/OptimisticPeach/restor
//! [ts]: https://github.com/OptimisticPeach/restor/tree/master/tests
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
mod macros;

///
/// The type alias for storage with interior mutability based on
/// [`RefCell`]s, only allowing for it exist on one thread. This
/// library currently restrains what goes into the storage to
/// `T: Send` because of how it is written, but that will change
/// in the future. This is mostly used in single-threaded contexts,
/// for example, the examples in this library's documentation.
///
/// # Note
/// Please defer to the [`make_storage`](../macro.make_storage.html)
/// macro to create these with a shorthand.
///
/// [`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
///
pub type DynamicStorage = BlackBox<
    (dyn for<'a> Unit<
        'a,
        Borrowed = Ref<'a, dyn Any>,
        MutBorrowed = RefMut<'a, dyn Any>,
        Owned = Box<dyn Any>,
    >),
>;

///
/// Shorthand for forming storage with preallocated types.
/// It will also wrap it in an [`Arc`] (More below)
///
/// # Usage
/// The syntax for the macro is as follows:
/// ```no_run
/// # fn main() {
/// # use restor::make_storage;
/// # struct OtherTypes;
/// # struct StorageType;
/// # impl StorageType {
/// #     pub fn new() -> Self {StorageType}
/// #     pub fn allocate_for<T>(&mut self) {}
/// # }
/// make_storage!(StorageType); // -> StorageType
/// make_storage!(Arc StorageType); // -> Arc<StorageType>
/// make_storage!(StorageType: String, usize, isize, i32, OtherTypes); // -> StorageType with types preallocated
/// make_storage!(Arc StorageType: String, usize, isize, i32, OtherTypes); // -> StorageType with types preallocated
/// # }
/// ```
///
/// # Example
/// ```
/// use restor::{DynamicStorage, make_storage};
/// let x: DynamicStorage = make_storage!(DynamicStorage: usize, String, isize);
/// x.insert(0usize).unwrap();
/// x.insert(String::new()).unwrap();
/// ```
/// # Arc Example
/// ```
/// use restor::{RwLockStorage, make_storage};
/// let x = make_storage!(Arc RwLockStorage: usize, String);
/// let nx = x.clone();
/// std::thread::spawn( move || {
///     nx.insert(String::new()).unwrap();
/// });
/// x.insert(0usize);
/// ```
///
/// [`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
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
    (Arc $storage:ty $(: $($contents:ty),*)? ) => {
        {
            ::std::sync::Arc::new(make_storage!($storage $(: $($contents),*)?))
        }
    }
}

pub use black_box::{BlackBox, ErrorDesc, Get, IndMultiple, Multiple, SliceMany, Unit, UnitError};
pub use concurrent_black_box::{MutexStorage, RwLockStorage};
use std::any::Any;
use std::cell::{Ref, RefMut};
