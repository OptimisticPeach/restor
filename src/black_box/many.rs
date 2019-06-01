use super::{BlackBox, Borrowed, Map, MapMut, MutBorrowed, StorageUnit, Unit};
use std::any::Any;

pub trait Get<'a, U: Unit<'a, Owned = Box<dyn Any + 'static>> + ?Sized> {
    type Output: 'a;
    type MultipleOutput: 'a;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output;
    fn many(boxed: &'a BlackBox<U>) -> Self::MultipleOutput;
    fn ind(boxed: &'a BlackBox<U>, ind: usize) -> Self::Output;
}

impl<'a, T: Any + 'static, U: for<'b> Unit<'b, Owned = Box<dyn Any + 'static>> + ?Sized> Get<'a, U>
    for &T
where
    Borrowed<'a, U>: Map<(dyn Any), T, Func = dyn Fn(&dyn Any) -> &T>
        + Map<(dyn Any), [T], Func = dyn Fn(&dyn Any) -> &[T]>,
{
    type Output = <Borrowed<'a, U> as Map<(dyn Any), T>>::Output;
    type MultipleOutput = <Borrowed<'a, U> as Map<(dyn Any), [T]>>::Output;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output {
        let unit = boxed.unit_get::<T>().unwrap();
        let f: &dyn for<'r> Fn(&'r dyn Any) -> &'r T =
            &|x: &dyn Any| x.downcast_ref::<StorageUnit<T>>().unwrap().one().unwrap();
        Map::<dyn Any, T>::map(unit.storage().unwrap(), f)
    }
    fn many(boxed: &'a BlackBox<U>) -> Self::MultipleOutput {
        let unit = boxed.unit_get::<T>().unwrap();
        let f: &dyn for<'r> Fn(&'r dyn Any) -> &'r [T] =
            &|x: &dyn Any| &x.downcast_ref::<StorageUnit<T>>().unwrap().many().unwrap()[..];
        Map::<dyn Any, [T]>::map(unit.storage().unwrap(), f)
    }
    fn ind(boxed: &'a BlackBox<U>, ind: usize) -> Self::Output {
        let unit = boxed.unit_get::<T>().unwrap();
        let f: &dyn for<'r> Fn(&'r dyn Any) -> &'r T =
            &move |x: &dyn Any| &x.downcast_ref::<StorageUnit<T>>().unwrap().many().unwrap()[ind];
        Map::<dyn Any, T>::map(unit.storage().unwrap(), f)
    }
}

impl<'a, T: Any + 'static, U: for<'b> Unit<'b, Owned = Box<dyn Any + 'static>> + ?Sized> Get<'a, U>
    for &mut T
where
    MutBorrowed<'a, U>: MapMut<(dyn Any), T, Func = dyn Fn(&mut dyn Any) -> &mut T>
        + MapMut<(dyn Any), [T], Func = dyn Fn(&mut dyn Any) -> &mut [T]>,
{
    type Output = <MutBorrowed<'a, U> as MapMut<(dyn Any), T>>::Output;
    type MultipleOutput = <MutBorrowed<'a, U> as MapMut<(dyn Any), [T]>>::Output;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output {
        let unit = boxed.unit_get::<T>().unwrap();
        let f: &dyn Fn(&mut dyn Any) -> &mut T = &|x: &mut dyn Any| {
            x.downcast_mut::<StorageUnit<T>>()
                .unwrap()
                .one_mut()
                .unwrap()
        };
        MapMut::<dyn Any, T>::map(unit.storage_mut().unwrap(), f)
    }
    fn many(boxed: &'a BlackBox<U>) -> Self::MultipleOutput {
        let unit = boxed.unit_get::<T>().unwrap();
        let f: &dyn Fn(&mut dyn Any) -> &mut [T] = &|x: &mut dyn Any| {
            &mut x
                .downcast_mut::<StorageUnit<T>>()
                .unwrap()
                .many_mut()
                .unwrap()[..]
        };
        MapMut::<dyn Any, [T]>::map(unit.storage_mut().unwrap(), f)
    }
    fn ind(boxed: &'a BlackBox<U>, ind: usize) -> Self::Output {
        let unit = boxed.unit_get::<T>().unwrap();
        let f: &dyn Fn(&mut dyn Any) -> &mut T = &move |x: &mut dyn Any| {
            &mut x
                .downcast_mut::<StorageUnit<T>>()
                .unwrap()
                .many_mut()
                .unwrap()[ind]
        };
        MapMut::<dyn Any, T>::map(unit.storage_mut().unwrap(), f)
    }
}

pub trait Multiple<'a, U: ?Sized> {
    type Output: 'a;
    fn get_many(boxed: &'a BlackBox<U>) -> Self::Output;
}

pub trait SliceMany<'a, U: ?Sized>: Multiple<'a, U> {
    type SliceOutput: 'a;
    fn slice_many(boxed: &'a BlackBox<U>) -> Self::SliceOutput;
}

pub trait IndMultiple<'a, U: ?Sized>: Multiple<'a, U> {
    type Index;
    fn ind_many(boxed: &'a BlackBox<U>, index: Self::Index) -> <Self as Multiple<'a, U>>::Output;
}

macro_rules! replace_expr {
    ($_a:ident, $other:ident) => {
        $other
    };
}
macro_rules! impl_tuple {
    () => {};
    (($first_type:ident, $first_ind: ident) $(, ($typ:ident, $ind:ident))* ) => {
        impl<'a, U: ?Sized, $first_type, $($typ),*> Multiple<'a, U> for ($first_type, $($typ),*)
        where
            $(
                $typ: Get<'a, U>,
            )*
            $first_type: Get<'a, U>,
            U: Unit<'a, Owned = Box<dyn Any>>,
        {
            type Output = ($first_type::Output, $($typ::Output),*);
            fn get_many(boxed: &'a BlackBox<U>) -> Self::Output {
                ($first_type::get(boxed), $($typ::get(boxed)),*)
            }
        }

        impl<'a, U: ?Sized, $first_type, $($typ),*> SliceMany<'a, U> for ($first_type, $($typ),*)
        where
            $(
                $typ: Get<'a, U>,
            )*
            $first_type: Get<'a, U>,
            U: Unit<'a, Owned = Box<dyn Any>>,
        {
            type SliceOutput = ($first_type::MultipleOutput, $($typ::MultipleOutput),*);
            fn slice_many(boxed: &'a BlackBox<U>) -> Self::SliceOutput {
                ($first_type::many(boxed), $($typ::many(boxed)),*)
            }
        }

        impl<'a, U: ?Sized, $first_type, $($typ),*> IndMultiple<'a, U> for ($first_type, $($typ),*)
        where
            $(
                $typ: Get<'a, U>,
            )*
            $first_type: Get<'a, U>,
            U: Unit<'a, Owned = Box<dyn Any>>,
        {
            type Index = (replace_expr!($first_type, usize), $(replace_expr!($typ, usize)),*);
            fn ind_many(boxed: &'a BlackBox<U>, ($first_ind, $($ind),*): Self::Index) -> Self::Output {
                ($first_type::ind(boxed, $first_ind), $($typ::ind(boxed, $ind)),*)
            }
        }
        impl_tuple!($(($typ, $ind)),*);
    }
}

impl_tuple!(
    (A, a),
    (B, b),
    (C, c),
    (D, d),
    (E, e),
    (F, f),
    (G, g),
    (H, h),
    (I, i),
    (J, j),
    (K, k),
    (L, l),
    (M, m),
    (N, n),
    (O, o),
    (P, p)
);
