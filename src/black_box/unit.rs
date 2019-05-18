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
///     ErrorDesc::Unit(UnitError::IsNone)
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
                ErrorDesc::Unit(UnitError::IsNone)
            }
            (x, y) => ErrorDesc::Two(Box::new((x, y))),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum UnitError {
    IsNotOne,
    IsNotMany,
    IsNone,
    OutOfBounds,
}

pub enum StorageUnit<T: Sized + 'static> {
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

pub trait Unit<'a> {
    type Borrowed: Deref<Target = (dyn Any + Send)> + 'a;
    type MutBorrowed: Deref<Target = (dyn Any + Send)> + DerefMut + 'a;
    type Owned: Deref<Target = (dyn Any + Send)> + DerefMut;

    fn one(&'a self) -> DynamicResult<Self::Borrowed>;
    fn one_mut(&'a self) -> DynamicResult<Self::MutBorrowed>;

    fn ind(&'a self, ind: usize) -> DynamicResult<Self::Borrowed>;
    fn ind_mut(&'a self, ind: usize) -> DynamicResult<Self::MutBorrowed>;

    fn extract(&self) -> DynamicResult<Self::Owned>;
    fn extract_ind(&self, ind: usize) -> DynamicResult<Self::Owned>;
    fn extract_many(&self) -> DynamicResult<Self::Owned>;

    fn insert_any(&self, new: Self::Owned) -> Option<(Self::Owned, ErrorDesc)>;
    unsafe fn run_for(&self, func: (TypeId, (*const (), *const ()))) -> Option<Box<dyn Any>>;

    fn storage(&'a self) -> DynamicResult<Self::Borrowed>;
    fn storage_mut(&'a self) -> DynamicResult<Self::MutBorrowed>;

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
