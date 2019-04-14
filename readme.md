# restor
A dyamic resource storage written in rust. It supports storage of multiple types and multiple entries. It also supports dynamic borrow checking with the help of [`RefCell`](https://doc.rust-lang.org/std/cell/struct.RefCell.html)s.

## Example:
```rust
use restor::DynamicStorage;

fn main() {
    let mut storage = DynamicStorage::new();
    storage.allocate_for::<usize>();
    storage.allocate_for::<String>();
    
    storage.insert(0usize);
    storage.insert_many(vec![1usize, 2, 3]);
    storage.insert("abc".to_string());

    for i in 0..4 {
        let mut string_accquisition = storage.get_mut::<String>();
        *string_accquisition = format!("{}, {}", &*string_accquisition, *storage.ind::<usize>(i));
    }

    assert_eq!(&*storage.get::<String>(), "abc, 0, 1, 2, 3");
}
```

## How it works:
`BlackBox` (Or `DynamicStorage`) is defined as so:
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
5. Pur inro its own place in the storage or in a `Vec`

## What's coming up:
- A multithreaded version
- A more ergonomic api
  - The ability to pass a `Fn` to run on a piece of data or a slice
    - Callbacks in general
  - The ability to iterate over a unit's contents
  - The ability to get a reference to the inner `Unit`
  - The ability to check if there is a unit attached to a type
    - The ability to insert a piece of data without worrying about the unit
  - (Very unlikely) The ability to only borrow a part of the storage in the case of multiple items
- A passthrough hasher to avoid the unnecessary hashing of the `TypeId`