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
impl<'a, T: 'static + Send> Unit<'a> for RefCellUnit<StorageUnit<T>> {
    type Borrowed = Ref<'a, (dyn Any + Send)>;
    type MutBorrowed = RefMut<'a, (dyn Any + Send)>;
    type Owned = Box<(dyn Any + Send)>;
    fn one(&'a self) -> DynamicResult<Ref<'a, (dyn Any + Send)>> {
        if let Some(nx) = self.inner.try_borrow().ok() {
            match nx.one() {
                Ok(_) => Ok(Ref::map(nx, |nx| &*nx.one().unwrap())),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn one_mut(&'a self) -> DynamicResult<RefMut<'a, (dyn Any + Send)>> {
        if let Some(mut nx) = self.inner.try_borrow_mut().ok() {
            match nx.one_mut() {
                Ok(_) => Ok(RefMut::map(nx, |nx| &mut *nx.one_mut().unwrap())),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }

    fn ind(&'a self, ind: usize) -> DynamicResult<Ref<'a, (dyn Any + Send)>> {
        if let Some(nx) = self.inner.try_borrow().ok() {
            match nx.many() {
                Ok(slice) => match slice.get(ind) {
                    Some(_) => Ok(Ref::map(nx, |nx| &*nx.many().unwrap().get(ind).unwrap())),
                    None => Err(ErrorDesc::Unit(UnitError::OutOfBounds)),
                },
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn ind_mut(&'a self, ind: usize) -> DynamicResult<RefMut<'a, (dyn Any + Send)>> {
        if let Some(mut nx) = self.inner.try_borrow_mut().ok() {
            match nx.many_mut() {
                Ok(slice) => match slice.get_mut(ind) {
                    Some(_) => Ok(RefMut::map(nx, |nx| {
                        &mut *nx.many_mut().unwrap().get_mut(ind).unwrap()
                    })),
                    None => Err(ErrorDesc::Unit(UnitError::OutOfBounds)),
                },
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }

    fn extract(&self) -> DynamicResult<Box<(dyn Any + Send)>> {
        if let Some(mut x) = self.inner.try_borrow_mut().ok() {
            match x.extract_one() {
                Ok(x) => Ok(Box::new(x)),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn extract_ind(&self, ind: usize) -> DynamicResult<Box<(dyn Any + Send)>> {
        if let Some(mut borrowed) = self.inner.try_borrow_mut().ok() {
            borrowed.many_mut().and_then(|x| {
                if ind < x.len() {
                    let x: Box<(dyn Any + Send)> = Box::new(x.remove(ind));
                    Ok(x)
                } else {
                    Err(ErrorDesc::Unit(UnitError::OutOfBounds))
                }
            })
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn extract_many(&self) -> DynamicResult<Box<(dyn Any + Send)>> {
        Ok(Box::new(
            self.inner
                .try_borrow_mut()
                .map_err(|_| ErrorDesc::BorrowedIncompatibly)?
                .extract_many_boxed(),
        ))
    }

    fn insert_any(&self, new: Box<(dyn Any + Send)>) -> Option<(Box<(dyn Any + Send)>, ErrorDesc)> {
        let newtype = new.type_id();
        if let Some(mut x) = self.inner.try_borrow_mut().ok() {
            if new.is::<T>() {
                x.insert(*new.downcast::<T>().expect(&format!(
                    "Tried to insert an object with type {:?} into a storage of type {:?}",
                    newtype,
                    TypeId::of::<T>()
                )));
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

    unsafe fn run_for(&self, (t, ptr): (TypeId, (*const (), *const ()))) -> Option<Box<dyn Any>> {
        if t == TypeId::of::<dyn for<'b> Fn(DynamicResult<&'b [T]>) -> Option<Box<dyn Any>> + 'static>(
        ) {
            if let Some(x) = self.inner.try_borrow_mut().ok() {
                let func = std::mem::transmute::<
                    _,
                    &dyn for<'b> Fn(DynamicResult<&'b [T]>) -> Option<Box<dyn Any>>,
                >(ptr);
                func(x.many())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}
