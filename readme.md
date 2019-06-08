# restor [![Crates.io](https://img.shields.io/badge/crates.io-docs.rs-brightgreen.svg?link=https://crates.io/crates/restor&link=https://docs.rs/restor/)](docs.rs/restor)
A dyamic resource storage written in rust. It supports storage of multiple types and multiple entries and dynamic borrow checking with the help of [`RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html)s, [`Mutex`](https://docs.rs/parking_lot/0.7.1/parking_lot/type.Mutex.html)s and [`RwLock`](https://docs.rs/parking_lot/0.7.1/parking_lot/type.RwLock.html)s from [`parking_lot`](https://docs.rs/parking_lot/0.7.1/parking_lot/index.html). It also supports extracting and aqcuiring multiple types at once. 

## Example:
```rust
use restor::{DynamicStorage, make_storage};

fn main() {
    // Use the shorthand for creating storage with preallocated types 
    let x = make_storage!(DynamicStorage: usize, String);
    // Insert some data into the storage, either many at once, or one
    x.insert_many((0..10).collect::<Vec<usize>>()).unwrap();
    x.insert("abc".to_string()).unwrap();
    create_string(&x);
    println!("{}", &*x.get::<&String>().unwrap());
}

fn create_string(x: &DynamicStorage) {
    let mut mystring = x.get::<&mut String>().unwrap();
    for i in x.get::<&[usize]>().unwrap().iter() {
        *mystring = format!("{}, {}", &*mystring, i);
    }
}
```
## How it works:
`BlackBox` (Or `DynamicStorage`) is defined as so (More or less):
```rust
struct BlackBox {
    data: HashMap<TypeId, Box<dyn Unit>>
}
```
The `Unit` trait allows us to abstract over the generic type of the container (Referred to as `UnitStorage` in the code), so we can pass data in and out of it by using the seemingly magical `Any` trait. When you insert something into the storage it goes through these stages:  
1. Your data in `BlackBox::insert<T>`
2. Boxed into a  `Box<dyn Any>`
3. Passed to the `StorageUnit as dyn Unit`
4. Try to downcast as either a `T` or a `Vec<T>`
5. Put into its own place in the storage or in a `Vec`

## What's coming up:
- ✓ A multithreaded version
- A more ergonomic api
  - Callbacks in general
    - ✓ The ability to pass a `FnMut(&[T])` to run on a piece of data or a slice
    - The ability to pass a `FnMut(&mut[T])` to run on a piece of data or a slice
  - The ability to get a reference to the inner `Unit`
  - ✓ The ability to check if there is a unit attached to a type
  - The ability to insert a piece of data without worrying about the unit
  - Add/get an item without worrying about errors, panic instead, and include `try_*` functions
- A passthrough hasher to avoid the unnecessary hashing of the `TypeId`
