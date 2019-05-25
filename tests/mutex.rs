use restor::{ErrorDesc, MutexStorage};
use std::sync::Arc;
use std::thread::spawn;
use std::time::Duration;

macro_rules! ok {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => panic!("Expected `Ok` but instead found `Err({:?})`", e),
        }
    };
    ($e:expr, $other:expr) => {
        match $e {
            Ok(x) => {
                assert_eq!(x, $other);
                x
            }
            Err(e) => panic!("Expected `Ok` but instead found `Err({:?})`", e),
        }
    };
    ($e:expr, $other:expr, *) => {
        match $e {
            Ok(x) => {
                assert_eq!(*x, $other);
                x
            }
            Err(e) => panic!("Expected `Ok` but instead found `Err({:?})`", e),
        }
    };
}

macro_rules! err {
    ($e:expr) => {
        match $e {
            Ok(_) => panic!("Expected `Err` but instead found `Ok(_)`"),
            Err(x) => x,
        }
    };
    ($e:expr, $other:expr) => {
        match $e {
            Ok(x) => panic!("Expected `Err` but instead found `Ok(_)`"),
            Err(e) => {
                assert_eq!(e, $other);
                e
            }
        }
    };
    ($e:expr, $other:expr, *) => {
        match $e {
            Ok(x) => panic!("Expected `Err` but instead found `Ok(_)`"),
            Err(e) => {
                assert_eq!(*e, $other);
                e
            }
        }
    };
}

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
    let y = x.get_mut::<usize>();
    assert!(y.is_ok());
    let z = x.get_mut::<usize>();
    if let Err(ErrorDesc::BorrowedIncompatibly) = z {
    } else {
        panic!();
    }
}

#[test]
fn ind_mut() {
    let mut x = MutexStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
    x.insert(1usize).unwrap();
    {
        let y = x.ind_mut::<usize>(0);
        assert!(y.is_ok());
        if let Ok(mut z) = y {
            assert_eq!(*z, 0usize);
            *z = 10;
        }
    }
    {
        let y = x.ind_mut::<usize>(1);
        ok!(y, 1usize, *);
    }
    {
        let y = x.ind_mut::<usize>(0);
        assert!(y.is_ok());
        if let Ok(z) = &y {
            assert_eq!(**z, 10usize);
        }
        let z = x.ind_mut::<usize>(1);
        assert!(z.is_err());
        if let Err(ErrorDesc::BorrowedIncompatibly) = z {
        } else {
            panic!("{:?}", *z.unwrap())
        }
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
        let y = xc.ind_mut::<usize>(0);
        assert!(y.is_ok());
        if let Ok(mut z) = y {
            assert_eq!(*z, 0usize);
            *z = 10;
        }
    });
    t.join();
    let xc = x.clone();
    let t = spawn(move || {
        let y = xc.ind_mut::<usize>(1);
        assert!(y.is_ok());
        if let Ok(z) = y {
            assert_eq!(*z, 1usize);
        }
    });
    let xc = x.clone();
    let t1 = spawn(move || {
        let y = xc.ind_mut::<usize>(0);
        assert!(y.is_ok());
        if let Ok(z) = &y {
            assert_eq!(**z, 10usize);
        }
        std::thread::sleep(Duration::from_millis(240));
    });
    let t2 = spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        let z = x.ind_mut::<usize>(1);
        assert!(z.is_err());
        if let Err(ErrorDesc::BorrowedIncompatibly) = z {
        } else {
            panic!("{:?}", *z.unwrap())
        }
    });
    t1.join();
    t2.join();
}
