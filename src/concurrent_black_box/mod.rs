use std::any::{Any, TypeId};

use super::black_box::{
    DynamicResult,
    ErrorDesc::{self, *},
    StorageUnit, Unit,
};
mod newtype;
pub use newtype::{MutexStorage, RwLockStorage};
use parking_lot::{
    MappedMutexGuard, MappedRwLockReadGuard, MappedRwLockWriteGuard, Mutex, MutexGuard, RwLock,
    RwLockReadGuard, RwLockWriteGuard,
};

#[repr(transparent)]
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
    type Borrowed = MappedMutexGuard<'a, dyn Any>;
    type MutBorrowed = MappedMutexGuard<'a, dyn Any>;
    fn insert_any(&self, new: Box<dyn Any>) -> Option<(Box<dyn Any>, ErrorDesc)> {
        let newtype = new.type_id();
        if let Some(mut x) = self.inner.try_lock() {
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
    fn waiting_insert(&self, new: Box<dyn Any>) -> Option<(Box<dyn Any>, ErrorDesc)> {
        let newtype = new.type_id();
        if new.is::<T>() || new.is::<Vec<T>>() {
            let mut x = self.inner.lock();
            if new.is::<T>() {
                x.insert(*new.downcast::<T>().unwrap_or_else(|_| {
                    panic!(
                        "Tried to insert an object with type {:?} into a storage of type {:?}",
                        newtype,
                        TypeId::of::<T>()
                    )
                }));
                None
            } else {
                x.insert_many(*new.downcast::<Vec<T>>().unwrap());
                None
            }
        } else {
            Some((new, ErrorDesc::NoMatchingType))
        }
    }
    fn storage(&'a self) -> DynamicResult<MappedMutexGuard<'a, dyn Any>> {
        self.inner
            .try_lock()
            .map(|x| MutexGuard::map::<dyn Any, _>(x, |z| &mut *z))
            .ok_or(BorrowedIncompatibly)
    }
    fn storage_mut(&'a self) -> DynamicResult<MappedMutexGuard<'a, dyn Any>> {
        self.storage()
    }

    fn waiting_storage(&'a self) -> MappedMutexGuard<'a, dyn Any> {
        MutexGuard::map::<dyn Any, _>(self.inner.lock(), |z| &mut *z)
    }
    fn waiting_storage_mut(&'a self) -> MappedMutexGuard<'a, dyn Any> {
        self.waiting_storage()
    }

    fn id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

#[repr(transparent)]
pub struct RwLockUnit<T> {
    inner: RwLock<T>,
}

impl<T> RwLockUnit<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: RwLock::new(data),
        }
    }
    #[cfg(test)]
    pub fn inner(&self) -> &RwLock<T> {
        &self.inner
    }
}

impl<'a, T: 'static + Send> Unit<'a> for RwLockUnit<StorageUnit<T>> {
    type Borrowed = MappedRwLockReadGuard<'a, dyn Any>;
    type MutBorrowed = MappedRwLockWriteGuard<'a, dyn Any>;

    fn insert_any(&self, new: Box<dyn Any>) -> Option<(Box<dyn Any>, ErrorDesc)> {
        let newtype = new.type_id();
        if let Some(mut x) = self.inner.try_write() {
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

    fn waiting_insert(&self, new: Box<dyn Any>) -> Option<(Box<dyn Any>, ErrorDesc)> {
        let newtype = new.type_id();
        if new.is::<T>() || new.is::<Vec<T>>() {
            let mut x = self.inner.write();
            if new.is::<T>() {
                x.insert(*new.downcast::<T>().unwrap_or_else(|_| {
                    panic!(
                        "Tried to insert an object with type {:?} into a storage of type {:?}",
                        newtype,
                        TypeId::of::<T>()
                    )
                }));
                None
            } else {
                x.insert_many(*new.downcast::<Vec<T>>().unwrap());
                None
            }
        } else {
            Some((new, ErrorDesc::NoMatchingType))
        }
    }

    fn storage(&'a self) -> DynamicResult<MappedRwLockReadGuard<'a, dyn Any>> {
        self.inner
            .try_read()
            .map(|x| RwLockReadGuard::map::<dyn Any, _>(x, |z| &*z))
            .ok_or(BorrowedIncompatibly)
    }
    fn storage_mut(&'a self) -> DynamicResult<MappedRwLockWriteGuard<'a, dyn Any>> {
        self.inner
            .try_write()
            .map(|x| RwLockWriteGuard::map::<dyn Any, _>(x, |z| &mut *z))
            .ok_or(BorrowedIncompatibly)
    }

    fn waiting_storage(&'a self) -> MappedRwLockReadGuard<'a, dyn Any> {
        RwLockReadGuard::map::<dyn Any, _>(self.inner.read(), |z| &*z)
    }
    fn waiting_storage_mut(&'a self) -> MappedRwLockWriteGuard<'a, dyn Any> {
        RwLockWriteGuard::map::<dyn Any, _>(self.inner.write(), |z| &mut *z)
    }

    fn id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}
