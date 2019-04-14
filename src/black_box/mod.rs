use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::mem::swap;
use std::rc::Rc;
use std::collections::btree_set::BTreeSet;
use std::marker::PhantomData;
use std::cell::{RefCell, Ref, RefMut};
use std::fmt::{Debug, Formatter};

pub type DynamicResult<Ok> = Result<Ok, ErrorDesc>;

/// The basic error descriptions for why a dynamically typed resource operation didn't work. It does
/// not contain however, the description for unit-related errors which handled with a `UnitError` by
/// using the `Unit` variant of `ErrorDesc`.
#[derive(Debug, PartialEq)]
pub enum ErrorDesc {
    /// Returned if there is an incompatible borrow on the contents of the unit. It follows the same
    /// rules for runtime checking as a `RefCell<T>`. Usually bundled with a `Ref<T>`/`RefMut<T>` in
    /// a `Result<RefVariant<T>, ErrorDesc>`.
    /// ## Example:
    /// ```
    /// # use restor::*;
    /// # fn main() {
    /// let mut storage = DynamicStorage::new();
    /// storage.allocate_for::<usize>();
    /// storage.insert(0usize);
    /// let x = storage.get::<usize>().unwrap();
    /// let y = storage.get_mut::<usize>();
    /// assert!(y.is_err());
    /// # }
    /// ```
    BorrowedIncompatibly,
    /// Returned when there is no unit allocated for the type that was requested. Allocate a unit to
    /// contain a `<T>` with `DynamicStorage::allocate_for::<T>(&mut self)`. Note that `<T>` must be
    /// `T: Sized + Any + 'static`.
    /// ## Example:
    /// ```
    /// # use restor::*;
    /// # fn main() {
    /// let mut storage = DynamicStorage::new();
    /// let x = storage.get::<usize>();
    /// assert!(x.is_err());
    /// // Error, there is no unit for `usize` allocated!
    /// storage.allocate_for::<usize>();
    /// storage.insert::<usize>(10);
    /// let x = storage.get::<usize>().unwrap();
    /// assert_eq!(*x, 10);
    /// # }
    /// ```
    NoAllocatedUnit,
    /// This is an internal error that should be ignored by the user. This should never be created.
    NoMatchingType,
    /// This holds an inner `ErrorDesc`. Call `unwrap` on an `Inner` variant to get the inner error.
    Inner(Box<ErrorDesc>),
    /// Contains an error specific to unit operations. Please refer to the `UnitError` documentation
    /// for more information.
    Unit(UnitError),
}

impl ErrorDesc {
    /// Consumes the `ErrorDesc` and returns an `ErrorDesc` if it's an `Inner` variant. Panics if it
    /// is not an `ErrorDesc::Inner` variant.
    pub fn unwrap(self) -> ErrorDesc {
        if let ErrorDesc::Inner(inner) = self {
            *inner
        } else { panic!("Try to unwrap a non-inner ErrorDesc value!") }
    }
}

#[derive(Debug, PartialEq)]
pub enum UnitError {
    IsNotOne,
    IsNotMany,
    OutOfBounds,
}

pub struct NonGuardedUnit<T> {
    inner: RefCell<T>,
}

impl<T> NonGuardedUnit<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: RefCell::new(data)
        }
    }
}

pub enum StorageUnit<T: Sized + 'static> {
    Nope,
    One(T),
    Many(Vec<T>),
}

impl<T: Sized> StorageUnit<T> {
    pub fn new() -> Self {
        StorageUnit::Nope
    }

    pub fn insert(&mut self, new: T) {
        match self {
            StorageUnit::Nope => {
                *self = StorageUnit::One(new);
            }
            StorageUnit::One(_) => {
                let mut rep = StorageUnit::Many(vec![new]);
                swap(self, &mut rep);
                if let StorageUnit::One(prev) = rep {
                    if let StorageUnit::Many(v) = self {
                        v.insert(0, prev);
                    } else { unreachable!() }
                } else { unreachable!() }
            }
            StorageUnit::Many(many) => {
                many.push(new);
            }
        }
    }

    pub fn insert_many(&mut self, new: Vec<T>) {
        match self {
            StorageUnit::Nope => {
                *self = StorageUnit::Many(new.into());
            }
            StorageUnit::One(_) => {
                let mut rep = StorageUnit::Many(new.into());
                swap(&mut rep, self);
                if let StorageUnit::One(val) = rep {
                    if let StorageUnit::Many(vec) = self {
                        vec.insert(0, val);
                    } else { unreachable!() }
                } else { unreachable!() }
            }
            StorageUnit::Many(arr) => {
                arr.append(&mut new.into());
            }
        }
    }

    pub fn one(&self) -> DynamicResult<&T> {
        if let StorageUnit::One(x) = self {
            Ok(x)
        } else {
            Err(ErrorDesc::Unit(UnitError::IsNotOne))
        }
    }

