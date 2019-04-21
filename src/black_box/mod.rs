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

pub trait Map<I: ?Sized, O: ?Sized>: Deref<Target=I> + Sized {
    type Output: Deref<Target=O>;
    type Func: Sized + 'static;
    fn map(self, f: Self::Func) -> Self::Output;
}

impl<'a, I: 'static, O: 'static> Map<I, O> for Ref<'a, I> {
    type Output = Ref<'a, O>;
    type Func = for<'b> fn(&'b I) -> &'b O;
    fn map(self, f: Self::Func) -> Ref<'a, O> {
        Ref::map(self, f)
    }
}

impl<'a, I: 'static, O: 'static> Map<I, O> for MappedMutexGuard<'a, I> {
    type Output = MappedMutexGuard<'a, O>;
    type Func = for<'b> fn(&'b mut I) -> &'b mut O;
    fn map(self, f: Self::Func) -> MappedMutexGuard<'a, O> {
        MappedMutexGuard::map(self, f)
    }
}

impl<'a, I: 'static, O: 'static> Map<I, O> for MappedRwLockReadGuard<'a, I> {
    type Output = MappedRwLockReadGuard<'a, O>;
    type Func = for<'b> fn(&'b I) -> &'b O;
    fn map(self, f: Self::Func) -> MappedRwLockReadGuard<'a, O> {
        MappedRwLockReadGuard::map(self, f)
    }
}

pub trait MapMut<I: ?Sized, O: ?Sized>: Deref<Target=I> + Sized + DerefMut {
    type Output: Deref<Target=O> + DerefMut;
    type Func: Sized + 'static;
    fn map(self, f: Self::Func) -> Self::Output;
}

impl<'a, I: 'static, O: 'static> MapMut<I, O> for RefMut<'a, I> {
    type Output = RefMut<'a, O>;
    type Func = for<'b> fn(&'b mut I) -> &'b mut O;
    fn map(self, f: Self::Func) -> RefMut<'a, O> {
        RefMut::map(self, f)
    }
}

impl<'a, I: 'static + Sync + Send, O: 'static + Sync + Send> MapMut<I, O> for MappedRwLockWriteGuard<'a, I> {
    type Output = MappedRwLockWriteGuard<'a, O>;
    type Func = for<'b> fn(&'b mut I) -> &'b mut O;
    fn map(self, f: Self::Func) -> MappedRwLockWriteGuard<'a, O> {
        MappedRwLockWriteGuard::map(self, f)
    }
}

impl<'a, I: 'static + Sync + Send, O: 'static + Sync + Send> MapMut<I, O> for MappedMutexGuard<'a, I> {
    type Output = MappedMutexGuard<'a, O>;
    type Func = for<'b> fn(&'b mut I) -> &'b mut O;
    fn map(self, f: Self::Func) -> MappedMutexGuard<'a, O> {
        MappedMutexGuard::map(self, f)
    }
}

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

impl<
    'a,
    R: Deref<Target=dyn Any> + 'a,
    W: Deref<Target=dyn Any> + DerefMut + 'a,
    U: Unit<'a, Borrowed=R, MutBorrowed=W, Owned=Box<dyn Any>> + ?Sized,
> BlackBox<'a, R, W, Box<dyn Any>, U> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            unused: Default::default(),
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
    fn unit_get<T: 'static>(&'a self) -> DynamicResult<&'a U> {
        self.data
            .get(&TypeId::of::<T>())
            .map(|x| &**x)
            .ok_or(ErrorDesc::NoAllocatedUnit)
    }

    #[inline]
    pub fn get_mut<T: 'static>(&'a self) -> DynamicResult<<W as MapMut<dyn Any, T>>::Output>
        where W: MapMut<dyn Any, T, Func=fn(&mut dyn Any) -> &mut T> {
        Ok(W::map(self.unit_get::<T>()?.one_mut()?, |x| {
            x.downcast_mut().unwrap()
        }))
    }

    #[inline]
    pub fn ind_mut<T: 'static>(&'a self, ind: usize) -> DynamicResult<<W as MapMut<dyn Any, T>>::Output>
        where W: MapMut<dyn Any, T, Func=fn(&mut dyn Any) -> &mut T> {
        Ok(W::map(self.unit_get::<T>()?.ind_mut(ind)?, |x| {
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

    #[inline]
    pub fn get<T: 'static>(&'a self) -> DynamicResult<<R as Map<dyn Any, T>>::Output>
        where R: Map<dyn Any, T, Func=for<'b> fn(&'b dyn Any) -> &'b T> {
        Ok(R::map(self.unit_get::<T>()?.one()?, |x| {
            x.downcast_ref().unwrap()
        }))
    }
    #[inline]
    pub fn ind<T: 'static>(&'a self, ind: usize) -> DynamicResult<<R as Map<dyn Any, T>>::Output>
        where R: Map<dyn Any, T, Func=for<'b> fn(&'b dyn Any) -> &'b T> {
        Ok(R::map(self.unit_get::<T>()?.ind(ind)?, |x| {
            x.downcast_ref().unwrap()
        }))
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
    #[inline]
    pub fn allocate_for<T: 'static + Send + Sync>(&mut self) {
        if !self.data.contains_key(&TypeId::of::<T>()) {
            self.data.insert(
                TypeId::of::<T>(),
                Box::new(RwLockUnit::new(StorageUnit::<T>::new())),
            );
        }
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
    #[inline]
    pub fn allocate_for<T: 'static + Send + Sync>(&mut self) {
        if !self.data.contains_key(&TypeId::of::<T>()) {
            self.data.insert(
                TypeId::of::<T>(),
                Box::new(MutexUnit::new(StorageUnit::<T>::new())),
            );
        }
    }
}

impl<'a> BlackBox<'a, Ref<'a, dyn Any>, RefMut<'a, dyn Any>, Box<dyn Any>, RefCellUnitTrait<'a>> {
    #[inline]
    pub fn allocate_for<T: 'static>(&mut self) {
        if !self.data.contains_key(&TypeId::of::<T>()) {
            self.data.insert(
                TypeId::of::<T>(),
                Box::new(RefCellUnit::new(StorageUnit::<T>::new())),
            );
        }
    }
}
