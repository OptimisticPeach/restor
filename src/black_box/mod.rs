use parking_lot::{MappedMutexGuard, MappedRwLockReadGuard, MappedRwLockWriteGuard};
use std::any::{Any, TypeId};
use std::cell::{Ref, RefMut};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

mod errors;
mod many;
mod storageunit;
mod unit;

pub use crate::black_box::unit::Unit;
pub use errors::{DynamicResult, ErrorDesc, UnitError};
pub use many::{Fetch, FetchMultiple};
pub use storageunit::StorageUnit;

mod refcell_unit;

pub use crate::black_box::refcell_unit::*;

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

impl<U: ?Sized + for<'a> Unit<'a, Owned = Box<dyn Any>>> BlackBox<U> {
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
    pub fn has_unit<T: 'static>(&self) -> bool {
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
    /// });
    /// # }
    /// ```
    ///
    /// ## Note
    /// This returns a `Result<(), (T, ErrorDesc)>` for ease of use, with calling `.unwrap()`.
    ///
    pub fn insert<T: 'static>(&self, data: T) -> Result<(), (T, ErrorDesc)> {
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
    /// });
    /// # }
    /// ```
    ///
    /// ## Note
    /// This returns the `Vec` passed to it in the case of an erroneous attempt
    /// at inserting into the storage.
    ///
    pub fn insert_many<T: 'static>(&self, data: Vec<T>) -> Result<(), (Vec<T>, ErrorDesc)> {
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
    pub(crate) fn unit_get<T: 'static>(&self) -> DynamicResult<&U> {
        self.data
            .get(&TypeId::of::<T>())
            .map(|x| &**x)
            .ok_or(ErrorDesc::NoAllocatedUnit)
    }

    ///
    /// Takes a function and runs it on the internal slice of data.
    ///
    /// The function may return a piece of data, which will be returned
    /// in the [`DynamicResult`]`<D>` that is returned.
    ///
    /// The function takes a `Result<&[T], ErrorDesc>`, so it is responsible
    /// for handling the case that there isn't available data or it's
    /// incompatibly borrowed or locked.
    ///
    /// The function is also `FnMut` so it can therefore mutate state
    /// such as a `move ||` closure.
    ///
    /// # Example
    /// ### Return nothing
    /// ```rust
    /// # fn main() {
    /// use restor::{DynamicStorage, make_storage};
    /// let storage = make_storage!(DynamicStorage: usize);
    /// storage.insert_many(vec![1usize, 2, 4, 8, 16, 32, 64, 128]).unwrap();
    /// storage.run_for::<usize, _, _>(|x| {
    ///     assert_eq!(x.unwrap().iter().sum::<usize>(), 0b11111111);
    /// });
    /// # }
    /// ```
    /// ### Return something
    /// ```rust
    /// # fn main() {
    /// use restor::{DynamicStorage, make_storage};
    /// let storage = make_storage!(DynamicStorage: usize);
    /// storage.insert_many(vec![0usize, 1, 2, 3, 4, 5, 6, 7]).unwrap();
    /// let transformed = storage.run_for::<usize, _, _>(|x| {
    ///     x.unwrap()
    ///      .iter()
    ///      .cloned()
    ///      .map(|nx| 2usize.pow(nx as u32))
    ///      .collect::<Vec<_>>()
    /// }).unwrap();
    /// assert_eq!(transformed,
    ///     vec![1, 2, 4, 8, 16, 32, 64, 128]
    /// );
    /// # }
    /// ```
    /// ### Handle error case
    /// ```
    /// # fn main() {
    /// use restor::{DynamicStorage, make_storage};
    /// let storage = make_storage!(DynamicStorage: usize);
    /// storage.insert_many(vec![1usize, 2, 4, 8, 16, 32, 64, 128]).unwrap();
    /// let res = storage.run_for::<usize, _, _>(|x| {
    ///     match x {
    ///         Ok(x) => Some(x.iter().sum::<usize>()),
    ///         Err(e) => {
    ///             println!("{:?}", e);
    ///             None
    ///         }
    ///     }
    /// });
    /// println!("{:?}", res);
    /// # }
    /// ```
    ///
    pub fn run_for<'a, T: 'static, D: 'static + Any, F: FnMut(DynamicResult<&[T]>) -> D + 'a>(
        &self,
        mut f: F,
    ) -> DynamicResult<D> {
        let mut new_fn = |x: DynamicResult<&[T]>| {
            let var: D = f(x);
            Box::new(var) as Box<dyn Any>
        };

        let ptr = unsafe {
            std::mem::transmute::<_, (*const (), *const ())>(
                &mut new_fn as &mut dyn FnMut(DynamicResult<&[T]>) -> Box<dyn Any>,
            )
        };

        let t = TypeId::of::<(dyn FnMut(DynamicResult<&[T]>) -> Box<dyn Any>)>();

        match self.unit_get::<T>() {
            Ok(x) => unsafe {
                let val = x.run_for((t, ptr));
                val.map(|x| *x.downcast().unwrap())
            },
            Err(e) => Err(e),
        }
    }

    ///
    /// Runs a function over a mutable [`Vec`] of type `T`, if there is a storage for
    /// `T` allocated. Similar to [`BlackBox::run_for`], this can optionally return an
    /// item, as long as it is an owned item, or has the same lifetime of the closure.
    ///
    /// The argument passed to the function is of type `Result<&mut Vec<T>, ErrorDesc>`
    /// so invalid attempts at running this function are handled within the closure.
    ///
    /// In the case that the `Vec` is left in an invalid state, only one value or no
    /// values, the internal storage is rearranged.
    ///
    /// # Note
    /// That this is the only way to extract an item from the storage given an index.
    ///
    /// # Examples
    /// ```rust
    /// # fn main() {
    /// use restor::{DynamicStorage, make_storage};
    /// let storage = make_storage!(DynamicStorage: usize);
    /// storage.insert_many(vec![0usize, 1, 2, 3, 4]);
    /// storage.run_for_mut::<usize, _, _>(|x| for i in x.unwrap() {*i *= 2; *i += 1;}).unwrap();
    /// storage.run_for::<usize, _, _>(|x| assert_eq!(x.unwrap(), &[1, 3, 5, 7, 9])).unwrap();
    /// # }
    /// ```
    ///
    /// ```rust
    /// # fn main() {
    /// use restor::{DynamicStorage, make_storage};
    /// let storage = make_storage!(DynamicStorage: usize);
    /// storage.insert_many(vec![0usize, 1, 2, 3, 4]);
    /// //Remove all but one element from the contents:
    /// let v = storage.run_for_mut::<usize, _, _>(|x| {
    ///     let x = x.unwrap();
    ///     x.split_off(1)
    /// }).unwrap();
    /// assert_eq!(v, vec![1, 2, 3, 4]);
    /// assert_eq!(*storage.get::<&usize>().unwrap(), 0usize);
    /// # }
    /// ```
    ///
    pub fn run_for_mut<
        'a,
        T: 'static,
        D: 'static + Any,
        F: FnMut(DynamicResult<&mut Vec<T>>) -> D + 'a,
    >(
        &self,
        mut f: F,
    ) -> DynamicResult<D> {
        let mut new_fn = |x: DynamicResult<&mut Vec<T>>| {
            let var: D = f(x);
            Box::new(var) as Box<dyn Any>
        };

        let ptr = unsafe {
            std::mem::transmute::<_, (*const (), *const ())>(
                &mut new_fn as &mut dyn FnMut(DynamicResult<&mut Vec<T>>) -> Box<dyn Any>,
            )
        };

        let t = TypeId::of::<(dyn FnMut(DynamicResult<&mut Vec<T>>) -> Box<dyn Any>)>();

        match self.unit_get::<T>() {
            Ok(x) => unsafe {
                let val = x.run_for((t, ptr));
                val.map(|x| *x.downcast().unwrap())
            },
            Err(e) => Err(e),
        }
    }

    ///
    /// "`get`"s values from the `BlackBox`, acquiring either locks or owned values
    /// depending on the type parameter(s) passed to this function. It follow these
    /// type mappings:
    ///
    /// - `&T -> Lock<T>`
    /// - `&mut T -> MutLock<T>`
    /// - `&[T] -> Lock<[T]>`
    /// - `&mut [T] -> MutLock<T>`
    /// - `Box<T> -> T`
    /// - `Vec<T> -> Vec<T>`
    ///
    /// Where `Lock` and `MutLock` are dependent on the kind of storage that you is
    /// being asked. For `DynamicStorage` it's [`Ref`] and [`RefMut`] respecitvely.
    /// For `MutexStorage` it is [`MappedMutexGuard`] for `MutLock`. It isn't possible
    /// to get a `Lock` from a `MutexStorage` due to its nature. For `RwLockStorage`
    /// a [`MappedRwLockReadGuard`] and a [`MappedRwLockWriteGuard`] are provided.
    ///
    /// [`Ref`]: https://doc.rust-lang.org/std/cell/struct.Ref.html
    /// [`RefMut`]: https://doc.rust-lang.org/std/cell/struct.RefMut.html
    /// [`MappedMutexGuard`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.MappedMutexGuard.html
    /// [`MappedRwLockReadGuard`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.MappedRwLockReadGuard.html
    /// [`MappedRwLockWriteGuard`]: https://docs.rs/parking_lot/0.8.0/parking_lot/type.MappedRwLockWriteGuard.html
    ///
    /// # Examples
    /// Read a single resource either mutably or immutably
    /// ```rust
    /// use restor::{DynamicStorage, make_storage, ok};
    /// let x = make_storage!(DynamicStorage: usize);
    /// x.insert(32usize).unwrap();
    /// let y = ok!(x.get::<&usize>());
    /// assert_eq!(*y, 32usize);
    /// drop(y);
    /// let mut y = ok!(x.get::<&mut usize>());
    /// *y = 20;
    /// drop(y);
    /// let y = ok!(x.get::<&usize>());
    /// assert_eq!(*y, 20);
    /// ```
    /// Read multiple resources either mutably or immutably
    /// ```rust
    /// use restor::{DynamicStorage, make_storage, ok};
    /// #[derive(Debug)]
    /// struct Person {
    ///     pub name: &'static str,
    ///     pub age: usize,
    /// }
    /// let x = make_storage!(DynamicStorage: usize, String, Person);
    /// let no_relatives = 3usize;
    /// let email = "john.doe@mailme.com".to_string();
    /// let person = Person {
    ///     name: "John Doe",
    ///     age: 32
    /// };
    /// x.insert(no_relatives).unwrap();
    /// x.insert(email).unwrap();
    /// x.insert(person).unwrap();
    /// {
    ///     let (person, no_relatives, mut email) = x.get::<(&Person, &usize, &mut String)>().unwrap();
    ///     println!("{:?}'s email is ", &*person);
    ///     println!("{}", &*email);
    ///     println!("And they've got {} relatives", *no_relatives);
    ///     *email = "doe.john@mailme.com".to_string();
    ///     println!("Their new email is {}", &*email);
    /// }
    /// ```
    ///
    #[inline(always)]
    pub fn get<'a, T: FetchMultiple<'a, U>>(&'a self) -> DynamicResult<T::Output> {
        T::get_many(self)
    }
}

impl
    BlackBox<
        (dyn for<'a> Unit<
            'a,
            Borrowed = Ref<'a, dyn Any>,
            MutBorrowed = RefMut<'a, dyn Any>,
            Owned = Box<dyn Any>,
        >),
    >
{
    #[inline]
    pub fn allocate_for<T: 'static>(&mut self) {
        self.data
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(RefCellUnit::new(StorageUnit::<T>::new())));
    }
}
