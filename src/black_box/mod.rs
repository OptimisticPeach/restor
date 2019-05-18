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

pub type RefCellUnitTrait = dyn for<'a> Unit<
    'a,
    Borrowed = Ref<'a, (dyn Any + Send)>,
    MutBorrowed = RefMut<'a, (dyn Any + Send)>,
    Owned = Box<(dyn Any + Send)>,
>;
pub type MutexUnitTrait = dyn for<'a> Unit<
    'a,
    Borrowed = MappedMutexGuard<'a, (dyn Any + Send)>,
    MutBorrowed = MappedMutexGuard<'a, (dyn Any + Send)>,
    Owned = Box<(dyn Any + Send)>,
>;
pub type RwLockUnitTrait = for<'a> Unit<
    'a,
    Borrowed = MappedRwLockReadGuard<'a, (dyn Any + Send)>,
    MutBorrowed = MappedRwLockWriteGuard<'a, (dyn Any + Send)>,
    Owned = Box<(dyn Any + Send)>,
>;

/// A trait forcing the implementor to implement a `map` function
/// this is used to genericize over `MappedMutexGuard`,
/// `MappedRwLockReadGuard` and `Ref`
pub trait Map<I: ?Sized, O: ?Sized>: Deref<Target = I> + Sized {
    type Output: Deref<Target = O>;
    type Func: Sized + 'static;
    fn map(self, f: Self::Func) -> Self::Output;
}

impl<'a, I: 'static + ?Sized, O: 'static + ?Sized> Map<I, O> for Ref<'a, I> {
    type Output = Ref<'a, O>;
    type Func = for<'b> fn(&'b I) -> &'b O;
    fn map(self, f: Self::Func) -> Ref<'a, O> {
        Ref::map(self, f)
    }
}

impl<'a, I: 'static + Send + ?Sized, O: 'static + Send + ?Sized> Map<I, O>
    for MappedMutexGuard<'a, I>
{
    type Output = MappedMutexGuard<'a, O>;
    type Func = for<'b> fn(&'b mut I) -> &'b mut O;
    fn map(self, f: Self::Func) -> MappedMutexGuard<'a, O> {
        MappedMutexGuard::map(self, f)
    }
}

impl<'a, I: 'static + Send + ?Sized, O: 'static + Send + ?Sized> Map<I, O>
    for MappedRwLockReadGuard<'a, I>
{
    type Output = MappedRwLockReadGuard<'a, O>;
    type Func = for<'b> fn(&'b I) -> &'b O;
    fn map(self, f: Self::Func) -> MappedRwLockReadGuard<'a, O> {
        MappedRwLockReadGuard::map(self, f)
    }
}
/// A trait forcing the implementor to implement a `map` method
/// this is used to genericize over `MappedMutexGuard` and
/// `MappedRwLockWriteGuard` and `RefMut`
pub trait MapMut<I: ?Sized, O: ?Sized>: Deref<Target = I> + Sized + DerefMut {
    type Output: Deref<Target = O> + DerefMut;
    type Func: Sized + 'static;
    fn map(self, f: Self::Func) -> Self::Output;
}

impl<'a, I: 'static + ?Sized, O: 'static + ?Sized> MapMut<I, O> for RefMut<'a, I> {
    type Output = RefMut<'a, O>;
    type Func = for<'b> fn(&'b mut I) -> &'b mut O;
    fn map(self, f: Self::Func) -> RefMut<'a, O> {
        RefMut::map(self, f)
    }
}

impl<'a, I: 'static + Send + ?Sized, O: 'static + Send + ?Sized> MapMut<I, O>
    for MappedRwLockWriteGuard<'a, I>
{
    type Output = MappedRwLockWriteGuard<'a, O>;
    type Func = for<'b> fn(&'b mut I) -> &'b mut O;
    fn map(self, f: Self::Func) -> MappedRwLockWriteGuard<'a, O> {
        MappedRwLockWriteGuard::map(self, f)
    }
}

