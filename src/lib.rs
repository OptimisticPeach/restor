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
//! storage.insert_many::<usize>(vec![2usize, 4, 8, 16, 32]);
//! let mut my_string = storage.get_mut::<String>();
//! for i in 0..5 {
//!     *my_string = format!("{:?}, {:?}", *my_string, *storage.ind::<usize>(i));
//! }
//! assert_eq!("abc, 2, 4, 8, 16, 32".to_string(), &*my_string);
//! # }
//! ```
//!
mod black_box;

pub use black_box::{BlackBox as DynamicStorage, ErrorDesc, UnitError, Unit};
