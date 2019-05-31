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
    type Owned = Box<dyn Any>;
    fn one(&'a self) -> DynamicResult<Ref<'a, dyn Any>> {
        if let Ok(nx) = self.inner.try_borrow() {
            match nx.one() {
                Ok(_) => Ok(Ref::map(nx, |nx| &*nx.one().unwrap())),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn one_mut(&'a self) -> DynamicResult<RefMut<'a, dyn Any>> {
        if let Ok(mut nx) = self.inner.try_borrow_mut() {
            match nx.one_mut() {
                Ok(_) => Ok(RefMut::map(nx, |nx| &mut *nx.one_mut().unwrap())),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }

    fn ind(&'a self, ind: usize) -> DynamicResult<Ref<'a, dyn Any>> {
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
    fn ind_mut(&'a self, ind: usize) -> DynamicResult<RefMut<'a, dyn Any>> {
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

    fn extract(&self) -> DynamicResult<Box<dyn Any>> {
        if let Ok(mut x) = self.inner.try_borrow_mut() {
            match x.extract_one() {
                Ok(x) => Ok(Box::new(x)),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn extract_ind(&self, ind: usize) -> DynamicResult<Box<dyn Any>> {
        if let Ok(mut borrowed) = self.inner.try_borrow_mut() {
            match borrowed.many_mut() {
                Ok(_) => borrowed.many_mut().and_then(|x| {
                    if ind < x.len() {
                        let x: Box<dyn Any> = Box::new(x.remove(ind));
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
    fn extract_many(&self) -> DynamicResult<Box<dyn Any>> {
        Ok(Box::new(
            self.inner
                .try_borrow_mut()
                .map_err(|_| ErrorDesc::BorrowedIncompatibly)?
                .extract_many()?,
        ))
    }

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
