use parking_lot::{MappedMutexGuard, MappedRwLockReadGuard, MappedRwLockWriteGuard};
use std::any::{Any, TypeId};
use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

mod unit;

pub use crate::black_box::unit::{DynamicResult, ErrorDesc, StorageUnit, Unit, UnitError};
use crate::concurrent_black_box::{MutexUnit, RwLockUnit};

mod refcell_unit;

pub use crate::black_box::refcell_unit::*;
use std::marker::PhantomData;

pub type RefCellUnitTrait<'a> = dyn Unit<
    'a,
    Borrowed=Ref<'a, dyn Any>,
    MutBorrowed=RefMut<'a, dyn Any>,
    Owned=Box<dyn Any>,
>;
pub type MutexUnitTrait<'a> = dyn Unit<
    'a,
    Borrowed=MappedMutexGuard<'a, dyn Any>,
    MutBorrowed=MappedMutexGuard<'a, dyn Any>,
    Owned=Box<dyn Any>,
>;
pub type RwLockUnitTrait<'a> = dyn Unit<
    'a,
    Borrowed=MappedRwLockReadGuard<'a, dyn Any>,
    MutBorrowed=MappedRwLockWriteGuard<'a, dyn Any>,
    Owned=Box<dyn Any>,
>;

pub struct BlackBox<
    'a,
    R: Deref<Target=dyn Any> + 'a,
    W: Deref<Target=dyn Any> + DerefMut + 'a,
    O: Deref<Target=dyn Any> + DerefMut,
    U: Unit<'a, Borrowed=R, MutBorrowed=W, Owned=O> + ?Sized,
> {
    data: HashMap<TypeId, Box<U>>,
    unused: PhantomData<&'a ()>,
}

impl<'a> BlackBox<'a, Ref<'a, dyn Any>, RefMut<'a, dyn Any>, Box<dyn Any>, RefCellUnitTrait<'a>> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            unused: Default::default(),
        }
    }

    #[inline]
    pub fn allocate_for<T: 'static>(&mut self) {
        if !self.data.contains_key(&TypeId::of::<T>()) {
            self.data.insert(
                TypeId::of::<T>(),
                Box::new(RefCellUnit::new(StorageUnit::<T>::new())),
            );
        }
    }

    pub fn insert<T: 'static>(&self, data: T) -> Option<(T, ErrorDesc)> {
        let entry = self.data.get(&TypeId::of::<T>());
        match entry {
            Some(x) => match x.insert_any(Box::new(data)) {
                Some((x, e)) => Some((*x.downcast().unwrap(), e)),
                None => None,
            },
            None => Some((data, ErrorDesc::NoAllocatedUnit)),
        }
    }

    pub fn insert_many<T: 'static>(&self, data: Vec<T>) -> Option<(Vec<T>, ErrorDesc)> {
        if let Some(unit) = self.data.get(&TypeId::of::<T>()) {
            if let Some((ret, e)) = unit.insert_any(Box::new(data)) {
                Some((*ret.downcast().unwrap(), e))
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    fn unit_get<T: 'static>(&'a self) -> DynamicResult<&'a RefCellUnitTrait<'a>> {
        self.data
            .get(&TypeId::of::<T>())
            .map(|x| &**x)
            .ok_or(ErrorDesc::NoAllocatedUnit)
    }

    #[inline]
    pub fn get<T: 'static>(&'a self) -> DynamicResult<Ref<'a, T>> {
        Ok(Ref::map(self.unit_get::<T>()?.one()?, |x| {
            x.downcast_ref().unwrap()
        }))
    }

    #[inline]
    pub fn ind<T: 'static>(&'a self, ind: usize) -> DynamicResult<Ref<'a, T>> {
        Ok(Ref::map(self.unit_get::<T>()?.ind(ind)?, |x| {
            x.downcast_ref().unwrap()
        }))
    }

    #[inline]
    pub fn get_mut<T: 'static>(&'a self) -> DynamicResult<RefMut<'a, T>> {
        Ok(RefMut::map(self.unit_get::<T>()?.one_mut()?, |x| {
            x.downcast_mut().unwrap()
        }))
    }

    #[inline]
    pub fn ind_mut<T: 'static>(&'a self, ind: usize) -> DynamicResult<RefMut<'a, T>> {
        Ok(RefMut::map(self.unit_get::<T>()?.ind_mut(ind)?, |x| {
            x.downcast_mut().unwrap()
        }))
    }

    #[inline]
    pub fn extract<T: 'static>(&'a self) -> DynamicResult<T> {
        Ok(*self.unit_get::<T>()?.extract()?.downcast().unwrap())
    }

    #[inline]
    pub fn extract_many<T: 'static>(&'a self) -> DynamicResult<Box<[T]>> {
        Ok(*self.unit_get::<T>()?.extract_many()?.downcast().unwrap())
    }
}