impl<'a, I: 'static + Send + ?Sized, O: 'static + Send + ?Sized> MapMut<I, O>
    for MappedMutexGuard<'a, I>
{
    type Output = MappedMutexGuard<'a, O>;
    type Func = for<'b> fn(&'b mut I) -> &'b mut O;
    fn map(self, f: Self::Func) -> MappedMutexGuard<'a, O> {
        MappedMutexGuard::map(self, f)
    }
}
///
/// The base structure for this library, contains all of the
/// dynamically typed storage units
///
/// This is the basis for this library. This should not be
/// directly interacted with, and should instead be interfaced
/// with the type alias at the root of this library:
///
/// * `DynamicStorage`:
/// Based on `RefCell`s, for its interior mutability.
/// This is _NOT_ `Send`, but it is faster, because it
/// does not use atomic operations.
/// * `MutexStorage`:
/// Uses a `Mutex` for `Send` capabilites, and interior mutability
/// This only exposes mutable getter methods, as there is only
/// a `&mut` api available for a `MappedMutexGuard`
/// * `RwLockStorage`:
/// This exposes the same api as a `RefCell` but is atomically guarded
/// and therefore guarantees a safe `Send`, while allowing multiple
/// readers.
///
/// The type parameter `U` is the `Unit` that is going to be used to store
/// the data that is placed into it. This type parameter should, once
/// again be avoided by the user, and should instead use the
/// type definitions that are noted above.
///
#[derive(Default)]
pub struct BlackBox<U: ?Sized> {
    pub(crate) data: HashMap<TypeId, Box<U>>,
}

type Borrowed<'a, T> = <T as Unit<'a>>::Borrowed;
type MutBorrowed<'a, T> = <T as Unit<'a>>::MutBorrowed;

