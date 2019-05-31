use super::errors::*;
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
    type Owned: Deref<Target = dyn Any> + DerefMut;

    ///
    /// Returns an immutable lock to one piece of data.
    ///
    fn one(&'a self) -> DynamicResult<Self::Borrowed>;
    ///
    /// Returns a mutable lock to one piece of data.
    ///
    fn one_mut(&'a self) -> DynamicResult<Self::MutBorrowed>;

    ///
    /// Indexes into a `Vec` given a particular index and
    /// returns an immutable lock to that data.
    ///
    fn ind(&'a self, ind: usize) -> DynamicResult<Self::Borrowed>;
    ///
    /// Indexes into a `Vec` given a particular index and
    /// returns a mutable lock to that data.
    ///
    fn ind_mut(&'a self, ind: usize) -> DynamicResult<Self::MutBorrowed>;

    ///
    /// Extracts an owned piece of data and returns it.
    ///
    fn extract(&self) -> DynamicResult<Self::Owned>;
    ///
    /// Extracts an owned piece of data at a given index and returns it.
    ///
    fn extract_ind(&self, ind: usize) -> DynamicResult<Self::Owned>;
    ///
    /// Extracts many owned pieces of data, and returns a `Box<Vec<T>>`.
    ///
    fn extract_many(&self) -> DynamicResult<Self::Owned>;

    ///
    /// Inserts an owned piece of data into storage, returning it if
    /// it cannot be inserted.
    ///
    fn insert_any(&self, new: Self::Owned) -> Option<(Self::Owned, ErrorDesc)>;
    ///
    /// Runs a given function on a `DynamicResult<&[T]>`, and returns the
    /// result of a given function.
    ///
    /// # Unsafety
    /// This will panic if it is given the wrong `TypeId` and will run
    /// undefined behaviour in the case that it has a wrong function pointer
    /// passed to it.
    ///
    /// # Calling
    /// The parameter is laid out as such:
    ///
    /// `(TypeId, (*const (), *const ())`
    ///
    /// - The `TypeId` is used to assure that the data being passed to it is
    /// a `dyn FnMut(DynamicResult<&[T]>) -> Option<Box<dyn Any>`.
    /// - The tuple containing two pointers is a fat pointer to the function
    ///   and the functions's vtable. This should not be created by the caller
    ///   and should instead be `std::mem::transmute`d from a preexisting function.
    ///
    unsafe fn run_for(&self, func: (TypeId, (*const (), *const ())))
        -> DynamicResult<Box<dyn Any>>;

    ///
    /// Returns an immutable lock to the internal `StorageUnit<T>`
    ///
    fn storage(&'a self) -> DynamicResult<Self::Borrowed>;
    ///
    /// Returns a mutable lock to the internal `StorageUnit<T>`
    ///
    fn storage_mut(&'a self) -> DynamicResult<Self::MutBorrowed>;

    ///
    /// Returns the `TypeId` of the type of data contained in the
    /// `StorageUnit<T>` (So the `TypeId` of `T`).
    ///
    fn id(&self) -> TypeId;
}

impl<
        'a,
        R: Deref<Target = dyn Any> + 'a,
        RM: Deref<Target = dyn Any> + DerefMut + 'a,
        O: Deref<Target = dyn Any> + DerefMut,
    > PartialEq for dyn Unit<'a, Borrowed = R, MutBorrowed = RM, Owned = O>
{
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<
        'a,
        R: Deref<Target = dyn Any> + 'a,
        RM: Deref<Target = dyn Any> + DerefMut + 'a,
        O: Deref<Target = dyn Any> + DerefMut,
    > Debug for dyn Unit<'a, Borrowed = R, MutBorrowed = RM, Owned = O>
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Unit(TypeId: {:?})", self.id())
    }
}
