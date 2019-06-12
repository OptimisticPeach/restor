use super::{BlackBox, Unit};
use crate::impl_unit;
use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};

use super::*;

pub struct RefCellUnit<T> {
    pub(crate) inner: RefCell<T>,
}

impl<T> RefCellUnit<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: RefCell::new(data),
        }
    }
}

// Any changes made to RefCell/Mutex/RwLock units are done first on this one, and then
// Must be copied onto the other ones.
impl<'a, T: 'static> Unit<'a> for RefCellUnit<StorageUnit<T>> {
    type Borrowed = Ref<'a, dyn Any>;
    type MutBorrowed = RefMut<'a, dyn Any>;

    fn insert_any(&self, new: Box<dyn Any>) -> Option<(Box<dyn Any>, ErrorDesc)> {
        let newtype = new.type_id();
        if let Ok(mut x) = self.inner.try_borrow_mut() {
            if new.is::<T>() {
                x.insert(*new.downcast::<T>().unwrap_or_else(|_| {
                    panic!(
                        "Tried to insert an object with type {:?} into a storage of type {:?}",
                        newtype,
                        TypeId::of::<T>()
                    )
                }));
                None
            } else if new.is::<Vec<T>>() {
                x.insert_many(*new.downcast::<Vec<T>>().unwrap());
                None
            } else {
                Some((new, ErrorDesc::NoMatchingType))
            }
        } else {
            Some((new, ErrorDesc::BorrowedIncompatibly))
        }
    }

    fn storage(&'a self) -> DynamicResult<Ref<'a, dyn Any>> {
        self.inner
            .try_borrow()
            .ok()
            .map(|x| Ref::map::<dyn Any, _>(x, |z| &*z))
            .ok_or(ErrorDesc::BorrowedIncompatibly)
    }
    fn storage_mut(&'a self) -> DynamicResult<RefMut<'a, dyn Any>> {
        self.inner
            .try_borrow_mut()
            .ok()
            .map(|x| RefMut::map::<dyn Any, _>(x, |z| &mut *z))
            .ok_or(ErrorDesc::BorrowedIncompatibly)
    }

    fn waiting_storage(&'a self) -> Ref<'a, dyn Any> {
        unreachable!()
    }
    fn waiting_storage_mut(&'a self) -> RefMut<'a, dyn Any> {
        unreachable!()
    }

    fn id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

type RefCellBlackBox =
    BlackBox<dyn for<'a> Unit<'a, Borrowed = Ref<'a, dyn Any>, MutBorrowed = RefMut<'a, dyn Any>>>;

///
/// The newtype for storage with interior mutability based on
/// [`RefCell`]s, only allowing for it exist on one thread.
///
/// # Note
/// Please refer to the [`make_storage`](../macro.make_storage.html)
/// macro to create storages using a shorthand.
///
/// [`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
///
#[repr(transparent)]
pub struct DynamicStorage {
    black_box: RefCellBlackBox,
}

impl DynamicStorage {
    pub fn new() -> Self {
        Self {
            black_box: RefCellBlackBox::new(),
        }
    }
}

impl_unit!(DynamicStorage, dyn Any, ('static), RefCellUnit(dyn for<'u> Unit<'u, Borrowed=Ref<'u, dyn Any>, MutBorrowed=RefMut<'u, dyn Any>>), RefMut, Ref, black_box, add_unmut);
