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

    unsafe fn run_for(
        &self,
        (t, ptr): (TypeId, (*const (), *const ())),
    ) -> DynamicResult<Box<dyn Any>> {
        if t == TypeId::of::<dyn FnMut(DynamicResult<&[T]>) -> Box<dyn Any>>() {
            if let Ok(x) = self.inner.try_borrow() {
                let func =
                    std::mem::transmute::<_, &mut dyn Fn(DynamicResult<&[T]>) -> Box<dyn Any>>(ptr);
                Ok(func(x.many()))
            } else {
                Err(ErrorDesc::BorrowedIncompatibly)
            }
        } else if t == TypeId::of::<dyn FnMut(DynamicResult<&mut Vec<T>>) -> Box<dyn Any>>() {
            if let Ok(mut x) = self.inner.try_borrow_mut() {
                let func = std::mem::transmute::<
                    _,
                    &mut dyn Fn(DynamicResult<&mut Vec<T>>) -> Box<dyn Any>,
                >(ptr);
                let res = func(x.many_mut());
                x.rearrange_if_necessary();
                Ok(res)
            } else {
                Err(ErrorDesc::BorrowedIncompatibly)
            }
        } else {
            panic!("Wrong function type passed to `run_for`!");
        }
    }

    fn id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}
