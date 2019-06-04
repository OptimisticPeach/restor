use restor::{ok, DynamicStorage, ErrorDesc};

#[test]
fn instantiate() {
    let _ = DynamicStorage::new();
}

#[test]
fn register() {
    let mut x = DynamicStorage::new();
    x.allocate_for::<usize>();
}

#[test]
fn register_multiple() {
    let mut x = DynamicStorage::new();
    x.allocate_for::<usize>();
    x.allocate_for::<isize>();
}

#[test]
fn register_repeated() {
    let mut x = DynamicStorage::new();
    x.allocate_for::<usize>();
    x.allocate_for::<usize>();
}

#[test]
fn insert() {
    let mut x = DynamicStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
}

#[test]
fn insert_non_registered() {
    let mut x = DynamicStorage::new();
    x.allocate_for::<usize>();
    assert_eq!(x.insert(0isize), Err((0isize, ErrorDesc::NoAllocatedUnit)));
}

#[test]
fn borrow_twice_im() {
    let mut x = DynamicStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
    let y = x.get::<&usize>();
    assert!(y.is_ok());
    let z = x.get::<&usize>();
    assert!(z.is_ok());
    drop(y);
    drop(z);
}

#[test]
fn borrow_twice_mut() {
    let mut x = DynamicStorage::new();
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
fn ind() {
    let mut x = DynamicStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
    x.insert(1usize).unwrap();
    let y = x.ind::<&usize>(0);
    let indexed = x.ind::<&usize>(0);
    ok!(indexed, 0, *);
}

#[test]
fn ind_many() {
    let mut x = DynamicStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
    x.insert(1usize).unwrap();
    {
        let y = x.ind::<&usize>(0);
        assert!(y.is_ok());
        if let Ok(z) = y {
            assert_eq!(*z, 0usize);
        }
    }
    {
        let y = x.ind::<&usize>(1);
        assert!(y.is_ok());
        if let Ok(z) = y {
            assert_eq!(*z, 1usize);
        }
    }
    {
        let y = x.ind::<&usize>(0);
        assert!(y.is_ok());
        if let Ok(z) = y {
            assert_eq!(*z, 0usize);
        }
        let z = x.ind::<&usize>(1);
        assert!(z.is_ok());
        if let Ok(nz) = z {
            assert_eq!(*nz, 1usize);
        }
    }
}

#[test]
fn ind_mut() {
    let mut x = DynamicStorage::new();
    x.allocate_for::<usize>();
    x.insert(0usize).unwrap();
    x.insert(1usize).unwrap();
    {
        let y = x.ind::<&mut usize>(0);
        assert!(y.is_ok());
        if let Ok(mut z) = y {
            assert_eq!(*z, 0usize);
            *z = 10;
        }
    }
    {
        let y = x.ind::<&mut usize>(1);
        assert!(y.is_ok());
        if let Ok(z) = y {
            assert_eq!(*z, 1usize);
        }
    }
    {
        let y = x.ind::<&mut usize>(0);
        assert!(y.is_ok());
        if let Ok(z) = &y {
            assert_eq!(**z, 10usize);
        }
        let z = x.ind::<&mut usize>(1);
        assert!(z.is_err());
        if let Err(ErrorDesc::BorrowedIncompatibly) = z {
        } else {
            panic!("{:?}", z.unwrap())
        }
    }
}
