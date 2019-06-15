use parking_lot::{MappedMutexGuard, MappedRwLockReadGuard, MappedRwLockWriteGuard};
use std::cell::{Ref, RefMut};
use std::ops::{Deref, DerefMut};

///
/// A trait forcing the implementor to implement a `map` function
/// this is used to genericize over `MappedMutexGuard`,
/// `MappedRwLockReadGuard` and `Ref`
///
pub trait Map<I: ?Sized, O: ?Sized>: Deref<Target = I> + Sized {
    type Output: Deref<Target = O>;
    type Func: ?Sized + 'static;
    fn map(self, f: &Self::Func) -> Self::Output;
}

impl<'a, I: 'static + ?Sized, O: 'static + ?Sized> Map<I, O> for Ref<'a, I> {
    type Output = Ref<'a, O>;
    type Func = dyn for<'b> Fn(&'b I) -> &'b O;
    fn map(self, f: &Self::Func) -> Ref<'a, O> {
        Ref::map(self, f)
    }
}

impl<'a, I: 'static + ?Sized, O: 'static + ?Sized> Map<I, O> for MappedRwLockReadGuard<'a, I> {
    type Output = MappedRwLockReadGuard<'a, O>;
    type Func = dyn for<'b> Fn(&'b I) -> &'b O;
    fn map(self, f: &Self::Func) -> MappedRwLockReadGuard<'a, O> {
        MappedRwLockReadGuard::map(self, f)
    }
}

///
/// A trait forcing the implementor to implement a `map` method
/// this is used to genericize over `MappedMutexGuard` and
/// `MappedRwLockWriteGuard` and `RefMut`
///
pub trait MapMut<I: ?Sized, O: ?Sized>: Deref<Target = I> + Sized + DerefMut {
    type Output: Deref<Target = O> + DerefMut;
    type Func: ?Sized + 'static;
    fn map(self, f: &Self::Func) -> Self::Output;
}

impl<'a, I: 'static + ?Sized, O: 'static + ?Sized> MapMut<I, O> for RefMut<'a, I> {
    type Output = RefMut<'a, O>;
    type Func = dyn for<'b> Fn(&'b mut I) -> &'b mut O;
    fn map(self, f: &Self::Func) -> RefMut<'a, O> {
        RefMut::map(self, f)
    }
}

impl<'a, I: 'static + ?Sized, O: 'static + ?Sized> MapMut<I, O> for MappedRwLockWriteGuard<'a, I> {
    type Output = MappedRwLockWriteGuard<'a, O>;
    type Func = dyn for<'b> Fn(&'b mut I) -> &'b mut O;
    fn map(self, f: &Self::Func) -> MappedRwLockWriteGuard<'a, O> {
        MappedRwLockWriteGuard::map(self, f)
    }
}

impl<'a, I: 'static + ?Sized, O: 'static + ?Sized> MapMut<I, O> for MappedMutexGuard<'a, I> {
    type Output = MappedMutexGuard<'a, O>;
    type Func = dyn for<'b> Fn(&'b mut I) -> &'b mut O;
    fn map(self, f: &Self::Func) -> MappedMutexGuard<'a, O> {
        MappedMutexGuard::map(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::{Map, MapMut};
    use crate::{make_storage, RwLockStorage};
    use lazy_static::*;
    lazy_static! {
        static ref storage: RwLockStorage = make_storage!(RwLockStorage: UnMut, Mut);
    }
    #[derive(Debug)]
    struct UnMut(pub String);

    #[derive(Debug)]
    struct Mut(pub String);

    #[test]
    fn test_map() {
        storage.insert(UnMut("Abc".into())).unwrap();
        let guard = storage.get::<&UnMut>().unwrap();
        assert_eq!(guard.0, "Abc");
        let guard = guard.map(&|x| &x.0[1..]);
        assert_eq!(&*guard, "bc");
    }

    #[test]
    fn test_map_mut() {
        storage.insert(Mut("Abc".into())).unwrap();
        let mut guard = storage.get::<&mut Mut>().unwrap();
        assert_eq!(guard.0, "Abc");
        guard.0.push_str("def");
        let guard = guard.map(&|x| &mut x.0[2..]);
        assert_eq!(&*guard, "cdef");
    }
}
