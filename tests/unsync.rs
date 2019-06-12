use restor::{err, ok, DynamicStorage, ErrorDesc};

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
    let y = x.get::<&[usize]>();
    ok!(y, 0, [0]);
}

#[test]
fn ind_many() {
    let mut x = DynamicStorage::new();
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
    let mut x = DynamicStorage::new();
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
