use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};

use super::*;
use crate::black_box::unit::ErrorDesc::BorrowedIncompatibly;

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
        if let Ok(nx) = self.inner.try_borrow() {
            match nx.one() {
                Ok(_) => Ok(Ref::map(nx, |nx| &*nx.one().unwrap())),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn one_mut(&'a self) -> DynamicResult<RefMut<'a, (dyn Any + Send)>> {
        if let Ok(mut nx) = self.inner.try_borrow_mut() {
            match nx.one_mut() {
                Ok(_) => Ok(RefMut::map(nx, |nx| &mut *nx.one_mut().unwrap())),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }

    fn ind(&'a self, ind: usize) -> DynamicResult<Ref<'a, (dyn Any + Send)>> {
        if let Ok(nx) = self.inner.try_borrow() {
            match nx.many() {
                Ok(slice) => match slice.get(ind) {
                    Some(_) => Ok(Ref::map(nx, |nx| &*nx.many().unwrap().get(ind).unwrap())),
                    None => Err(ErrorDesc::Unit(UnitError::OutOfBounds)),
                },
                Err(many_err) => {
                    if ind == 0 {
                        match nx.one() {
                            Ok(_) => Ok(Ref::map(nx, |nx| &*nx.one().unwrap())),
                            Err(one_err) => Err(one_err & many_err),
                        }
                    } else {
                        Err(many_err)
                    }
                }
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn ind_mut(&'a self, ind: usize) -> DynamicResult<RefMut<'a, (dyn Any + Send)>> {
        if let Ok(mut nx) = self.inner.try_borrow_mut() {
            match nx.many_mut() {
                Ok(slice) => match slice.get_mut(ind) {
                    Some(_) => Ok(RefMut::map(nx, |nx| {
                        &mut *nx.many_mut().unwrap().get_mut(ind).unwrap()
                    })),
                    None => Err(ErrorDesc::Unit(UnitError::OutOfBounds)),
                },
                Err(many_err) => {
                    if ind == 0 {
                        match nx.one_mut() {
                            Ok(_) => Ok(RefMut::map(nx, |nx| &mut *nx.one_mut().unwrap())),
                            Err(one_err) => Err(one_err & many_err),
                        }
                    } else {
                        Err(many_err)
                    }
                }
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }

    fn extract(&self) -> DynamicResult<Box<(dyn Any + Send)>> {
        if let Ok(mut x) = self.inner.try_borrow_mut() {
            match x.extract_one() {
                Ok(x) => Ok(Box::new(x)),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn extract_ind(&self, ind: usize) -> DynamicResult<Box<(dyn Any + Send)>> {
        if let Ok(mut borrowed) = self.inner.try_borrow_mut() {
            match borrowed.many_mut() {
                Ok(_) => borrowed.many_mut().and_then(|x| {
                    if ind < x.len() {
                        let x: Box<(dyn Any + Send)> = Box::new(x.remove(ind));
                        Ok(x)
                    } else {
                        Err(ErrorDesc::Unit(UnitError::OutOfBounds))
                    }
                }),
                Err(e) => {
                    if ind == 0 {
                        borrowed
                            .extract_one()
                            .map(|x| Box::new(x) as _)
                            .map_err(|ne| ne & e)
                    } else {
                        Err(e)
                    }
                }
            }
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

    fn storage(&'a self) -> DynamicResult<Ref<'a, (dyn Any + Send)>> {
        self.inner
            .try_borrow()
            .ok()
            .map(|x| Ref::map::<(dyn Any + Send), _>(x, |z| &*z))
            .ok_or(BorrowedIncompatibly)
    }
    fn storage_mut(&'a self) -> DynamicResult<RefMut<'a, (dyn Any + Send)>> {
        self.inner
            .try_borrow_mut()
            .ok()
            .map(|x| RefMut::map::<(dyn Any + Send), _>(x, |z| &mut *z))
            .ok_or(BorrowedIncompatibly)
    }

    unsafe fn run_for(&self, (t, ptr): (TypeId, (*const (), *const ()))) -> Option<Box<dyn Any>> {
        if t == TypeId::of::<dyn Fn(DynamicResult<&[T]>) -> Option<Box<dyn Any>> + 'static>() {
            if let Ok(x) = self.inner.try_borrow_mut() {
                let func = std::mem::transmute::<
                    _,
                    &dyn Fn(DynamicResult<&[T]>) -> Option<Box<dyn Any>>,
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