impl<U: ?Sized + for<'a> Unit<'a, Owned = Box<(dyn Any + Send)>>> BlackBox<U> {
    ///
    /// A default implementation of `BlackBox`
    ///
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    ///
    /// Checks if there is an allocated unit for
    /// the type parameter in the internal hashmap.
    ///
    /// # Example
    /// ```
    /// # fn main() {
    /// use restor::DynamicStorage;
    /// let mut storage = DynamicStorage::new();
    /// assert!(!storage.has_unit::<usize>());
    /// storage.allocate_for::<usize>();
    /// assert!(storage.has_unit::<usize>());
    /// # }
    /// ```
    #[inline]
    pub fn has_unit<T: 'static + Send>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<T>())
    }

    ///
    /// Inserts a value into the storage and returns it in the case
    /// that it's impossible to insert or it is already borrowed.
    ///
    /// This appends to a list of values in the case that there is
    /// one or values of the type in the internal storage.
    ///
    /// # Example
    /// ```
    /// # fn main() {
    /// use restor::{DynamicStorage, ErrorDesc};
    /// let mut storage = DynamicStorage::new();
    /// assert_eq!(storage.insert(0usize), Err((0usize, ErrorDesc::NoAllocatedUnit)));
    /// storage.allocate_for::<usize>();
    /// storage.insert(0usize).unwrap();
    /// # }
    /// ```
    ///
    /// # Example 2: Appending
    /// ```
    /// # fn main() {
    /// use restor::{DynamicStorage, ErrorDesc};
    /// let mut storage = DynamicStorage::new();
    /// storage.allocate_for::<usize>();
    /// storage.insert(0usize).unwrap();
    /// storage.insert(1usize).unwrap();
    /// storage.insert(2usize).unwrap();
    /// storage.run_for::<usize, (), _>(|x| {
    ///     assert_eq!(x.unwrap().len(), 3);
    ///     None
    /// });
    /// # }
    /// ```
    ///
    /// ## Note
    /// This returns a `Result<(), (T, ErrorDesc)>` for ease of use, with calling `.unwrap()`.
    ///
    pub fn insert<T: 'static + Send>(&self, data: T) -> Result<(), (T, ErrorDesc)> {
        let entry = self.data.get(&TypeId::of::<T>());
        match entry {
            Some(x) => match x.insert_any(Box::new(data)) {
                Some((x, e)) => Err((*x.downcast().unwrap(), e)),
                None => Ok(()),
            },
            None => Err((data, ErrorDesc::NoAllocatedUnit)),
        }
    }

    ///
    /// Sibling to `insert`, this inserts many values at the same time and returns them
    /// in the case of an error. This will append to a pre-exisiting dataset if there
    /// is one present, or a single value, if possible.
    ///
    /// # Example
    /// ```
    /// # fn main() {
    /// use restor::{DynamicStorage, ErrorDesc};
    /// let mut storage = DynamicStorage::new();
    /// assert_eq!(storage.insert_many(vec![0usize, 1, 2, 3]), Err((vec![0usize, 1, 2, 3], ErrorDesc::NoAllocatedUnit)));
    /// storage.allocate_for::<usize>();
    /// storage.insert_many(vec![0usize, 1, 2, 3]).unwrap();
    /// storage.insert_many(vec![4usize, 5, 6, 7]).unwrap();
    /// storage.run_for::<usize, (), _>(|x| {
    ///     assert_eq!(x.unwrap(), &[0usize, 1, 2, 3, 4, 5, 6, 7]);
    ///     None
    /// });
    /// # }
    /// ```
    ///
    /// ## Note
    /// This returns the `Vec` passed to it in the case of an erroneous attempt
    /// at inserting into the storage.
    ///
    pub fn insert_many<T: 'static + Send>(&self, data: Vec<T>) -> Result<(), (Vec<T>, ErrorDesc)> {
        if let Some(unit) = self.data.get(&TypeId::of::<T>()) {
            if let Some((ret, e)) = unit.insert_any(Box::new(data)) {
                Err((*ret.downcast().unwrap(), e))
            } else {
                Ok(())
            }
        } else {
            Err((data, ErrorDesc::NoAllocatedUnit))
        }
    }

    ///
    /// Internal function. Returns a reference to the `Unit` for `T`
    ///
    #[inline]
    fn unit_get<T: 'static + Send>(&self) -> DynamicResult<&U> {
        self.data
            .get(&TypeId::of::<T>())
            .map(|x| &**x)
            .ok_or(ErrorDesc::NoAllocatedUnit)
    }

    ///
    /// Returns a mutable lock on a value of type `T`.
    /// This will return:
    ///
    /// - A [`RefMut<'a, T>`] in the case of `DynamicStorage`
    /// - A [`MappedMutexGuard<'a, T>`] in the case of `MutexStorage`
    /// - A [`MappedRwLockWriteGuard<'a, T>`] in the case of `RwLockStorage`
    ///
    /// In the case that there is no unit, or that the data is incompatibly borrowed,
    /// an `Err` value will be returned, containing a description of the error in the
    /// enum variant name. This will also return an `Err` variant if more than one `T`
    /// are stored in the storage.
    ///
    /// # Example
    /// ```
    /// # fn main() {
    /// use restor::DynamicStorage;
    ///
    /// let mut storage = DynamicStorage::new();
    /// storage.allocate_for::<String>();
    /// storage.insert(String::new());
    ///
    /// let mut lock = storage.get_mut::<String>().unwrap();
    /// lock.push_str("Abc");
    /// drop(lock);
    ///
    /// let lock = storage.get::<String>().unwrap();
    /// assert_eq!(&*lock, "Abc");
    ///
    /// # }
    /// ```
    ///
    /// [`MappedMutexGuard<'a, T>`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.MappedMutexGuard.html
    /// [`MappedRwLockWriteGuard<'a, T>`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.MappedRwLockWriteGuard.html
    /// [`RefMut<'a, T>`]: https://doc.rust-lang.org/std/cell/struct.RefMut.html
    ///
    #[inline]
    pub fn get_mut<'a, T: 'static + Send>(
        &'a self,
    ) -> DynamicResult<<MutBorrowed<'a, U> as MapMut<(dyn Any + Send), T>>::Output>
    where
        MutBorrowed<'a, U>: MapMut<(dyn Any + Send), T, Func = fn(&mut (dyn Any + Send)) -> &mut T>,
    {
        Ok(self
            .unit_get::<T>()?
            .one_mut()?
            .map(|x| x.downcast_mut().unwrap()))
    }

    ///
    /// Returns a mutable lock to the element at a given index.
    /// This will return the only element available if there is
    /// only one element in the storage, and the index specified
    /// is 0.
    ///
    /// This will return the same lock types as `BlackBox::get_mut()`
    ///
    /// # Example
    /// ```
    /// # fn main() {
    /// use restor::DynamicStorage;
    /// let mut storage = DynamicStorage::new();
    /// storage.allocate_for::<String>();
    /// storage.insert(String::new());
    /// {
    ///     let mut lock = storage.ind_mut::<String>(0).unwrap();
    ///     lock.push_str("abc");
    /// }
    /// storage.insert(String::new());
    /// storage.ind_mut::<String>(0).unwrap().push_str("def");
    /// assert_eq!(
    ///		&storage.run_for::<String, String, _>(|x| {
    /// 		let x = x.unwrap();
    ///         Some(x[0].clone() + &x[1])
    /// 	}).unwrap(),
    ///		"abcdef"
    /// );
    /// # }
    /// ```
    ///
    #[inline]
    pub fn ind_mut<'a, T: 'static + Send>(
        &'a self,
        ind: usize,
    ) -> DynamicResult<<MutBorrowed<'a, U> as MapMut<(dyn Any + Send), T>>::Output>
    where
        MutBorrowed<'a, U>: MapMut<(dyn Any + Send), T, Func = fn(&mut (dyn Any + Send)) -> &mut T>,
    {
        Ok(self
            .unit_get::<T>()?
            .ind_mut(ind)?
            .map(|x| x.downcast_mut().unwrap()))
    }

    ///
    /// Retrieves an owned `T` from the storage.
    ///
    /// Returns an `Err`or in the case that it is impossible to retrieve one.
    ///
    /// # Example
    /// ```
    /// # fn main() {
    ///	use restor::DynamicStorage;
    /// let mut storage = DynamicStorage::new();
    /// storage.allocate_for::<String>();
    /// storage.insert(String::new()).unwrap();
    /// let my_str: String = storage.extract::<String>().unwrap();
    /// assert_eq!(my_str, String::new());
    /// # }
    /// ```
    ///
    #[inline]
    pub fn extract<T: 'static + Send>(&self) -> DynamicResult<T> {
        Ok(*self.unit_get::<T>()?.extract()?.downcast().unwrap())
    }

    ///
    /// Extracts multiple values and returns them in the form of
    /// a `Box<[T]>` which can be turned into a `Vec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() {
    /// use restor::{DynamicStorage, make_storage};
    /// let storage = make_storage!(DynamicStorage: usize);
    /// storage.insert_many(vec![1usize, 2, 3, 4]).unwrap();
    /// let v: Vec<usize> = storage.extract_many::<usize>().unwrap().into();
    /// assert_eq!(v, vec![1usize, 2, 3, 4]);
    /// # }
    /// ```
    ///
    #[inline]
    pub fn extract_many<T: 'static + Send>(&self) -> DynamicResult<Vec<T>> {
        Ok(*self.unit_get::<T>()?.extract_many()?.downcast().unwrap())
    }

    ///
    /// Gets an immutable lock on the single value variant of storage.
    ///
    /// This will return an `Err` in the case that it is either borrowed
    /// incompatibly or there is no allocated unit.
    ///
    /// # Note
    /// This method is only available for use in the case where the underlying
    /// interior mutability type supports it:
    ///
    /// - `RwLockStorage`: `RwLock`s
    /// - `DynamicStorage`: `RefCell`s
    ///
    /// And so therefor it is not implemented in the case of `MutexStorage`
    ///
    /// # Example
    /// ```
    /// # fn main() {
    /// use restor::{DynamicStorage, make_storage};
    /// let storage = make_storage!(DynamicStorage: usize);
    /// storage.insert(32usize).unwrap();
    /// println!("{}", &*storage.get::<usize>().unwrap());
    /// # }
    /// ```
    ///
    #[inline]
    pub fn get<'a, T: 'static + Send>(
        &'a self,
    ) -> DynamicResult<<Borrowed<'a, U> as Map<(dyn Any + Send), T>>::Output>
    where
        Borrowed<'a, U>: Map<(dyn Any + Send), T, Func = for<'b> fn(&'b (dyn Any + Send)) -> &'b T>,
    {
        Ok(self
            .unit_get::<T>()?
            .one()?
            .map(|x| x.downcast_ref().unwrap()))
    }

    #[inline]
    pub fn ind<'a, T: 'static + Send>(
        &'a self,
        ind: usize,
    ) -> DynamicResult<<Borrowed<'a, U> as Map<(dyn Any + Send), T>>::Output>
    where
        Borrowed<'a, U>: Map<(dyn Any + Send), T, Func = for<'b> fn(&'b (dyn Any + Send)) -> &'b T>,
    {
        Ok(self
            .unit_get::<T>()?
            .ind(ind)?
            .map(|x| x.downcast_ref().unwrap()))
    }

    #[inline]
    pub fn run_for<
        'a,
        T: 'static + Send,
        D: 'static + Any,
        F: FnMut(DynamicResult<&[T]>) -> Option<D>,
    >(
        &self,
        mut f: F,
    ) -> Option<D> {
        let mut new_fn = |x: DynamicResult<&[T]>| {
            let var: Option<D> = f(x);
            var.map(|x| Box::new(x) as Box<dyn Any>)
        };

        let ptr = unsafe {
            std::mem::transmute::<_, (*const (), *const ())>(
                &mut new_fn as &mut dyn FnMut(DynamicResult<&[T]>) -> Option<Box<dyn Any>>,
            )
        };

        let t = TypeId::of::<(dyn FnMut(DynamicResult<&[T]>) -> Option<Box<dyn Any>>)>();

        let unit = self.unit_get::<T>();

        if let Ok(x) = unit {
            unsafe {
                let val = x.run_for((t, ptr));
                val.map(|x| *x.downcast().unwrap())
            }
        } else {
            None
        }
    }
}

impl
    BlackBox<
        (dyn for<'a> Unit<
            'a,
            Borrowed = MappedRwLockReadGuard<'a, (dyn Any + Send)>,
            MutBorrowed = MappedRwLockWriteGuard<'a, (dyn Any + Send)>,
            Owned = Box<(dyn Any + Send)>,
        > + Send),
    >
{
    #[inline]
    pub fn allocate_for<T: 'static + Send>(&mut self) {
        self.data
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(RwLockUnit::new(StorageUnit::<T>::new())));
    }
}