impl<'a>
BlackBox<
    'a,
    MappedRwLockReadGuard<'a, dyn Any>,
    MappedRwLockWriteGuard<'a, dyn Any>,
    Box<dyn Any>,
    RwLockUnitTrait<'a>,
>
{
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            unused: Default::default(),
        }
    }

    #[inline]
    pub fn allocate_for<T: 'static + Send + Sync>(&mut self) {
        if !self.data.contains_key(&TypeId::of::<T>()) {
            self.data.insert(
                TypeId::of::<T>(),
                Box::new(RwLockUnit::new(StorageUnit::<T>::new())),
            );
        }
    }

    pub fn insert<T: 'static + Send + Sync>(&self, data: T) -> Option<(T, ErrorDesc)> {
        let entry = self.data.get(&TypeId::of::<T>());
        match entry {
            Some(x) => match x.insert_any(Box::new(data)) {
                Some((x, e)) => Some((*x.downcast().unwrap(), e)),
                None => None,
            },
            None => Some((data, ErrorDesc::NoAllocatedUnit)),
        }
    }

    pub fn insert_many<T: 'static + Send + Sync>(
        &self,
        data: Vec<T>,
    ) -> Option<(Vec<T>, ErrorDesc)> {
        if let Some(unit) = self.data.get(&TypeId::of::<T>()) {
            if let Some((ret, e)) = unit.insert_any(Box::new(data)) {
                Some((*ret.downcast().unwrap(), e))
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    fn unit_get<T: 'static + Send + Sync>(&'a self) -> DynamicResult<&'a RwLockUnitTrait<'a>> {
        self.data
            .get(&TypeId::of::<T>())
            .map(|x| &**x)
            .ok_or(ErrorDesc::NoAllocatedUnit)
    }

    #[inline]
    pub fn get<T: 'static + Send + Sync>(&'a self) -> DynamicResult<MappedRwLockReadGuard<'a, T>> {
        Ok(MappedRwLockReadGuard::map(
            self.unit_get::<T>()?.one()?,
            |x| x.downcast_ref().unwrap(),
        ))
    }

    #[inline]
    pub fn ind<T: 'static + Send + Sync>(
        &'a self,
        ind: usize,
    ) -> DynamicResult<MappedRwLockReadGuard<'a, T>> {
        Ok(MappedRwLockReadGuard::map(
            self.unit_get::<T>()?.ind(ind)?,
            |x| x.downcast_ref().unwrap(),
        ))
    }

    #[inline]
    pub fn get_mut<T: 'static + Send + Sync>(
        &'a self,
    ) -> DynamicResult<MappedRwLockWriteGuard<'a, T>> {
        Ok(MappedRwLockWriteGuard::map(
            self.unit_get::<T>()?.one_mut()?,
            |x| x.downcast_mut().unwrap(),
        ))
    }

    #[inline]
    pub fn ind_mut<T: 'static + Send + Sync>(
        &'a self,
        ind: usize,
    ) -> DynamicResult<MappedRwLockWriteGuard<'a, T>> {
        Ok(MappedRwLockWriteGuard::map(
            self.unit_get::<T>()?.ind_mut(ind)?,
            |x| x.downcast_mut().unwrap(),
        ))
    }

    #[inline]
    pub fn extract<T: 'static + Send + Sync>(&'a self) -> DynamicResult<T> {
        Ok(*self.unit_get::<T>()?.extract()?.downcast().unwrap())
    }

    #[inline]
    pub fn extract_many<T: 'static + Send + Sync>(&'a self) -> DynamicResult<Box<[T]>> {
        Ok(*self.unit_get::<T>()?.extract_many()?.downcast().unwrap())
    }
}

