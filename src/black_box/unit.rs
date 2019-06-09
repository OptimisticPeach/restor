use super::errors::*;
use parking_lot::{MappedMutexGuard, MappedRwLockReadGuard, MappedRwLockWriteGuard};
use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

///
/// The type erasure trait for `restor`.
///
/// This contains an interface for interacting with a `StorageUnit`
/// wrapper, and should be ignored by the end user.
///
/// Exposed here are three types:
/// - `Borrowed` which must deref to a `dyn Any`
///   - [`Ref`] in the case of `DynamicStorage`
///   - [`MappedRwLockReadGuard`] in the case of `RwLockStorage`
///   - [`MappedMutexGuard`] in the case of `MutexStorage`
/// - `MutBorrowed` which must deref_mut to a `dyn Any`
///   - [`RefMut`] in the case of `DynamicStorage`
///   - [`MappedRwLockWriteGuard`] in the case of `MutexStorage`
///   - [`MappedMutexGuard`] in the case of `MutexStorage`
/// - `Owned` which must deref to a `dyn Any`, usually `Box<dyn Any>`
///   - [`Box`] in the case of `DynamicStorage`, `MutexStorage`, and `RwLockStorage`
///
/// [`Ref`]: https://doc.rust-lang.org/std/cell/struct.Ref.html
/// [`RefMut`]: https://doc.rust-lang.org/std/cell/struct.RefMut.html
///
/// [`MappedRwLockReadGuard`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.MappedRwLockReadGuard.html
/// [`MappedRwLockWriteGuard`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.MappedRwLockWriteGuard.html
///
/// [`MappedMutexGuard`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.MappedMutexGuard.html
///
/// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
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

impl<'a, R: Deref<Target = dyn Any> + 'a, RM: Deref<Target = dyn Any> + DerefMut + 'a> PartialEq
    for dyn Unit<'a, Borrowed = R, MutBorrowed = RM>
{
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<'a, R: Deref<Target = dyn Any> + 'a, RM: Deref<Target = dyn Any> + DerefMut + 'a> Debug
    for dyn Unit<'a, Borrowed = R, MutBorrowed = RM>
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Unit(TypeId: {:?})", self.id())
    }
}

pub trait Waitable {}

impl<'b> Waitable for MappedMutexGuard<'b, dyn Any> {}
impl<'b> Waitable for MappedRwLockReadGuard<'b, dyn Any> {}
impl<'b> Waitable for MappedRwLockWriteGuard<'b, dyn Any> {}
