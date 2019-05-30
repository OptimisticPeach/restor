use super::{BlackBox, Borrowed, Map, MapMut, MutBorrowed, Unit};
use std::any::Any;
use std::cell::{Ref, RefMut};
use std::ops::Deref;

pub trait Get<'a, U: Unit<'a, Owned = Box<dyn Any + 'static>>> {
    type Output: 'a;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output;
}

impl<'a, T: Any + 'static, U: for<'b> Unit<'b, Owned = Box<dyn Any + 'static>>> Get<'a, U> for &T
where
    Borrowed<'a, U>: Map<(dyn Any), T, Func = fn(&dyn Any) -> &T>,
{
    type Output = <Borrowed<'a, U> as Map<(dyn Any), T>>::Output;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output {
        boxed.get::<T>().unwrap()
    }
}

impl<'a, T: Any + 'static, U: for<'b> Unit<'b, Owned = Box<dyn Any + 'static>>> Get<'a, U>
    for &mut T
where
    MutBorrowed<'a, U>: MapMut<(dyn Any), T, Func = fn(&mut dyn Any) -> &mut T>,
{
    type Output = <MutBorrowed<'a, U> as MapMut<(dyn Any), T>>::Output;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output {
        boxed.get_mut::<T>().unwrap()
    }
}

trait Many<'a, U> {
    type Output: 'a;
    fn get_many(boxed: &'a BlackBox<U>) -> Self::Output;
}

impl<'a, U, T> Many<'a, U> for (T,)
where
    T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Output = (T::Output,);
    fn get_many(boxed: &'a BlackBox<U>) -> Self::Output {
        (T::get(boxed),)
    }
}