impl
    BlackBox<
        (dyn for<'a> Unit<
            'a,
            Borrowed = MappedMutexGuard<'a, (dyn Any + Send)>,
            MutBorrowed = MappedMutexGuard<'a, (dyn Any + Send)>,
            Owned = Box<(dyn Any + Send)>,
        > + Send),
    >
{
    #[inline]
    pub fn allocate_for<T: 'static + Send>(&mut self) {
        self.data
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(MutexUnit::new(StorageUnit::<T>::new())));
    }
}

impl
    BlackBox<
        (dyn for<'a> Unit<
            'a,
            Borrowed = Ref<'a, (dyn Any + Send)>,
            MutBorrowed = RefMut<'a, (dyn Any + Send)>,
            Owned = Box<(dyn Any + Send)>,
        >),
    >
{
    #[inline]
    pub fn allocate_for<T: 'static + Send>(&mut self) {
        self.data
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(RefCellUnit::new(StorageUnit::<T>::new())));
    }
}

unsafe impl Send
    for BlackBox<
        (dyn for<'a> Unit<
            'a,
            Borrowed = MappedMutexGuard<'a, (dyn Any + Send)>,
            MutBorrowed = MappedMutexGuard<'a, (dyn Any + Send)>,
            Owned = Box<(dyn Any + Send)>,
        > + Send),
    >
{
}

unsafe impl Sync
    for BlackBox<
        (dyn for<'a> Unit<
            'a,
            Borrowed = MappedMutexGuard<'a, (dyn Any + Send)>,
            MutBorrowed = MappedMutexGuard<'a, (dyn Any + Send)>,
            Owned = Box<(dyn Any + Send)>,
        > + Send),
    >
{
}
