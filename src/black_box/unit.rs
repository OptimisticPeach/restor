use super::errors::*;
use parking_lot::{MappedMutexGuard, MappedRwLockReadGuard, MappedRwLockWriteGuard};
use std::any::{Any, TypeId};
use std::ops::{Deref, DerefMut};

///
/// The type erasure trait for `restor`.
///
/// This contains an interface for interacting with a `StorageUnit`
/// wrapper, and should be ignored by the end user.
///
/// Exposed here are two types:
/// - `Borrowed` which must deref to a `dyn Any`
///   - [`Ref`] in the case of `DynamicStorage`
///   - [`MappedRwLockReadGuard`] in the case of `RwLockStorage`
///   - [`MappedMutexGuard`] in the case of `MutexStorage`
/// - `MutBorrowed` which must deref_mut to a `dyn Any`
///   - [`RefMut`] in the case of `DynamicStorage`
///   - [`MappedRwLockWriteGuard`] in the case of `MutexStorage`
///   - [`MappedMutexGuard`] in the case of `MutexStorage`
///
/// [`Ref`]: https://doc.rust-lang.org/std/cell/struct.Ref.html
/// [`RefMut`]: https://doc.rust-lang.org/std/cell/struct.RefMut.html
///
/// [`MappedRwLockReadGuard`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.MappedRwLockReadGuard.html
/// [`MappedRwLockWriteGuard`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.MappedRwLockWriteGuard.html
///
/// [`MappedMutexGuard`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.MappedMutexGuard.html
///
pub trait Unit<'a> {
    type Borrowed: Deref<Target = dyn Any> + 'a;
    type MutBorrowed: Deref<Target = dyn Any> + DerefMut + 'a;
    ///
    /// Inserts an owned piece of data into storage, returning it if
    /// it cannot be inserted.
    ///
    fn insert_any(&self, new: Box<dyn Any>) -> Option<(Box<dyn Any>, ErrorDesc)>;

    ///
    /// Waits to insert a value into the storage.
    ///
    fn waiting_insert(&self, new: Box<dyn Any>) -> Option<(Box<dyn Any>, ErrorDesc)>
    where
        Self::Borrowed: Waitable,
        Self::MutBorrowed: Waitable;

    ///
    /// Returns an immutable lock to the internal `StorageUnit<T>`
    ///
    fn storage(&'a self) -> DynamicResult<Self::Borrowed>;
    ///
    /// Returns a mutable lock to the internal `StorageUnit<T>`
    ///
    fn storage_mut(&'a self) -> DynamicResult<Self::MutBorrowed>;

    fn waiting_storage(&'a self) -> Self::Borrowed
    where
        Self::Borrowed: Waitable;
    fn waiting_storage_mut(&'a self) -> Self::MutBorrowed
    where
        Self::MutBorrowed: Waitable;

    ///
    /// Returns the `TypeId` of the type of data contained in the
    /// `StorageUnit<T>` (So the `TypeId` of `T`).
    ///
    fn id(&self) -> TypeId;
}

pub trait Waitable {}

impl<'b, T: ?Sized> Waitable for MappedMutexGuard<'b, T> {}
impl<'b, T: ?Sized> Waitable for MappedRwLockReadGuard<'b, T> {}
impl<'b, T: ?Sized> Waitable for MappedRwLockWriteGuard<'b, T> {}

#[cfg(test)]
mod tests {
    use super::Unit;
    use crate::black_box::StorageUnit;
    use crate::concurrent_black_box::RwLockUnit;
    use std::any::TypeId;

    #[test]
    fn insert() {
        let storage = RwLockUnit::new(StorageUnit::<usize>::new());
        storage.insert_any(Box::new(0usize) as _);
        storage.insert_any(Box::new(vec![1usize, 2, 3, 4]));
        assert_eq!(
            storage.inner().read().many().unwrap(),
            &[0usize, 1, 2, 3, 4]
        );
    }

    #[test]
    fn waiting_insert() {
        let storage = RwLockUnit::new(StorageUnit::<usize>::new());
        storage.waiting_insert(Box::new(0usize) as _);
        storage.waiting_insert(Box::new(vec![1usize, 2, 3, 4]));
        assert_eq!(
            storage.inner().read().many().unwrap(),
            &[0usize, 1, 2, 3, 4]
        );
    }

    #[test]
    fn id() {
        let storage = RwLockUnit::new(StorageUnit::<usize>::new());
        assert_eq!(storage.id(), TypeId::of::<usize>());
    }

    #[test]
    fn eq() {
        let storage = RwLockUnit::new(StorageUnit::<usize>::new());
        let storage_2 = RwLockUnit::new(StorageUnit::<usize>::new());
        assert_eq!(storage.id(), storage_2.id());
    }
}
