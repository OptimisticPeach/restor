use restor::{err, ok, ErrorDesc, MutexStorage};
use std::sync::Arc;
use std::thread::spawn;
use std::time::Duration;

#[test]
fn instantiate() {
    let _ = MutexStorage::new();
}

#[test]
fn register() {
    let mut x = MutexStorage::new();
    x.allocate_for::<usize>();
}

#[test]
fn register_multiple() {
    let mut x = MutexStorage::new();
    x.allocate_for::<usize>();
    x.allocate_for::<isize>();
}

#[test]
fn register_repeated() {
    let mut x = MutexStorage::new();
    x.allocate_for::<usize>();
    x.allocate_for::<usize>();
}

#[test]
fn insert() {
    let mut x = MutexStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
}

#[test]
fn insert_non_registered() {
    let mut x = MutexStorage::new();
    x.allocate_for::<usize>();
    assert_eq!(x.insert(0isize), Err((0isize, ErrorDesc::NoAllocatedUnit)));
}

#[test]
fn borrow_twice_mut() {
    let mut x = MutexStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
    let y = x.get::<&mut usize>();
    assert!(y.is_ok());
    let z = x.get::<&mut usize>();
    if let Err(ErrorDesc::BorrowedIncompatibly) = z {
    } else {
        panic!();
    }
}

#[test]
fn slice_mut() {
    let mut x = MutexStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
    x.insert(1usize).unwrap();
    {
        let y = x.get::<&mut [usize]>();
        ok!(y, 0usize, [0])[0] = 10;
    }
    {
        let y = x.get::<&mut [usize]>();
        ok!(y, 1usize, [1]);
    }
    {
        let y = x.get::<&mut [usize]>();
        let z = x.get::<&mut [usize]>();
        err!(z, ErrorDesc::BorrowedIncompatibly);
        ok!(y, 10usize, [0]);
    }
}

#[test]
fn concurrent_ind_mut() {
    let mut x = MutexStorage::new();
    x.allocate_for::<usize>();
    let x = Arc::new(x);
    let xc = x.clone();
    x.insert(0usize).unwrap();
    x.insert(1usize).unwrap();
    let t = spawn(move || {
        let y = xc.get::<&mut [usize]>();
        ok!(y, 0, [0])[0] = 10;
    });
    t.join();
    let xc = x.clone();
    let t = spawn(move || {
        let y = xc.get::<&mut [usize]>();
        ok!(y, 1, [1]);
    });
    let xc = x.clone();
    let t1 = spawn(move || {
        let y = xc.get::<&mut [usize]>();
        std::thread::sleep(Duration::from_millis(240));
        ok!(y, 10, [0]);
    });
    let t2 = spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        let z = x.get::<&mut [usize]>();
        err!(z, ErrorDesc::BorrowedIncompatibly);
    });
    t1.join();
    t2.join();
}
