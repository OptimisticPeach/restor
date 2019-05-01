use std::any::{Any, TypeId};

use super::black_box::{DynamicResult, ErrorDesc, StorageUnit, Unit, UnitError};
use parking_lot::{
    MappedMutexGuard, MappedRwLockReadGuard, MappedRwLockWriteGuard, Mutex, MutexGuard, RwLock,
    RwLockReadGuard, RwLockWriteGuard,
};

pub struct MutexUnit<T> {
    inner: Mutex<T>,
}

impl<T> MutexUnit<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: Mutex::new(data),
        }
    }
}

impl<'a, T: 'static + Send> Unit<'a> for MutexUnit<StorageUnit<T>> {
    type Borrowed = MappedMutexGuard<'a, (dyn Any + Send)>;
    type MutBorrowed = MappedMutexGuard<'a, (dyn Any + Send)>;
    type Owned = Box<(dyn Any + Send)>;
    fn one(&'a self) -> DynamicResult<MappedMutexGuard<'a, (dyn Any + Send)>> {
        if let Some(mut nx) = self.inner.try_lock() {
            match nx.one_mut() {
                Ok(_) => Ok(MutexGuard::map(nx, |x| {
                    let r: &mut (dyn Any + Send) = &mut *x.one_mut().unwrap();
                    r
                })),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn one_mut(&'a self) -> DynamicResult<MappedMutexGuard<'a, (dyn Any + Send)>> {
        if let Some(mut nx) = self.inner.try_lock() {
            match nx.one_mut() {
                Ok(_) => Ok(MutexGuard::map(nx, |x| {
                    let r: &mut (dyn Any + Send) = &mut *x.one_mut().unwrap();
                    r
                })),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }

    fn ind(&'a self, ind: usize) -> DynamicResult<MappedMutexGuard<'a, (dyn Any + Send)>> {
        if let Some(mut nx) = self.inner.try_lock() {
            match nx.many_mut() {
                Ok(slice) => match slice.get_mut(ind) {
                    Some(_) => Ok(MutexGuard::map(nx, |x| {
                        let r: &mut (dyn Any + Send) = &mut x.many_mut().unwrap()[ind];
                        r
                    })),
                    None => Err(ErrorDesc::Unit(UnitError::OutOfBounds)),
                },
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn ind_mut(&'a self, ind: usize) -> DynamicResult<MappedMutexGuard<'a, (dyn Any + Send)>> {
        if let Some(mut nx) = self.inner.try_lock() {
            match nx.many_mut() {
                Ok(slice) => match slice.get_mut(ind) {
                    Some(_) => Ok(MutexGuard::map(nx, |x| {
                        let r: &mut (dyn Any + Send) = &mut x.many_mut().unwrap()[ind];
                        r
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
        if let Some(mut x) = self.inner.try_lock() {
            match x.extract_one() {
                Ok(x) => Ok(Box::new(x)),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn extract_ind(&self, ind: usize) -> DynamicResult<Box<(dyn Any + Send)>> {
        if let Some(mut borrowed) = self.inner.try_lock() {
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
                .try_lock()
                .ok_or(ErrorDesc::BorrowedIncompatibly)?
                .extract_many_boxed(),
        ))
    }

    fn insert_any(&self, new: Box<(dyn Any + Send)>) -> Option<(Box<(dyn Any + Send)>, ErrorDesc)> {
        let newtype = new.type_id();
        if let Some(mut x) = self.inner.try_lock() {
            if new.is::<T>() {
                x.insert(*new.downcast::<T>().expect(&format!(
                    "Tried to insert an object with type {:?} into a storage of type {:?}",
                    newtype,
                    TypeId::of::<T>()
                )));
                None
            } else if new.is::<Box<Vec<T>>>() {
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
            if let Some(x) = self.inner.try_lock() {
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

unsafe impl<T: Send> Send for MutexUnit<StorageUnit<T>> {}

pub struct RwLockUnit<T> {
    inner: RwLock<T>,
}

impl<T> RwLockUnit<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: RwLock::new(data),
        }
    }
}

impl<'a, T: 'static + Send> Unit<'a> for RwLockUnit<StorageUnit<T>> {
    type Borrowed = MappedRwLockReadGuard<'a, (dyn Any + Send)>;
    type MutBorrowed = MappedRwLockWriteGuard<'a, (dyn Any + Send)>;
    type Owned = Box<(dyn Any + Send)>;
    fn one(&'a self) -> DynamicResult<MappedRwLockReadGuard<'a, (dyn Any + Send)>> {
        if let Some(nx) = self.inner.try_read() {
            match nx.one() {
                Ok(_) => Ok(RwLockReadGuard::map(nx, |x| {
                    let r: &(dyn Any + Send) = &*x.one().unwrap();
                    r
                })),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn one_mut(&'a self) -> DynamicResult<MappedRwLockWriteGuard<'a, (dyn Any + Send)>> {
        if let Some(mut nx) = self.inner.try_write() {
            match nx.one_mut() {
                Ok(_) => Ok(RwLockWriteGuard::map(nx, |x| {
                    let r: &mut (dyn Any + Send) = &mut *x.one_mut().unwrap();
                    r
                })),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }

    fn ind(&'a self, ind: usize) -> DynamicResult<MappedRwLockReadGuard<'a, (dyn Any + Send)>> {
        if let Some(nx) = self.inner.try_read() {
            match nx.many() {
                Ok(slice) => match slice.get(ind) {
                    Some(_) => Ok(RwLockReadGuard::map(nx, |x| {
                        let r: &(dyn Any + Send) = &x.many().unwrap()[ind];
                        r
                    })),
                    None => Err(ErrorDesc::Unit(UnitError::OutOfBounds)),
                },
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn ind_mut(
        &'a self,
        ind: usize,
    ) -> DynamicResult<MappedRwLockWriteGuard<'a, (dyn Any + Send)>> {
        if let Some(nx) = self.inner.try_write() {
            match nx.many() {
                Ok(slice) => match slice.get(ind) {
                    Some(_) => Ok(RwLockWriteGuard::map(nx, |x| {
                        let r: &mut (dyn Any + Send) = &mut x.many_mut().unwrap()[ind];
                        r
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
        if let Some(mut x) = self.inner.try_write() {
            match x.extract_one() {
                Ok(x) => Ok(Box::new(x)),
                Err(e) => Err(e),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn extract_ind(&self, ind: usize) -> DynamicResult<Box<(dyn Any + Send)>> {
        if let Some(mut borrowed) = self.inner.try_write() {
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
                .try_write()
                .ok_or(ErrorDesc::BorrowedIncompatibly)?
                .extract_many_boxed(),
        ))
    }

    fn insert_any(&self, new: Box<(dyn Any + Send)>) -> Option<(Box<(dyn Any + Send)>, ErrorDesc)> {
        let newtype = new.type_id();
        if let Some(mut x) = self.inner.try_write() {
            if new.is::<T>() {
                x.insert(*new.downcast::<T>().expect(&format!(
                    "Tried to insert an object with type {:?} into a storage of type {:?}",
                    newtype,
                    TypeId::of::<T>()
                )));
                None
            } else if new.is::<Box<Vec<T>>>() {
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
        if t == TypeId::of::<
            (dyn for<'b> Fn(DynamicResult<&'b [T]>) -> Option<Box<dyn Any>> + 'static),
        >() {
            if let Some(x) = self.inner.try_read() {
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

unsafe impl<T: Send> Send for RwLockUnit<StorageUnit<T>> {}