    pub fn one_mut(&mut self) -> DynamicResult<&mut T> {
        if let StorageUnit::One(x) = self {
            Ok(x)
        } else {
            Err(ErrorDesc::Unit(UnitError::IsNotOne))
        }
    }

    pub fn many(&self) -> DynamicResult<&[T]> {
        if let StorageUnit::Many(x) = self {
            Ok(x)
        } else {
            Err(ErrorDesc::Unit(UnitError::IsNotMany))
        }
    }

    pub fn many_mut(&mut self) -> DynamicResult<&mut Vec<T>> {
        if let StorageUnit::Many(x) = self {
            Ok(x)
        } else {
            Err(ErrorDesc::Unit(UnitError::IsNotMany))
        }
    }

    pub fn extract_one(&mut self) -> DynamicResult<T> {
        match self {
            StorageUnit::Nope => Err(ErrorDesc::Unit(UnitError::IsNotOne)),
            StorageUnit::Many(_) => Err(ErrorDesc::Unit(UnitError::IsNotOne)),
            StorageUnit::One(_) => {
                let mut repl = StorageUnit::Nope;
                swap(&mut repl, self);
                if let StorageUnit::One(data) = repl {
                    Ok(data)
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub fn extract_many(&mut self) -> DynamicResult<Vec<T>> {
        match self {
            StorageUnit::Nope => Err(ErrorDesc::Unit(UnitError::IsNotMany)),
            StorageUnit::One(_) => Err(ErrorDesc::Unit(UnitError::IsNotMany)),
            StorageUnit::Many(_) => {
                let mut repl = StorageUnit::Nope;
                swap(&mut repl, self);
                if let StorageUnit::Many(data) = repl {
                    Ok(data)
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub fn extract_many_boxed(&mut self) -> DynamicResult<Box<[T]>> {
        match self {
            StorageUnit::Nope => Err(ErrorDesc::Unit(UnitError::IsNotMany)),
            StorageUnit::One(_) => Err(ErrorDesc::Unit(UnitError::IsNotMany)),
            StorageUnit::Many(_) => {
                let mut repl = StorageUnit::Nope;
                swap(&mut repl, self);
                if let StorageUnit::Many(data) = repl {
                    Ok(data.into_boxed_slice())
                } else {
                    unreachable!()
                }
            }
        }
    }
}

impl<T: Clone> Clone for StorageUnit<T> {
    fn clone(&self) -> Self {
        match self {
            StorageUnit::Nope => StorageUnit::Nope,
            StorageUnit::One(data) => StorageUnit::One(data.clone()),
            StorageUnit::Many(data) => StorageUnit::Many(data.clone())
        }
    }
}

pub trait Unit {
    fn one<'a>(&'a self) -> DynamicResult<Ref<'a, dyn Any>>;
    fn one_mut<'a>(&'a self) -> DynamicResult<RefMut<'a, dyn Any>>;

    fn ind<'a>(&'a self, ind: usize) -> DynamicResult<Ref<'a, dyn Any>>;
    fn ind_mut<'a>(&'a self, ind: usize) -> DynamicResult<RefMut<'a, dyn Any>>;

    fn extract(&self) -> DynamicResult<Box<dyn Any>>;
    fn extract_ind(&self, ind: usize) -> DynamicResult<Box<dyn Any>>;
    fn extract_many(&self) -> DynamicResult<Box<dyn Any>>;

    fn insert_any(&self, new: Box<dyn Any>) -> Option<(Box<dyn Any>, ErrorDesc)>;

    fn is_guarded(&self) -> bool;
    fn to_guarded(&self) -> Option<&()>;

    fn id(&self) -> TypeId;
}

impl<T: 'static> Unit for NonGuardedUnit<StorageUnit<T>> {
    fn one<'a>(&'a self) -> DynamicResult<Ref<'a, dyn Any>> {
        if let Some(nx) = self.inner.try_borrow().ok() {
            match nx.one() {
                Ok(_) => Ok(Ref::map(nx, |nx| &*nx.one().unwrap())),
                Err(e) => Err(ErrorDesc::Inner(Box::new(e)))
            }
        } else { Err(ErrorDesc::BorrowedIncompatibly) }
    }
    fn one_mut<'a>(&'a self) -> DynamicResult<RefMut<'a, dyn Any>> {
        if let Some(mut nx) = self.inner.try_borrow_mut().ok() {
            match nx.one_mut() {
                Ok(_) => Ok(RefMut::map(nx, |nx| &mut *nx.one_mut().unwrap())),
                Err(e) => Err(ErrorDesc::Inner(Box::new(e)))
            }
        } else { Err(ErrorDesc::BorrowedIncompatibly) }
    }

    fn ind<'a>(&'a self, ind: usize) -> DynamicResult<Ref<'a, dyn Any>> {
        if let Some(nx) = self.inner.try_borrow().ok() {
            match nx.many() {
                Ok(slice) => {
                    match slice.get(ind) {
                        Some(_) => Ok(Ref::map(nx, |nx| &*nx.many().unwrap().get(ind).unwrap())),
                        None => Err(ErrorDesc::Unit(UnitError::OutOfBounds))
                    }
                }
                Err(e) => Err(ErrorDesc::Inner(Box::new(e)))
            }
        } else { Err(ErrorDesc::BorrowedIncompatibly) }
    }
    fn ind_mut<'a>(&'a self, ind: usize) -> DynamicResult<RefMut<'a, dyn Any>> {
        if let Some(mut nx) = self.inner.try_borrow_mut().ok() {
            match nx.many_mut() {
                Ok(slice) => match slice.get_mut(ind) {
                    Some(_) => Ok(RefMut::map(nx, |nx| &mut *nx.many_mut().unwrap().get_mut(ind).unwrap())),
                    None => Err(ErrorDesc::Unit(UnitError::OutOfBounds))
                },
                Err(e) => Err(ErrorDesc::Inner(Box::new(e)))
            }
        } else { Err(ErrorDesc::BorrowedIncompatibly) }
    }

    fn extract(&self) -> DynamicResult<Box<dyn Any>> {
        if let Some(mut x) = self.inner.try_borrow_mut().ok() {
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
            .try_borrow_mut()
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
        Ok(Box::new(self.inner.try_borrow_mut().map_err(|_| ErrorDesc::BorrowedIncompatibly)?.extract_many_boxed()))
    }

    fn insert_any(&self, new: Box<dyn Any>) -> Option<(Box<dyn Any>, ErrorDesc)> {
        let newtype = new.type_id();
        if let Some(mut x) = self
            .inner
            .try_borrow_mut()
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

    fn is_guarded(&self) -> bool { false }
    fn to_guarded(&self) -> Option<&()> { None }

    fn id(&self) -> TypeId { TypeId::of::<T>() }
}

impl PartialEq<Self> for dyn Unit {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Debug for dyn Unit {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Unit(TypeId: {:?})", self.id())
    }
}

pub struct BlackBox {
    data: HashMap<TypeId, Box<dyn Unit>>,
}

impl BlackBox {
    pub fn new() -> Self {
        Self {
            data: HashMap::new()
        }
    }

    #[inline]
    pub fn allocate_for<T: 'static>(&mut self) {
        if !self.data.contains_key(&TypeId::of::<T>()) {
            self.data.insert(TypeId::of::<T>(), Box::new(NonGuardedUnit::new(StorageUnit::<T>::new())));
        }
    }

    pub fn insert<T: 'static>(&self, data: T) -> Option<(T, ErrorDesc)> {
        let entry = self.data.get(&TypeId::of::<T>());
        match entry {
            Some(x) => {
                match x.insert_any(Box::new(data)) {
                    Some((x, e)) => {
                        Some((*x.downcast().unwrap(), e))
                    }
                    None => None
                }
            }
            None => {
                Some((data, ErrorDesc::NoAllocatedUnit))
            }
        }
    }

    pub fn insert_many<T: 'static>(&self, data: Vec<T>) -> Option<(Vec<T>, ErrorDesc)> {
        if let Some(unit) = self.data.get(&TypeId::of::<T>()) {
            if let Some((ret, e)) = unit.insert_any(Box::new(data)) {
                Some((*ret.downcast().unwrap(), e))
            } else { None }
        } else { None }
    }

    #[inline]
    fn unit_get<T: 'static>(&self) -> DynamicResult<&dyn Unit> {
        self.data.get(&TypeId::of::<T>()).map(|x| &**x).ok_or(ErrorDesc::NoAllocatedUnit)
    }

    #[inline]
    pub fn get<T: 'static>(&self) -> DynamicResult<Ref<T>> {
        Ok(Ref::map(self.unit_get::<T>()?.one()?, |x| x.downcast_ref().unwrap()))
    }

    #[inline]
    pub fn ind<T: 'static>(&self, ind: usize) -> DynamicResult<Ref<T>> {
        Ok(Ref::map(self.unit_get::<T>()?.ind(ind)?, |x| x.downcast_ref().unwrap()))
    }

    #[inline]
    pub fn get_mut<T: 'static>(&self) -> DynamicResult<RefMut<T>> {
        Ok(RefMut::map(self.unit_get::<T>()?.one_mut()?, |x| x.downcast_mut().unwrap()))
    }

    #[inline]
    pub fn ind_mut<T: 'static>(&self, ind: usize) -> DynamicResult<RefMut<T>> {
        Ok(RefMut::map(self.unit_get::<T>()?.ind_mut(ind)?, |x| x.downcast_mut().unwrap()))
    }

    #[inline]
    pub fn extract<T: 'static>(&self) -> DynamicResult<T> {
        Ok(*self.unit_get::<T>()?.extract()?.downcast().unwrap())
    }

    #[inline]
    pub fn extract_many<T: 'static>(&self) -> DynamicResult<Box<[T]>> {
        Ok(*self.unit_get::<T>()?.extract_many()?.downcast().unwrap())
    }
}
