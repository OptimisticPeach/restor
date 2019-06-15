use std::any::{Any, TypeId};
use std::collections::HashMap;

mod errors;
mod hasher;
mod many;
mod map;
mod refcell_unit;
mod storageunit;
mod unit;

pub use errors::{DynamicResult, ErrorDesc, UnitError};
use hasher::PassthroughHasherBuilder;
pub use many::{Fetch, FetchMultiple};
pub use map::{Map, MapMut};
pub use refcell_unit::{DynamicStorage, RefCellUnit};
pub use storageunit::StorageUnit;
pub use unit::{Unit, Waitable};

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
/// Uses a `Mutex` for `Send` capabilities, and interior mutability
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
    pub(crate) data: HashMap<TypeId, Box<U>, PassthroughHasherBuilder>,
}

pub(crate) type Borrowed<'a, T> = <T as Unit<'a>>::Borrowed;
pub(crate) type MutBorrowed<'a, T> = <T as Unit<'a>>::MutBorrowed;

impl<U: ?Sized + for<'a> Unit<'a>> BlackBox<U> {
    ///
    /// A default implementation of `BlackBox`
    ///
    pub fn new() -> Self {
        Self {
            data: HashMap::with_hasher(PassthroughHasherBuilder),
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
    ///     assert_eq!(x.len(), 3);
    /// });
    /// # }
    /// ```
    ///
    /// ## Note
    /// - This returns a `Result<(), (T, ErrorDesc)>` for ease of use, with calling `.unwrap()`.
    /// - It is currently impossible to insert `Vec<T>`s, which would result in inserting `T`s.
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
    /// A waiting version of [`BlackBox::insert`]. This will wait for a lock to be available
    /// so as to be able to insert the data. This will work with all of the examples from
    /// [`BlackBox::insert`], as long as the storage type is `RwLockStorage` or `MutexStorage`.
    ///
    /// [`BlackBox::insert`]: #method.insert
    ///
    pub fn waiting_insert<'a, T: 'static>(&'a self, data: T) -> Result<(), (T, ErrorDesc)>
    where
        Borrowed<'a, U>: Waitable,
        MutBorrowed<'a, U>: Waitable,
    {
        let entry = self.data.get(&TypeId::of::<T>());
        match entry {
            Some(x) => match x.waiting_insert(Box::new(data)) {
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
    ///     assert_eq!(x, &[0usize, 1, 2, 3, 4, 5, 6, 7]);
    /// });
    /// # }
    /// ```
    ///
    /// ## Note
    /// This returns the `Vec` passed to it in the case of an erroneous attempt
    /// at inserting into the storage.
    ///
    pub fn insert_many<T: 'static>(&self, data: Vec<T>) -> Result<(), (Vec<T>, ErrorDesc)> {
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
    /// Waits for a lock and inserts when possible. This, like [`insert_many`]
    /// returns a `Result<(), (Vec<T>, ErrorDesc)>`, which is meant to be used
    /// in most contexts as `storage.insert(value).unwrap()`.
    ///
    /// Please refer to both [`insert_many`] and [`insert`] for further info on
    /// this and related functions. Examples from both will work as long as the
    /// storage type used supports waiting, including both `RwLockStorage` and
    /// `MutexStorage`.
    ///
    /// [`insert_many`]: #method.insert_many
    /// [`insert`]: #method.insert
    ///
    pub fn waiting_insert_many<'a, T: 'static>(
        &'a self,
        data: Vec<T>,
    ) -> Result<(), (Vec<T>, ErrorDesc)>
    where
        Borrowed<'a, U>: Waitable,
        MutBorrowed<'a, U>: Waitable,
    {
        let entry = self.data.get(&TypeId::of::<T>());
        match entry {
            Some(x) => match x.waiting_insert(Box::new(data)) {
                Some((x, e)) => Err((*x.downcast().unwrap(), e)),
                None => Ok(()),
            },
            None => Err((data, ErrorDesc::NoAllocatedUnit)),
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
    /// The function takes a `&[T]`, so in the case it is impossible
    /// to acquire the appropriate data, it will short circuit and
    /// return the appropriate error instead of running `f`.
    ///
    /// The function is also `FnMut` so it can therefore mutate state
    /// such as in a `move ||` closure.
    ///
    /// [`DynamicResult`]: ./enum.ErrorDesc.html
    ///
    /// # Example
    /// ### Return nothing
    /// ```rust
    /// # fn main() {
    /// use restor::{DynamicStorage, make_storage};
    /// let storage = make_storage!(DynamicStorage: usize);
    /// storage.insert_many(vec![1usize, 2, 4, 8, 16, 32, 64, 128]).unwrap();
    /// storage.run_for::<usize, _, _>(|x| {
    ///     assert_eq!(x.iter().sum::<usize>(), 0b11111111);
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
    ///     x.iter()
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
    ///     x.iter().sum::<usize>()
    /// }).expect("Error, couldn't get lock");
    /// println!("{:?}", res);
    /// # }
    /// ```
    ///
    pub fn run_for<'a, 'b, T: 'static, D: 'static + Any, F: FnMut(&[T]) -> D + 'a>(
        &'b self,
        mut f: F,
    ) -> DynamicResult<D>
    where
        Borrowed<'b, U>: Map<dyn Any, StorageUnit<T>, Func = dyn Fn(&dyn Any) -> &StorageUnit<T>>,
    {
        let unit = self.unit_get::<T>()?;
        let dynstorage = unit.storage()?;
        let conv_func: &dyn for<'r> Fn(&'r dyn Any) -> &'r StorageUnit<T> =
            &|x| x.downcast_ref::<StorageUnit<T>>().unwrap();
        let storage = Map::map(dynstorage, conv_func);
        Ok(f(storage.many()?))
    }

    ///
    /// Waits for a lock and then runs a function over a slice within the lock.
    ///
    /// The only difference between this and [`run_for`] is that this will
    /// not immediately return in the case that it is impossible to acquire a
    /// lock to the data immediately upon calling.
    ///
    /// Any examples from [`run_for`] will work as long as the storage used
    /// is either [`RwLockStorage`] or [`MutexStorage`], because you cannot wait
    /// for a lock on a [`RefCell`] due to its single-threaded nature.
    ///
    /// [`run_for`]: #method.run_for
    /// [`RwLockStorage`]: ./struct.RwLockStorage.html
    /// [`MutexStorage`]: ./struct.MutexStorage.html
    /// [`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
    ///
    pub fn waiting_run_for<'a, 'b, T: 'static, D: 'static + Any, F: FnMut(&[T]) -> D + 'a>(
        &'b self,
        mut f: F,
    ) -> DynamicResult<D>
    where
        Borrowed<'b, U>:
            Map<dyn Any, StorageUnit<T>, Func = dyn Fn(&dyn Any) -> &StorageUnit<T>> + Waitable,
    {
        let unit = self.unit_get::<T>()?;
        let dynstorage = unit.waiting_storage();
        let conv_func: &dyn for<'r> Fn(&'r dyn Any) -> &'r StorageUnit<T> =
            &|x| x.downcast_ref::<StorageUnit<T>>().unwrap();
        let storage = Map::map(dynstorage, conv_func);
        Ok(f(storage.many()?))
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
    /// [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
    /// [`BlackBox::run_for`]: #method.run_for
    ///
    /// # Examples
    /// ```rust
    /// # fn main() {
    /// use restor::{DynamicStorage, make_storage};
    /// let storage = make_storage!(DynamicStorage: usize);
    /// storage.insert_many(vec![0usize, 1, 2, 3, 4]);
    /// storage.run_for_mut::<usize, _, _>(|x| for i in x {*i *= 2; *i += 1;}).unwrap();
    /// storage.run_for::<usize, _, _>(|x| assert_eq!(x, &[1, 3, 5, 7, 9])).unwrap();
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
    ///     x.split_off(1)
    /// }).unwrap();
    /// assert_eq!(v, vec![1, 2, 3, 4]);
    /// assert_eq!(*storage.get::<&usize>().unwrap(), 0usize);
    /// # }
    /// ```
    ///
    pub fn run_for_mut<'a, 'b, T: 'static, D: 'static + Any, F: FnMut(&mut Vec<T>) -> D + 'a>(
        &'b self,
        mut f: F,
    ) -> DynamicResult<D>
    where
        MutBorrowed<'b, U>:
            MapMut<dyn Any, StorageUnit<T>, Func = dyn Fn(&mut dyn Any) -> &mut StorageUnit<T>>,
    {
        let unit = self.unit_get::<T>()?;
        let dynstorage = unit.storage_mut()?;
        let conv_func: &dyn for<'r> Fn(&'r mut dyn Any) -> &'r mut StorageUnit<T> =
            &|x: &mut dyn Any| x.downcast_mut::<StorageUnit<T>>().unwrap();
        let mut storage = MapMut::map(dynstorage, conv_func);
        let res = f(storage.many_mut()?);
        storage.rearrange_if_necessary();
        Ok(res)
    }

    ///
    /// Waits for a lock and then runs a function over a slice within the lock.
    ///
    /// The only difference between this and [`run_for_mut`] is that this will
    /// not immediately return in the case that it is impossible to acquire a
    /// lock to the data immediately upon calling.
    ///
    /// Any examples from [`run_for_mut`] will work as long as the storage used
    /// is either [`RwLockStorage`] or [`MutexStorage`], because you cannot wait
    /// for a lock on a [`RefCell`] due to its single-threaded nature.
    ///
    /// [`run_for_mut`]: #method.run_for_mut
    /// [`RwLockStorage`]: ./struct.RwLockStorage.html
    /// [`MutexStorage`]: ./struct.MutexStorage.html
    /// [`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
    ///
    pub fn waiting_run_for_mut<
        'a,
        'b,
        T: 'static,
        D: 'static + Any,
        F: FnMut(&mut Vec<T>) -> D + 'a,
    >(
        &'b self,
        mut f: F,
    ) -> DynamicResult<D>
    where
        MutBorrowed<'b, U>:
            MapMut<dyn Any, StorageUnit<T>, Func = dyn Fn(&mut dyn Any) -> &mut StorageUnit<T>>
                + Waitable,
    {
        let unit = self.unit_get::<T>()?;
        let dynstorage = unit.waiting_storage_mut();
        let conv_func: &dyn for<'r> Fn(&'r mut dyn Any) -> &'r mut StorageUnit<T> =
            &|x: &mut dyn Any| x.downcast_mut::<StorageUnit<T>>().unwrap();
        let mut storage = MapMut::map(dynstorage, conv_func);
        let res = f(storage.many_mut()?);
        storage.rearrange_if_necessary();
        Ok(res)
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
    /// Acquiring other forms of data
    /// ```rust
    /// use restor::{make_storage, DynamicStorage};
    /// let storage = make_storage!(DynamicStorage: usize, String);
    /// storage.insert_many(vec![0usize, 1, 2, 3, 4, 3, 2, 1, 0]).unwrap();
    /// storage.insert(String::new()).unwrap();
    /// storage.insert(String::from("Text")).unwrap();
    /// //We can iter over the returned lock
    /// assert_eq!(storage.get::<&[usize]>().unwrap().iter().sum::<usize>(), 16);
    /// //We can also get mutable locks to slices
    /// for i in storage.get::<&mut [usize]>().unwrap().iter_mut() {
    ///     *i += 30;
    /// }
    /// //We can extract an item, either from the
    /// //first slot or the only item. Note that
    /// //the returned value is not `Box<usize>`
    /// assert_eq!(storage.get::<Box<usize>>().unwrap(), 30);
    /// //This works for tuples. Each item in the
    /// //tuple is individually acquired, so we can
    /// //acquire multiple of the same type at the
    /// //same time.
    /// let (strings, number, nums) = storage.get::<(Vec<String>, Box<usize>, &[usize])>().unwrap();
    /// assert_eq!(strings, vec![String::new(), String::from("Text")]);
    /// assert_eq!(number, 31);
    /// assert_eq!(&*nums, &[32, 33, 34, 33, 32, 31, 30]);
    /// ```
    ///
    #[inline(always)]
    pub fn get<'a, T: FetchMultiple<'a, U>>(&'a self) -> DynamicResult<T::Output> {
        T::get_many(self)
    }
    ///
    /// Waits to get a lock for each of the types instead of returning an error in the case of
    /// a blocking operation. This will still return an error in the case that it is impossible
    /// to acquire the lock, due to a data format inconsistency (Such as a `Many` present when
    /// a `One` was requested) or a lack of an allocated `StorageUnit`. All the examples on
    /// [`BlackBox::get`] still apply as long as the type of storage used is either `RwLockStorage`
    /// or `MutexStorage`, because they are able to block the thread to acquire a lock.
    ///
    #[inline(always)]
    pub fn waiting_get<'a, T: FetchMultiple<'a, U>>(&'a self) -> DynamicResult<T::Output>
    where
        Borrowed<'a, U>: Waitable,
        MutBorrowed<'a, U>: Waitable,
    {
        T::waiting_get_many(self)
    }
}
