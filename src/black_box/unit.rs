use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter};
use std::mem::swap;
use std::ops::{BitAnd, Deref, DerefMut};

pub type DynamicResult<Ok> = Result<Ok, ErrorDesc>;

///
/// The basic error descriptions for why a dynamically typed resource operation didn't work. It does
/// not contain however, the description for unit-related errors which handled with a `UnitError` by
/// using the `Unit` variant of `ErrorDesc`.
///
/// # Note
/// This implements [`BitAnd`]
///
/// Used for combining errors; This will combine certain errors into more concise errors.
/// ## Example
/// ```
/// use restor::{ErrorDesc, UnitError};
/// let a = ErrorDesc::Unit(UnitError::OutOfBounds);
/// let b = ErrorDesc::BorrowedIncompatibly;
/// let combo = a & b;
/// assert_eq!(
///     combo,
///     ErrorDesc::Two(Box::new(
///                 (ErrorDesc::Unit(UnitError::OutOfBounds),
///                  ErrorDesc::BorrowedIncompatibly)
///     ))
/// );
/// let a = ErrorDesc::Unit(UnitError::IsNotMany);
/// let b = ErrorDesc::Unit(UnitError::IsNotOne);
/// assert_eq!(
///     a & b,
///     ErrorDesc::Unit(UnitError::IsNope)
/// );
/// ```
/// [`BitAnd`]: https://doc.rust-lang.org/std/ops/trait.BitAnd.html
///
#[derive(Debug, PartialEq, Clone)]
pub enum ErrorDesc {
    /// Returned if there is an incompatible borrow on the contents of the unit. It follows the same
    /// rules for runtime checking as a `RefCell<T>`. Usually bundled with a `Ref<T>`/`RefMut<T>` in
    /// a `Result<RefVariant<T>, ErrorDesc>`.
    /// ## Example:
    /// ```
    /// # use restor::*;
    /// # fn main() {
    /// let mut storage = DynamicStorage::new();
    /// storage.allocate_for::<usize>();
    /// storage.insert(0usize);
    /// let x = storage.get::<usize>().unwrap();
    /// let y = storage.get_mut::<usize>();
    /// assert!(y.is_err());
    /// # }
    /// ```
    BorrowedIncompatibly,
    /// Returned when there is no unit allocated for the type that was requested. Allocate a unit to
    /// contain a `<T>` with `DynamicStorage::allocate_for::<T>(&mut self)`. Note that `<T>` must be
    /// `T: Sized + Any + 'static`.
    /// ## Example:
    /// ```
    /// # use restor::*;
    /// # fn main() {
    /// let mut storage = DynamicStorage::new();
    /// let x = storage.get::<usize>();
    /// assert!(x.is_err());
    /// // Error, there is no unit for `usize` allocated!
    /// drop(x);
    /// storage.allocate_for::<usize>();
    /// storage.insert::<usize>(10);
    /// let x = storage.get::<usize>().unwrap();
    /// assert_eq!(*x, 10);
    /// # }
    /// ```
    NoAllocatedUnit,
    /// This is an internal error that should be ignored by the user. This should never be created.
    NoMatchingType,
    /// Contains an error specific to unit operations. Please refer to the `UnitError` documentation
    /// for more information.
    Unit(UnitError),
    /// The case where there were two errors
    Two(Box<(ErrorDesc, ErrorDesc)>),
}

impl BitAnd for ErrorDesc {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        match (self, rhs) {
            (ErrorDesc::Unit(UnitError::IsNotMany), ErrorDesc::Unit(UnitError::IsNotOne)) => {
                ErrorDesc::Unit(UnitError::IsNope)
            }
            (x, y) => ErrorDesc::Two(Box::new((x, y))),
        }
    }
}

///
/// Miscellaneous errors pertaining to the internal `StorageUnit`,
/// such as an out of bounds error, or improper accessing of data.
///
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnitError {
    ///
    /// Created when a `One` variant of data was requested, but either
    /// `Many` or `Nope` were presented.
    ///
    /// Describes when there is either more than one or zero pieces of
    /// data in the storage, but the function requires there to be one.
    ///
    IsNotOne,
    ///
    /// Created when a `Many` variant of data was requested, but either
    /// `One` or `Nope` were presented.
    ///
    /// Describes when there is either one or zero pieces of data in the
    /// storage, but the function requires there to be more than one.
    ///
    IsNotMany,
    ///
    /// Created when either `One` or `Many` variants were requested but
    /// a `Nope` variant was presented.
    ///
    /// Describes when any amount of data was requested from the storage,
    /// but there was no data in the storage.
    ///
    IsNope,
    ///
    /// Returned when the specified index is outside of the bounds of
    /// the `Vec<T>` contained within the storage.
    ///
    OutOfBounds,
}

///
/// The base storage unit for this library.
///
/// This is similar in implementation to [`SmallVec`], but is
/// specialized to this library's needs. In the future it is
/// possibly that this will simply be replaced with [`SmallVec`].
///
/// This should not be interacted with through the user, as
/// this is meant to be an internal implementation detail for
/// the user. This is usually abstracted through a type erased
/// `Unit`.
///
/// [`SmallVec`]: https://docs.rs/smallvec/0.6.9/smallvec/
///
pub enum StorageUnit<T: 'static> {
    Nope,
    One(T),
    Many(Vec<T>),
}

impl<T: Sized> StorageUnit<T> {
    pub fn new() -> Self {
        StorageUnit::Nope
    }

