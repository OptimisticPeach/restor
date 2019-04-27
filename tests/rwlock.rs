use restor::{RwLockStorage, ErrorDesc};

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
    assert!(x.insert(0usize).is_none());
}

#[test]
fn insert_non_registered() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    assert_eq!(x.insert(0isize), Some((0isize, ErrorDesc::NoAllocatedUnit)));
}

#[test]
fn borrow_twice_im() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    assert!(x.insert(0usize).is_none());
    let y = x.get::<usize>();
    assert!(y.is_ok());
    let z = x.get::<usize>();
    assert!(z.is_ok());
    drop(y);
    drop(z);
}

#[test]
fn borrow_twice_mut() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    assert!(x.insert(0usize).is_none());
    let y = x.get_mut::<usize>();
    assert!(y.is_ok());
    let z = x.get_mut::<usize>();
    if let Err(ErrorDesc::BorrowedIncompatibly) = z {} else {
        panic!();
    }
}

#[test]
fn ind() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    assert!(x.insert(0usize).is_none());
    assert!(x.insert(1usize).is_none());
    let y = x.ind::<usize>(0);
    let indexed = x.ind::<usize>(0);
    assert!(indexed.is_ok());
    if let Ok(val) = indexed {
        assert_eq!(*val, 0);
    }
}

#[test]
fn ind_many() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    assert!(x.insert(0usize).is_none());
    assert!(x.insert(1usize).is_none());
    {
        let y = x.ind::<usize>(0);
        assert!(y.is_ok());
        if let Ok(z) = y {
            assert_eq!(*z, 0usize);
        }
    }
    {
        let y = x.ind::<usize>(1);
        assert!(y.is_ok());
        if let Ok(z) = y {
            assert_eq!(*z, 1usize);
        }
    }
    {
        let y = x.ind::<usize>(0);
        assert!(y.is_ok());
        if let Ok(z) = y {
            assert_eq!(*z, 0usize);
        }
        let z = x.ind::<usize>(1);
        assert!(z.is_ok());
        if let Ok(nz) = z {
            assert_eq!(*nz, 1usize);
        }
    }
}

#[test]
fn ind_mut() {
    let mut x = RwLockStorage::new();
    x.allocate_for::<usize>();
    assert!(x.insert(0usize).is_none());
    assert!(x.insert(1usize).is_none());
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
        assert!(y.is_ok());
        if let Ok(z) = y {
            assert_eq!(*z, 1usize);
        }
    }
    {
        let y = x.ind_mut::<usize>(0);
        assert!(y.is_ok());
        if let Ok(z) = &y {
            assert_eq!(**z, 10usize);
        }
        let z = x.ind_mut::<usize>(1);
        assert!(z.is_err());
        if let Err(ErrorDesc::BorrowedIncompatibly) = z {} else {
            panic!("{:?}", *z.unwrap())
        }
    }
}
mod concurrent {
    use std::sync::Arc;
    use restor::{RwLockStorage, ErrorDesc};
    use std::thread::spawn;
    use std::time::Duration;

    #[test]
    fn ind_many() {
        let mut x = RwLockStorage::new();
        x.allocate_for::<usize>();
        let x = Arc::new(x);
        assert!(x.insert(0usize).is_none());
        assert!(x.insert(1usize).is_none());
        let xc = x.clone();
        let t = spawn(move ||
        {
            let y = (&*xc).ind::<usize>(0);
            assert!(y.is_ok());
            if let Ok(z) = y {
                assert_eq!(*z, 0usize);
            }
        });
        t.join();
        let xc = x.clone();
        let t = spawn(move ||
        {
            let y = (&*xc).ind::<usize>(1);
            assert!(y.is_ok());
            if let Ok(z) = y {
                assert_eq!(*z, 1usize);
            }
        });
        t.join();
        let xc = x.clone();
        let t1 = spawn(move ||
        {
            let y = xc.ind::<usize>(0);
            assert!(y.is_ok());
            if let Ok(z) = y {
                assert_eq!(*z, 0usize);
            }
            std::thread::sleep(Duration::from_millis(240));
        });
        let t2 = spawn(move ||
        {
            std::thread::sleep(Duration::from_millis(200));
            let z = x.ind::<usize>(1);
            assert!(z.is_ok());
            if let Ok(nz) = z {
                assert_eq!(*nz, 1usize);
            }
        });
        t1.join();
        t2.join();
    }

    #[test]
    fn ind_mut() {
        let mut x = RwLockStorage::new();
        x.allocate_for::<usize>();
        let x = Arc::new(x);
        let xc = x.clone();
        assert!(x.insert(0usize).is_none());
        assert!(x.insert(1usize).is_none());
        let t = spawn(move ||
            {
                let y = xc.ind_mut::<usize>(0);
                assert!(y.is_ok());
                if let Ok(mut z) = y {
                    assert_eq!(*z, 0usize);
                    *z = 10;
                }
            });
        t.join();
        let xc = x.clone();
        let t = spawn(move ||
            {
                let y = xc.ind_mut::<usize>(1);
                assert!(y.is_ok());
                if let Ok(z) = y {
                    assert_eq!(*z, 1usize);
                }
            });
        let xc = <Arc<RwLockStorage> as Clone>::clone(&x);
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
            let z = (&*x).ind_mut::<usize>(1);
            assert!(z.is_err());
            if let Err(ErrorDesc::BorrowedIncompatibly) = z {} else {
                panic!("{:?}", *z.unwrap())
            }
        });
        t1.join();
        t2.join();
    }
}

