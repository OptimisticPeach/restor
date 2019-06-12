#![allow(unused_must_use)]
use restor::{err, ok, ErrorDesc, RwLockStorage};

#[test]
fn instantiate() {
    let _ = RwLockStorage::new();
}

#[test]
fn register() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
}

#[test]
fn register_multiple() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    x.allocate_for::<isize>();
}

#[test]
fn register_repeated() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    x.allocate_for::<usize>();
}

#[test]
fn insert() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
}

#[test]
fn insert_non_registered() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    assert_eq!(x.insert(0isize), Err((0isize, ErrorDesc::NoAllocatedUnit)));
}

#[test]
fn borrow_twice_im() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
    let y = x.get::<&usize>();
    ok!(y);
    let z = x.get::<&usize>();
    ok!(z);
}

#[test]
fn borrow_twice_mut() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
    let y = x.get::<&mut usize>();
    let z = x.get::<&mut usize>();
    err!(z, ErrorDesc::BorrowedIncompatibly);
    ok!(y);
}

#[test]
fn ind() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
    x.insert(1usize).unwrap();
    let y = x.get::<&[usize]>();
    let indexed = x.get::<&[usize]>();
    ok!(indexed, 0, [0]);
    ok!(y, 1, [1]);
}

#[test]
fn ind_many() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
    x.insert(1usize).unwrap();
    {
        let y = x.get::<&[usize]>();
        ok!(y, 0, [0]);
    }
    {
        let y = x.get::<&[usize]>();
        ok!(y, 1, [1]);
    }
    {
        let y = x.get::<&[usize]>();
        ok!(y, 0, [0]);
        let z = x.get::<&[usize]>();
        ok!(z, 1, [1]);
    }
}

#[test]
fn ind_mut() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
    x.insert(1usize).unwrap();
    {
        let y = x.get::<&mut [usize]>();
        ok!(y, 0, [0])[0] = 10;
    }
    {
        let y = x.get::<&mut [usize]>();
        ok!(y, 1, [1]);
    }
    {
        let y = x.get::<&mut [usize]>();
        let z = x.get::<&mut [usize]>();
        err!(z, ErrorDesc::BorrowedIncompatibly);
        ok!(y, 10, [0]);
    }
}
mod concurrent {
    use restor::{ok, RwLockStorage};
    use std::sync::Arc;
    use std::thread::spawn;
    use std::time::Duration;

    #[test]
    fn ind_many() {
        let mut x = RwLockStorage::new();
        x.allocate_for::<usize>();
        let x = Arc::new(x);
        x.insert(0usize).unwrap();
        x.insert(1usize).unwrap();
        let xc = x.clone();
        let t = spawn(move || {
            let y = xc.get::<&[usize]>();
            ok!(y, 0, [0]);
        });
        t.join().unwrap();
        let xc = x.clone();
        let t = spawn(move || {
            let y = xc.get::<&[usize]>();
            ok!(y, 1, [1]);
        });
        t.join().unwrap();
        let xc = x.clone();
        let t1 = spawn(move || {
            let y = xc.get::<&[usize]>();
            ok!(y, 0, [0]);
            std::thread::sleep(Duration::from_millis(240));
        });
        let t2 = spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            let z = x.get::<&[usize]>();
            ok!(z, 1, [1]);
        });
        t1.join().unwrap();
        t2.join().unwrap();
    }

    #[test]
    fn ind_mut() {
        let mut x = RwLockStorage::new();
        x.allocate_for::<usize>();
        let x = Arc::new(x);
        let xc = x.clone();
        x.insert(0usize).unwrap();
        x.insert(1usize).unwrap();
        let t = spawn(move || {
            let y = xc.get::<&mut [usize]>();
            y.map(|m| m[0])
        });
        t.join().unwrap().unwrap();
        let xc = x.clone();
        let t = spawn(move || {
            let y = xc.get::<&mut [usize]>();
            y.map(|m| m[1])
        });
        t.join().unwrap().unwrap();
        let xc = <Arc<RwLockStorage> as Clone>::clone(&x);
        let t1 = spawn(move || {
            let y = xc.get::<&mut [usize]>();
            std::thread::sleep(Duration::from_millis(200));
            y.map(|m| m[0])
        });
        let t2 = spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            let z = x.get::<&mut [usize]>();
            z.map(|m| m[1])
        });
        t1.join().unwrap().unwrap();
        assert!(t2.join().unwrap().is_err());
    }
}