    pub fn insert(&mut self, new: T) {
        match self {
            StorageUnit::Nope => {
                *self = StorageUnit::One(new);
            }
            StorageUnit::One(_) => {
                let mut rep = StorageUnit::Many(vec![new]);
                swap(self, &mut rep);
                if let StorageUnit::One(prev) = rep {
                    if let StorageUnit::Many(v) = self {
                        v.insert(0, prev);
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            StorageUnit::Many(many) => {
                many.push(new);
            }
        }
    }

    pub fn insert_many(&mut self, mut new: Vec<T>) {
        match self {
            StorageUnit::Nope => {
                *self = StorageUnit::Many(new);
            }
            StorageUnit::One(_) => {
                let mut rep = StorageUnit::Many(new);
                swap(&mut rep, self);
                if let StorageUnit::One(val) = rep {
                    if let StorageUnit::Many(vec) = self {
                        vec.insert(0, val);
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            StorageUnit::Many(arr) => {
                arr.append(&mut new);
            }
        }
    }

    pub fn one(&self) -> DynamicResult<&T> {
        if let StorageUnit::One(x) = self {
            Ok(x)
        } else {
            Err(ErrorDesc::Unit(UnitError::IsNotOne))
        }
    }

    pub fn one_mut(&mut self) -> DynamicResult<&mut T> {
        if let StorageUnit::One(x) = self {
            Ok(x)
        } else {
            Err(ErrorDesc::Unit(UnitError::IsNotOne))
        }
    }

    pub fn many(&self) -> DynamicResult<&[T]> {
        if let StorageUnit::Many(x) = self {
            Ok(x)
        } else {
            Err(ErrorDesc::Unit(UnitError::IsNotMany))
        }
    }

    pub fn many_mut(&mut self) -> DynamicResult<&mut Vec<T>> {
        if let StorageUnit::Many(x) = self {
            Ok(x)
        } else {
            Err(ErrorDesc::Unit(UnitError::IsNotMany))
        }
    }

    pub fn extract_one(&mut self) -> DynamicResult<T> {
        match self {
            StorageUnit::Nope => Err(ErrorDesc::Unit(UnitError::IsNotOne)),
            StorageUnit::Many(x) => Ok(x.remove(0)),
            StorageUnit::One(_) => {
                let mut repl = StorageUnit::Nope;
                swap(&mut repl, self);
                if let StorageUnit::One(data) = repl {
                    Ok(data)
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub fn extract_many(&mut self) -> DynamicResult<Vec<T>> {
        match self {
            StorageUnit::Nope => Err(ErrorDesc::Unit(UnitError::IsNotMany)),
            StorageUnit::One(_) => {
                let mut repl = StorageUnit::Nope;
                swap(&mut repl, self);
                if let StorageUnit::One(data) = repl {
                    Ok(vec![data])
                } else {
                    unreachable!()
                }
            }
            StorageUnit::Many(_) => {
                let mut repl = StorageUnit::Nope;
                swap(&mut repl, self);
                if let StorageUnit::Many(data) = repl {
                    Ok(data)
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub fn extract_many_boxed(&mut self) -> DynamicResult<Box<[T]>> {
        match self {
            StorageUnit::Nope => Err(ErrorDesc::Unit(UnitError::IsNotMany)),
            StorageUnit::One(_) => Err(ErrorDesc::Unit(UnitError::IsNotMany)),
            StorageUnit::Many(_) => {
                let mut repl = StorageUnit::Nope;
                swap(&mut repl, self);
                if let StorageUnit::Many(data) = repl {
                    Ok(data.into_boxed_slice())
                } else {
                    unreachable!()
                }
            }
        }
    }
}

impl<T> Default for StorageUnit<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for StorageUnit<T> {
    fn clone(&self) -> Self {
        match self {
            StorageUnit::Nope => StorageUnit::Nope,
            StorageUnit::One(data) => StorageUnit::One(data.clone()),
            StorageUnit::Many(data) => StorageUnit::Many(data.clone()),
        }
    }
}

///
/// The type erasure trait for `restor`.
///
/// This contains an interface for interacting with a `StorageUnit`
/// wrapper.
///
/// Exposed here are three types:
/// - `Borrowed` which must deref to a `(dyn Any + Send)`
///   - [`Ref`] in the case of `DynamicStorage`
///   - [`MappedRwLockReadGuard`] in the case of `RwLockStorage`
///   - [`MappedMutexGuard`] in the case of `MutexStorage`
/// - `MutBorrowed` which must deref_mut to a `(dyn Any + Send)`
///   - [`RefMut`] in the case of `DynamicStorage`
///   - [`MappedRwLockWriteGuard`] in the case of `MutexStorage`
///   - [`MappedMutexGuard`] in the case of `MutexStorage`
/// - `Owned` which must deref to a `(dyn Any + Send)`, usually `Box<(dyn Any + Send)>`
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
    type Borrowed: Deref<Target = (dyn Any + Send)> + 'a;
    type MutBorrowed: Deref<Target = (dyn Any + Send)> + DerefMut + 'a;
    type Owned: Deref<Target = (dyn Any + Send)> + DerefMut;

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
    unsafe fn run_for(&self, func: (TypeId, (*const (), *const ()))) -> Option<Box<dyn Any>>;

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
        R: Deref<Target = (dyn Any + Send)> + 'a,
        RM: Deref<Target = (dyn Any + Send)> + DerefMut + 'a,
        O: Deref<Target = (dyn Any + Send)> + DerefMut,
    > PartialEq for dyn Unit<'a, Borrowed = R, MutBorrowed = RM, Owned = O>
{
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl<
        'a,
        R: Deref<Target = (dyn Any + Send)> + 'a,
        RM: Deref<Target = (dyn Any + Send)> + DerefMut + 'a,
        O: Deref<Target = (dyn Any + Send)> + DerefMut,
    > Debug for dyn Unit<'a, Borrowed = R, MutBorrowed = RM, Owned = O>
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Unit(TypeId: {:?})", self.id())
    }
}
