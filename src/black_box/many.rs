use super::{BlackBox, Borrowed, Map, MapMut, MutBorrowed, StorageUnit, Unit};
use std::any::Any;

pub trait Get<'a, U: Unit<'a, Owned = Box<dyn Any + 'static>> + ?Sized> {
    type Output: 'a;
    type MultipleOutput: 'a;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output;
    fn many(boxed: &'a BlackBox<U>) -> Self::MultipleOutput;
}

impl<'a, T: Any + 'static, U: for<'b> Unit<'b, Owned = Box<dyn Any + 'static>> + ?Sized> Get<'a, U>
    for &T
where
    Borrowed<'a, U>: Map<(dyn Any), T, Func = fn(&dyn Any) -> &T>
        + Map<(dyn Any), [T], Func = fn(&dyn Any) -> &[T]>,
{
    type Output = <Borrowed<'a, U> as Map<(dyn Any), T>>::Output;
    type MultipleOutput = <Borrowed<'a, U> as Map<(dyn Any), [T]>>::Output;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output {
        boxed.get::<T>().unwrap()
    }
    fn many(boxed: &'a BlackBox<U>) -> Self::MultipleOutput {
        let unit = boxed.unit_get::<T>().unwrap();
        Map::<dyn Any, [T]>::map(unit.storage().unwrap(), |x: &dyn Any| {
            &x.downcast_ref::<StorageUnit<T>>().unwrap().many().unwrap()[..]
        })
    }
}

impl<'a, T: Any + 'static, U: for<'b> Unit<'b, Owned = Box<dyn Any + 'static>> + ?Sized> Get<'a, U>
    for &mut T
where
    MutBorrowed<'a, U>: MapMut<(dyn Any), T, Func = fn(&mut dyn Any) -> &mut T>
        + MapMut<(dyn Any), [T], Func = fn(&mut dyn Any) -> &mut [T]>,
{
    type Output = <MutBorrowed<'a, U> as MapMut<(dyn Any), T>>::Output;
    type MultipleOutput = <MutBorrowed<'a, U> as MapMut<(dyn Any), [T]>>::Output;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output {
        boxed.get_mut::<T>().unwrap()
    }
    fn many(boxed: &'a BlackBox<U>) -> Self::MultipleOutput {
        let unit = boxed.unit_get::<T>().unwrap();
        MapMut::<dyn Any, [T]>::map(unit.storage_mut().unwrap(), |x: &mut dyn Any| {
            &mut x
                .downcast_mut::<StorageUnit<T>>()
                .unwrap()
                .many_mut()
                .unwrap()[..]
        })
    }
}

pub trait Multiple<'a, U: ?Sized> {
    type Output: 'a;
    fn get_many(boxed: &'a BlackBox<U>) -> Self::Output;
}

pub trait SliceMany<'a, U: ?Sized> {
    type Output: 'a;
    fn slice_many(boxed: &'a BlackBox<U>) -> Self::Output;
}

macro_rules! impl_tuple {
    () => {};
    ($first:ident $(, $t:ident)*) => {
        impl<'a, U: ?Sized, $first, $($t),*> Multiple<'a, U> for ($first, $($t),*)
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

        impl<'a, U: ?Sized, $first, $($t),*> SliceMany<'a, U> for ($first, $($t),*)
        where
            $(
                $t: Get<'a, U>,
            )*
            $first: Get<'a, U>,
            U: Unit<'a, Owned = Box<dyn Any>>,
        {
            type Output = ($first::MultipleOutput, $($t::MultipleOutput),*);
            fn slice_many(boxed: &'a BlackBox<U>) -> Self::Output {
                ($first::many(boxed), $($t::many(boxed)),*)
            }
        }
        impl_tuple!($($t),*);
    }
}

impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
