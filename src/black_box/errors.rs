use std::ops::BitAnd;

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
    /// let x = storage.get::<&usize>().unwrap();
    /// let y = storage.get::<&mut usize>();
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
    /// let x = storage.get::<&usize>();
    /// assert!(x.is_err());
    /// // Error, there is no unit for `usize` allocated!
    /// drop(x);
    /// storage.allocate_for::<usize>();
    /// storage.insert::<usize>(10);
    /// let x = storage.get::<&usize>().unwrap();
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