impl<'a>
BlackBox<
    'a,
    MappedMutexGuard<'a, dyn Any>,
    MappedMutexGuard<'a, dyn Any>,
    Box<dyn Any>,
    MutexUnitTrait<'a>,
>
{
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            unused: Default::default(),
        }
    }

    #[inline]
    pub fn allocate_for<T: 'static + Send + Sync>(&mut self) {
        if !self.data.contains_key(&TypeId::of::<T>()) {
            self.data.insert(
                TypeId::of::<T>(),
                Box::new(MutexUnit::new(StorageUnit::<T>::new())),
            );
        }
    }

    pub fn insert<T: 'static + Send + Sync>(&self, data: T) -> Option<(T, ErrorDesc)> {
        let entry = self.data.get(&TypeId::of::<T>());
        match entry {
            Some(x) => match x.insert_any(Box::new(data)) {
                Some((x, e)) => Some((*x.downcast().unwrap(), e)),
                None => None,
            },
            None => Some((data, ErrorDesc::NoAllocatedUnit)),
        }
    }

    pub fn insert_many<T: 'static + Send + Sync>(
        &self,
        data: Vec<T>,
    ) -> Option<(Vec<T>, ErrorDesc)> {
        if let Some(unit) = self.data.get(&TypeId::of::<T>()) {
            if let Some((ret, e)) = unit.insert_any(Box::new(data)) {
                Some((*ret.downcast().unwrap(), e))
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    fn unit_get<T: 'static + Send + Sync>(&'a self) -> DynamicResult<&'a MutexUnitTrait<'a>> {
        self.data
            .get(&TypeId::of::<T>())
            .map(|x| &**x)
            .ok_or(ErrorDesc::NoAllocatedUnit)
    }

    #[inline]
    pub fn get<T: 'static + Send + Sync>(&'a self) -> DynamicResult<MappedMutexGuard<'a, T>> {
        Ok(MappedMutexGuard::map(self.unit_get::<T>()?.one()?, |x| {
            x.downcast_mut().unwrap()
        }))
    }

    #[inline]
    pub fn ind<T: 'static + Send + Sync>(
        &'a self,
        ind: usize,
    ) -> DynamicResult<MappedMutexGuard<'a, T>> {
        Ok(MappedMutexGuard::map(
            self.unit_get::<T>()?.ind(ind)?,
            |x| x.downcast_mut().unwrap(),
        ))
    }

    #[inline]
    pub fn get_mut<T: 'static + Send + Sync>(&'a self) -> DynamicResult<MappedMutexGuard<'a, T>> {
        Ok(MappedMutexGuard::map(
            self.unit_get::<T>()?.one_mut()?,
            |x| x.downcast_mut().unwrap(),
        ))
    }

    #[inline]
    pub fn ind_mut<T: 'static + Send + Sync>(
        &'a self,
        ind: usize,
    ) -> DynamicResult<MappedMutexGuard<'a, T>> {
        Ok(MappedMutexGuard::map(
            self.unit_get::<T>()?.ind_mut(ind)?,
            |x| x.downcast_mut().unwrap(),
        ))
    }

    #[inline]
    pub fn extract<T: 'static + Send + Sync>(&'a self) -> DynamicResult<T> {
        Ok(*self.unit_get::<T>()?.extract()?.downcast().unwrap())
    }

    #[inline]
    pub fn extract_many<T: 'static + Send + Sync>(&'a self) -> DynamicResult<Box<[T]>> {
        Ok(*self.unit_get::<T>()?.extract_many()?.downcast().unwrap())
    }
}
