use super::errors::*;
use std::mem::swap;

///
/// The base storage unit for this library.
///
/// This is similar in implementation to [`SmallVec`], but is
/// specialized to this library's needs. In the future it is
/// possibly that this will simply be replaced with [`SmallVec`].
///
/// This should not be interacted with through the user, as
/// this is meant to be an internal implementation detail for
/// the user. This is usually abstracted through a type erased
/// `Unit`.
///
/// [`SmallVec`]: https://docs.rs/smallvec/0.6.9/smallvec/
///
pub enum StorageUnit<T: 'static> {
    Nope,
    One(T),
    Many(Vec<T>),
}

impl<T: Sized> StorageUnit<T> {
    pub fn new() -> Self {
        StorageUnit::Nope
    }

    pub fn insert(&mut self, new: T) {
        match self {
            StorageUnit::Nope => {
                *self = StorageUnit::One(new);
            }
            StorageUnit::One(_) => {
                let mut rep = StorageUnit::Many(vec![new]);
                swap(self, &mut rep);
                if let StorageUnit::One(prev) = rep {
                    if let StorageUnit::Many(v) = self {
                        v.insert(0, prev);
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            StorageUnit::Many(many) => {
                many.push(new);
            }
        }
    }

    pub fn insert_many(&mut self, mut new: Vec<T>) {
        match self {
            StorageUnit::Nope => {
                *self = StorageUnit::Many(new);
            }
            StorageUnit::One(_) => {
                let mut rep = StorageUnit::Many(new);
                swap(&mut rep, self);
                if let StorageUnit::One(val) = rep {
                    if let StorageUnit::Many(vec) = self {
                        vec.insert(0, val);
                    } else {
                        unreachable!()
                    }
                } else {
                    unreachable!()
                }
            }
            StorageUnit::Many(arr) => {
                arr.append(&mut new);
            }
        }
    }

    pub fn one(&self) -> DynamicResult<&T> {
        if let StorageUnit::One(x) = self {
            Ok(x)
        } else {
            Err(ErrorDesc::Unit(UnitError::IsNotOne))
        }
    }

    pub fn one_mut(&mut self) -> DynamicResult<&mut T> {
        if let StorageUnit::One(x) = self {
            Ok(x)
        } else {
            Err(ErrorDesc::Unit(UnitError::IsNotOne))
        }
    }

    pub fn many(&self) -> DynamicResult<&[T]> {
        if let StorageUnit::Many(x) = self {
            Ok(x)
        } else {
            Err(ErrorDesc::Unit(UnitError::IsNotMany))
        }
    }

    pub fn many_mut(&mut self) -> DynamicResult<&mut Vec<T>> {
        if let StorageUnit::Many(x) = self {
            Ok(x)
        } else {
            Err(ErrorDesc::Unit(UnitError::IsNotMany))
        }
    }

    pub fn extract_one(&mut self) -> DynamicResult<T> {
        match self {
            StorageUnit::Nope => Err(ErrorDesc::Unit(UnitError::IsNotOne)),
            StorageUnit::Many(x) => Ok(x.remove(0)),
            StorageUnit::One(_) => {
                let mut repl = StorageUnit::Nope;
                swap(&mut repl, self);
                if let StorageUnit::One(data) = repl {
                    Ok(data)
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub fn extract_many(&mut self) -> DynamicResult<Vec<T>> {
        match self {
            StorageUnit::Nope => Err(ErrorDesc::Unit(UnitError::IsNotMany)),
            StorageUnit::One(_) => {
                let mut repl = StorageUnit::Nope;
                swap(&mut repl, self);
                if let StorageUnit::One(data) = repl {
                    Ok(vec![data])
                } else {
                    unreachable!()
                }
            }
            StorageUnit::Many(_) => {
                let mut repl = StorageUnit::Nope;
                swap(&mut repl, self);
                if let StorageUnit::Many(data) = repl {
                    Ok(data)
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub fn extract_many_boxed(&mut self) -> DynamicResult<Box<[T]>> {
        match self {
            StorageUnit::Nope => Err(ErrorDesc::Unit(UnitError::IsNotMany)),
            StorageUnit::One(_) => Err(ErrorDesc::Unit(UnitError::IsNotMany)),
            StorageUnit::Many(_) => {
                let mut repl = StorageUnit::Nope;
                swap(&mut repl, self);
                if let StorageUnit::Many(data) = repl {
                    Ok(data.into_boxed_slice())
                } else {
                    unreachable!()
                }
            }
        }
    }
    #[inline]
    pub fn rearrange_if_necessary(&mut self) {
        if let StorageUnit::Many(v) = self {
            match v.as_slice() {
                [] => *self = StorageUnit::Nope,
                [_] => {
                    let mut one_container = StorageUnit::Nope;
                    swap(self, &mut one_container);
                    if let StorageUnit::Many(mut v) = one_container {
                        *self = StorageUnit::One(v.remove(0));
                    } else {
                        unreachable!()
                    }
                }
                _ => {}
            }
        }
    }
}

impl<T> Default for StorageUnit<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for StorageUnit<T> {
    fn clone(&self) -> Self {
        match self {
            StorageUnit::Nope => StorageUnit::Nope,
            StorageUnit::One(data) => StorageUnit::One(data.clone()),
            StorageUnit::Many(data) => StorageUnit::Many(data.clone()),
        }
    }
}
