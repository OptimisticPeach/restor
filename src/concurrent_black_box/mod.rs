use std::any::{Any, TypeId};

use super::black_box::{StorageUnit, DynamicResult, ErrorDesc, UnitError, Unit};
use std::sync::{Mutex, MutexGuard};
use std::ops::{Deref, DerefMut};

pub struct MutexUnit<T> {
    inner: Mutex<T>,
}

impl<T> MutexUnit<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: Mutex::new(data)
        }
    }
}

trait StdGuard {}

impl<'a, T> StdGuard for MutexGuard<'a, T> {}

struct MGuard<'a, T: ?Sized + 'static> {
    guard: Box<dyn StdGuard + 'a>,
    data: &'a mut T,
}

impl<'a, T: ?Sized + 'static> MGuard<'a, T> {
    fn new<H>(mut guard: MutexGuard<'a, H>, getter: impl FnOnce(&mut MutexGuard<'a, H>) -> &'a mut T) -> Self {
        let data = getter(&mut guard);
        Self { guard: Box::new(guard), data }
    }

    fn optional_new<H>(mut guard: MutexGuard<'a, H>, getter: impl FnOnce(&mut MutexGuard<'a, H>) -> Option<&'a mut T>) -> Option<Self> {
        let data = getter(&mut guard)?;
        Some(Self { guard: Box::new(guard), data })
    }

    fn result_new<E, H>(mut guard: MutexGuard<'a, H>, getter: impl FnOnce(&mut MutexGuard<'a, H>) -> Result<&'a mut T, E>) -> Result<Self, E> {
        let data = getter(&mut guard)?;
        Ok(Self { guard: Box::new(guard), data })
    }
}

impl<'a, T: ?Sized + 'static> Deref for MGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: ?Sized + 'static> DerefMut for MGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

struct RGuard<'a, T: ?Sized + 'static> {
    guard: Box<dyn StdGuard + 'a>,
    data: &'a T,
}

impl<'a, T: ?Sized + 'static> RGuard<'a, T> {
    fn new<H>(mut guard: MutexGuard<'a, H>, getter: impl FnOnce(&mut MutexGuard<'a, H>) -> &'a T) -> Self {
        let data = getter(&mut guard);
        Self { guard: Box::new(guard), data }
    }

    fn optional_new<H>(mut guard: MutexGuard<'a, H>, getter: impl FnOnce(&mut MutexGuard<'a, H>) -> Option<&'a T>) -> Option<Self> {
        let data = getter(&mut guard)?;
        Some(Self { guard: Box::new(guard), data })
    }

    fn result_new<E, H>(mut guard: MutexGuard<'a, H>, getter: impl FnOnce(&mut MutexGuard<'a, H>) -> Result<&'a T, E>) -> Result<Self, E> {
        let data = getter(&mut guard)?;
        Ok(Self { guard: Box::new(guard), data })
    }
}

impl<'a, T: ?Sized + 'static> Deref for RGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

type MutexUnitTrait<'a> = dyn Unit<'a, MGuard<'a, dyn Any>, RGuard<'a, dyn Any>>;

impl<'a, T: 'static> Unit<'a, RGuard<'a, dyn Any>, MGuard<'a, dyn Any>> for MutexUnit<StorageUnit<T>> {
    fn one(&'a self) -> DynamicResult<RGuard<'a, dyn Any>> {
        if let Some(nx) = self.inner.try_lock().ok() {
            Ok(RGuard::result_new(nx, |nx: &mut MutexGuard<StorageUnit<T>>| {
                match nx.one() {
                    Ok(_) => {
                        let reference: &dyn Any = &nx.one().unwrap();
                        Ok(reference)
                    }
                    Err(e) => Err(ErrorDesc::Inner(Box::new(e)))
                }
            })?)
        } else { Err(ErrorDesc::BorrowedIncompatibly) }
    }
    fn one_mut(&'a self) -> DynamicResult<MGuard<'a, dyn Any>> {
        if let Some(nx) = self.inner.try_lock().ok() {
            Ok(MGuard::result_new(nx, |nx: &mut MutexGuard<StorageUnit<T>>| {
                match nx.one_mut() {
                    Ok(_) => {
                        let reference: &mut dyn Any = &mut *nx.one_mut().unwrap();
                        Ok(reference)
                    }
                    Err(e) => Err(ErrorDesc::Inner(Box::new(e)))
                }
            })?)
        } else { Err(ErrorDesc::BorrowedIncompatibly) }
    }

    fn ind(&'a self, ind: usize) -> DynamicResult<RGuard<'a, dyn Any>> {
        if let Some(nx) = self.inner.try_lock().ok() {
            Ok(RGuard::result_new(nx, |nx: &mut MutexGuard<StorageUnit<T>>| {
                match nx.many() {
                    Ok(slice) => {
                        match slice.get(ind) {
                            Some(_) => {
                                let reference: &dyn Any = &*nx.many().unwrap().get(ind).unwrap();
                                Ok(reference)
                            }
                            None => Err(ErrorDesc::Unit(UnitError::OutOfBounds))
                        }
                    }
                    Err(e) => Err(ErrorDesc::Inner(Box::new(e)))
                }
            })?)
        } else { Err(ErrorDesc::BorrowedIncompatibly) }
    }
    fn ind_mut(&'a self, ind: usize) -> DynamicResult<MGuard<'a, dyn Any>> {
        if let Some(nx) = self.inner.try_lock().ok() {
            Ok(MGuard::result_new(nx, |nx: &mut MutexGuard<StorageUnit<T>>| {
                match nx.many_mut() {
                    Ok(slice) => {
                        match slice.get(ind) {
                            Some(_) => {
                                let reference: &mut dyn Any = &mut *nx.many().unwrap().get_mut(ind).unwrap();
                                Ok(reference)
                            }
                            None => Err(ErrorDesc::Unit(UnitError::OutOfBounds))
                        }
                    }
                    Err(e) => Err(ErrorDesc::Inner(Box::new(e)))
                }
            })?)
        } else { Err(ErrorDesc::BorrowedIncompatibly) }
    }

    fn extract(&self) -> DynamicResult<Box<dyn Any>> {
        if let Some(mut x) = self.inner.try_lock().ok() {
            match x.extract_one() {
                Ok(x) => Ok(Box::new(x)),
                Err(e) => Err(ErrorDesc::Inner(Box::new(e)))
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn extract_ind(&self, ind: usize) -> DynamicResult<Box<dyn Any>> {
        if let Some(mut borrowed) =
        self
            .inner
            .try_lock()
            .ok() {
            borrowed.many_mut()
                .and_then(
                    |x|
                        if ind < x.len() {
                            let x: Box<dyn Any> = Box::new(x.remove(ind));
                            Ok(x)
                        } else { Err(ErrorDesc::Unit(UnitError::OutOfBounds)) }
                )
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn extract_many(&self) -> DynamicResult<Box<dyn Any>> {
        Ok(Box::new(self.inner.try_lock().map_err(|_| ErrorDesc::BorrowedIncompatibly)?.extract_many_boxed()))
    }

    fn insert_any(&self, new: Box<dyn Any>) -> Option<(Box<dyn Any>, ErrorDesc)> {
        let newtype = new.type_id();
        if let Some(mut x) = self
            .inner
            .try_lock()
            .ok() {
            if new.is::<T>() {
                x.insert(*new.downcast::<T>().expect(
                    &format!("Tried to insert an object with type {:?} into a storage of type {:?}", newtype, TypeId::of::<T>())
                ));
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

    fn id(&self) -> TypeId { TypeId::of::<T>() }
}
