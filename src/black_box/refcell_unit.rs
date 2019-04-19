use std::any::{Any, TypeId};
use std::cell::{Ref, RefMut};

use super::*;

impl<'a, T: 'static> Unit<'a> for NonGuardedUnit<StorageUnit<T>> {
    type Borrowed = Ref<'a, dyn Any>;
    type MutBorrowed = RefMut<'a, dyn Any>;
    type Owned = Box<dyn Any>;
    fn one(&'a self) -> DynamicResult<Ref<'a, dyn Any>> {
        if let Some(nx) = self.inner.try_borrow().ok() {
            match nx.one() {
                Ok(_) => Ok(Ref::map(nx, |nx| &*nx.one().unwrap())),
                Err(e) => Err(ErrorDesc::Inner(Box::new(e))),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn one_mut(&'a self) -> DynamicResult<RefMut<'a, dyn Any>> {
        if let Some(mut nx) = self.inner.try_borrow_mut().ok() {
            match nx.one_mut() {
                Ok(_) => Ok(RefMut::map(nx, |nx| &mut *nx.one_mut().unwrap())),
                Err(e) => Err(ErrorDesc::Inner(Box::new(e))),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }

    fn ind(&'a self, ind: usize) -> DynamicResult<Ref<'a, dyn Any>> {
        if let Some(nx) = self.inner.try_borrow().ok() {
            match nx.many() {
                Ok(slice) => match slice.get(ind) {
                    Some(_) => Ok(Ref::map(nx, |nx| &*nx.many().unwrap().get(ind).unwrap())),
                    None => Err(ErrorDesc::Unit(UnitError::OutOfBounds)),
                },
                Err(e) => Err(ErrorDesc::Inner(Box::new(e))),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn ind_mut(&'a self, ind: usize) -> DynamicResult<RefMut<'a, dyn Any>> {
        if let Some(mut nx) = self.inner.try_borrow_mut().ok() {
            match nx.many_mut() {
                Ok(slice) => match slice.get_mut(ind) {
                    Some(_) => Ok(RefMut::map(nx, |nx| {
                        &mut *nx.many_mut().unwrap().get_mut(ind).unwrap()
                    })),
                    None => Err(ErrorDesc::Unit(UnitError::OutOfBounds)),
                },
                Err(e) => Err(ErrorDesc::Inner(Box::new(e))),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }

    fn extract(&self) -> DynamicResult<Box<dyn Any>> {
        if let Some(mut x) = self.inner.try_borrow_mut().ok() {
            match x.extract_one() {
                Ok(x) => Ok(Box::new(x)),
                Err(e) => Err(ErrorDesc::Inner(Box::new(e))),
            }
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn extract_ind(&self, ind: usize) -> DynamicResult<Box<dyn Any>> {
        if let Some(mut borrowed) = self.inner.try_borrow_mut().ok() {
            borrowed.many_mut().and_then(|x| {
                if ind < x.len() {
                    let x: Box<dyn Any> = Box::new(x.remove(ind));
                    Ok(x)
                } else {
                    Err(ErrorDesc::Unit(UnitError::OutOfBounds))
                }
            })
        } else {
            Err(ErrorDesc::BorrowedIncompatibly)
        }
    }
    fn extract_many(&self) -> DynamicResult<Box<dyn Any>> {
        Ok(Box::new(
            self.inner
                .try_borrow_mut()
                .map_err(|_| ErrorDesc::BorrowedIncompatibly)?
                .extract_many_boxed(),
        ))
    }

    fn insert_any(&self, new: Box<dyn Any>) -> Option<(Box<dyn Any>, ErrorDesc)> {
        let newtype = new.type_id();
        if let Some(mut x) = self.inner.try_borrow_mut().ok() {
            if new.is::<T>() {
                x.insert(*new.downcast::<T>().expect(&format!(
                    "Tried to insert an object with type {:?} into a storage of type {:?}",
                    newtype,
                    TypeId::of::<T>()
                )));
                None
            } else if new.is::<Box<Vec<T>>>() {
                x.insert_many(*new.downcast::<Vec<T>>().unwrap());
                None
            } else {
                Some((new, ErrorDesc::NoMatchingType))
            }
        } else {
            Some((new, ErrorDesc::BorrowedIncompatibly))
        }
    }

    fn id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}
