use super::{BlackBox, Borrowed, Map, MapMut, MutBorrowed, Unit};
use std::any::Any;

pub trait Get<'a, U: Unit<'a, Owned = Box<dyn Any + 'static>> + ?Sized> {
    type Output: 'a;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output;
}

impl<'a, T: Any + 'static, U: for<'b> Unit<'b, Owned = Box<dyn Any + 'static>> + ?Sized> Get<'a, U>
    for &T
where
    Borrowed<'a, U>: Map<(dyn Any), T, Func = fn(&dyn Any) -> &T>,
{
    type Output = <Borrowed<'a, U> as Map<(dyn Any), T>>::Output;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output {
        boxed.get::<T>().unwrap()
    }
}

impl<'a, T: Any + 'static, U: for<'b> Unit<'b, Owned = Box<dyn Any + 'static>> + ?Sized> Get<'a, U>
    for &mut T
where
    MutBorrowed<'a, U>: MapMut<(dyn Any), T, Func = fn(&mut dyn Any) -> &mut T>,
{
    type Output = <MutBorrowed<'a, U> as MapMut<(dyn Any), T>>::Output;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output {
        boxed.get_mut::<T>().unwrap()
    }
}

pub trait Many<'a, U: ?Sized> {
    type Output: 'a;
    fn get_many(boxed: &'a BlackBox<U>) -> Self::Output;
}

macro_rules! impl_tuple {
    () => {};
    ($first:ident $(, $t:ident)*) => {
        impl<'a, U: ?Sized, $first, $($t),*> Many<'a, U> for ($first, $($t),*)
        where
            $(
                $t: Get<'a, U>,
            )*
            $first: Get<'a, U>,
            U: Unit<'a, Owned = Box<dyn Any>>,
        {
            type Output = ($first::Output, $($t::Output),*);
            fn get_many(boxed: &'a BlackBox<U>) -> Self::Output {
                ($first::get(boxed), $($t::get(boxed)),*)
            }
        }
        impl_tuple!($($t),*);
    }
}

impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
