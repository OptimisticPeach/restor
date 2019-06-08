use std::any::{Any, TypeId};

use super::black_box::{
    DynamicResult,
    ErrorDesc::{self, *},
    StorageUnit, Unit,
};
use crate::BlackBox;
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
    fn storage(&'a self) -> DynamicResult<MappedMutexGuard<'a, dyn Any>> {
        self.inner
            .try_lock()
            .map(|x| MutexGuard::map::<dyn Any, _>(x, |z| &mut *z))
            .ok_or(BorrowedIncompatibly)
    }
    fn storage_mut(&'a self) -> DynamicResult<MappedMutexGuard<'a, dyn Any>> {
        self.storage()
    }

    unsafe fn run_for(
        &self,
        (t, ptr): (TypeId, (*const (), *const ())),
    ) -> DynamicResult<Box<dyn Any>> {
        if t == TypeId::of::<dyn FnMut(DynamicResult<&[T]>) -> Box<dyn Any>>() {
            if let Some(x) = self.inner.try_lock() {
                let func = std::mem::transmute::<
                    _,
                    &mut dyn FnMut(DynamicResult<&[T]>) -> Box<dyn Any>,
                >(ptr);
                Ok(func(x.many()))
            } else {
                Err(BorrowedIncompatibly)
            }
        } else if t == TypeId::of::<dyn FnMut(DynamicResult<&mut Vec<T>>) -> Box<dyn Any>>() {
            if let Some(mut x) = self.inner.try_lock() {
                let func = std::mem::transmute::<
                    _,
                    &mut dyn FnMut(DynamicResult<&mut Vec<T>>) -> Box<dyn Any>,
                >(ptr);
                let res = func(x.many_mut());
                x.rearrange_if_necessary();
                Ok(res)
            } else {
                Err(BorrowedIncompatibly)
            }
        } else {
            panic!("Wrong function type passed to `run_for`!");
        }
    }

    fn id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}

unsafe impl<T: Send> Send for MutexUnit<StorageUnit<T>> {}
unsafe impl<T: Send> Sync for MutexUnit<StorageUnit<T>> {}

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
    type Borrowed = MappedRwLockReadGuard<'a, dyn Any>;
    type MutBorrowed = MappedRwLockWriteGuard<'a, dyn Any>;
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
    unsafe fn run_for(
        &self,
        (t, ptr): (TypeId, (*const (), *const ())),
    ) -> DynamicResult<Box<dyn Any>> {
        if t == TypeId::of::<(dyn FnMut(DynamicResult<&[T]>) -> Box<dyn Any>)>() {
            if let Some(x) = self.inner.try_read() {
                let func = std::mem::transmute::<
                    _,
                    &mut dyn FnMut(DynamicResult<&[T]>) -> Box<dyn Any>,
                >(ptr);
                Ok(func(x.many()))
            } else {
                Err(ErrorDesc::BorrowedIncompatibly)
            }
        } else if t == TypeId::of::<(dyn FnMut(DynamicResult<&mut Vec<T>>) -> Box<dyn Any>)>() {
            if let Some(mut x) = self.inner.try_write() {
                let func = std::mem::transmute::<
                    _,
                    &mut dyn FnMut(DynamicResult<&mut Vec<T>>) -> Box<dyn Any>,
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

unsafe impl<T: Send> Send for RwLockUnit<StorageUnit<T>> {}

type RwLockBlackBox = BlackBox<
    (dyn for<'a> Unit<
        'a,
        Borrowed = MappedRwLockReadGuard<'a, dyn Any>,
        MutBorrowed = MappedRwLockWriteGuard<'a, dyn Any>,
    > + Send
         + Sync),
>;

///
/// A wrapper for a `RwLock`-safe `BlackBox` that is `Send` + `Sync`!
///
/// > ## Please note!
/// > The documentation for the functions implemented for this type are
/// > found in [`BlackBox`]'s documentation under the same name.
///
/// This only allows for allocation of `T: Send + Sync + Any` units,
/// making it safe to `impl Sync for RwLockStorage`.
///
/// This is done by `Deref`ing into the appropriate `BlackBox`, but not
/// `DerefMut`ing into it. This prevents the user from using the
/// `allocate_for` function from `BlackBox` and instead forces them to
/// use `RwLockStorage`'s `allocate_for` function, which satisfies the
/// above requirements.
///
/// This fits into the same context as any storage type provided by
/// `restor`.
///
pub struct RwLockStorage {
    black_box: RwLockBlackBox,
}

crate::impl_unit!(
    RwLockStorage,
    (dyn Any + Send + Sync),
    (Send + Sync + Any),
    RwLockUnit,
    MappedRwLockWriteGuard,
    MappedRwLockReadGuard,
    black_box,
    add_unmut
);

impl RwLockStorage {
    pub fn new() -> Self {
        RwLockStorage {
            black_box: RwLockBlackBox::new(),
        }
    }
}

impl Default for RwLockStorage {
    fn default() -> Self {
        Self::new()
    }
}

type MutexBlackBox = BlackBox<
    (dyn for<'a> Unit<
        'a,
        Borrowed = MappedMutexGuard<'a, dyn Any>,
        MutBorrowed = MappedMutexGuard<'a, dyn Any>,
    > + Send
         + Sync),
>;

///
/// The storage with interior mutability based on [`Mutex`]es.
/// This allows the data that is put in to only need to be `T: Send`,
/// because this only allows one thread to read or write to the data.
///
/// Because of the above, this also only has the mutable versions
/// of functions implemented, as though a [`MutexGuard`] can only
/// contain a mutable reference due to the nature of the mutex.
///
/// > ## Please note!
/// > The documentation for the functions implemented for this type are
/// > found in [`BlackBox`]'s documentation under the same name.
///
/// This can be used in any context a `DynamicStorage` can be used
/// with the exception of the uses of non-mut functions
/// Please refer to the [`make_storage`](../macro.make_storage.html)
/// macro to create these with a shorthand.
///
/// [`Mutex`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.Mutex.html
/// [`MutexGuard`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.MappedMutexGuard.html
///
/// ----
/// ## Implementation note
///
/// A wrapper for a `Mutex`-safe `BlackBox`.
///
/// This only allows for allocation of `T: Send + Any` units,
/// making it safe to `impl Sync for MutexStorage`.
///
/// This is done by `Deref`ing into the appropriate `BlackBox`, but not
/// `DerefMut`ing into it. This prevents the user from using the
/// `allocate_for` function from `BlackBox` and instead forces them to
/// use `MutexStorage`'s `allocate_for` function, which satisfies the
/// above requirements.
///
/// This fits into the same context as any storage type provided by
/// `restor`.
///
pub struct MutexStorage {
    black_box: MutexBlackBox,
}

crate::impl_unit!(
    MutexStorage,
    (dyn Any + Send),
    (Send + Any),
    MutexUnit,
    MappedMutexGuard,
    MappedMutexGuard,
    black_box
);

impl MutexStorage {
    pub fn new() -> Self {
        MutexStorage {
            black_box: BlackBox::new(),
        }
    }
}

impl Default for MutexStorage {
    fn default() -> Self {
        Self::new()
    }
}
