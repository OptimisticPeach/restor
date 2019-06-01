use super::{BlackBox, Borrowed, Map, MapMut, MutBorrowed, StorageUnit, Unit};
use std::any::Any;

pub trait Get<'a, U: Unit<'a, Owned = Box<dyn Any + 'static>> + ?Sized> {
    type Output: 'a;
    type MultipleOutput: 'a;
    type Owned: 'static;
    type MultipleOwned: 'static;
    fn get(boxed: &'a BlackBox<U>) -> Self::Output;
    fn many(boxed: &'a BlackBox<U>) -> Self::MultipleOutput;
    fn ind(boxed: &'a BlackBox<U>, ind: usize) -> Self::Output;
    fn extract(boxed: &'a BlackBox<U>) -> Self::Owned;
    fn extract_ind(boxed: &'a BlackBox<U>, ind: usize) -> Self::Owned;
    fn extract_many(boxed: &'a BlackBox<U>) -> Self::MultipleOwned;
}

impl<'a, T: Any + 'static, U: for<'b> Unit<'b, Owned = Box<dyn Any + 'static>> + ?Sized> Get<'a, U>
    for &T
where
    Borrowed<'a, U>: Map<(dyn Any), T, Func = dyn Fn(&dyn Any) -> &T>
        + Map<(dyn Any), [T], Func = dyn Fn(&dyn Any) -> &[T]>,
{
    type Output = <Borrowed<'a, U> as Map<(dyn Any), T>>::Output;
    type MultipleOutput = <Borrowed<'a, U> as Map<(dyn Any), [T]>>::Output;
    type Owned = Box<T>;
    type MultipleOwned = Vec<T>;
    #[inline]
    fn get(boxed: &'a BlackBox<U>) -> Self::Output {
        let unit = boxed.unit_get::<T>().unwrap();
        let f: &dyn for<'r> Fn(&'r dyn Any) -> &'r T =
            &|x: &dyn Any| x.downcast_ref::<StorageUnit<T>>().unwrap().one().unwrap();
        Map::<dyn Any, T>::map(unit.storage().unwrap(), f)
    }
    #[inline]
    fn many(boxed: &'a BlackBox<U>) -> Self::MultipleOutput {
        let unit = boxed.unit_get::<T>().unwrap();
        let f: &dyn for<'r> Fn(&'r dyn Any) -> &'r [T] =
            &|x: &dyn Any| &x.downcast_ref::<StorageUnit<T>>().unwrap().many().unwrap()[..];
        Map::<dyn Any, [T]>::map(unit.storage().unwrap(), f)
    }
    #[inline]
    fn ind(boxed: &'a BlackBox<U>, ind: usize) -> Self::Output {
        let unit = boxed.unit_get::<T>().unwrap();
        let f: &dyn for<'r> Fn(&'r dyn Any) -> &'r T =
            &move |x: &dyn Any| &x.downcast_ref::<StorageUnit<T>>().unwrap().many().unwrap()[ind];
        Map::<dyn Any, T>::map(unit.storage().unwrap(), f)
    }
    #[inline]
    fn extract(boxed: &'a BlackBox<U>) -> Self::Owned {
        let unit = boxed.unit_get::<T>().unwrap();
        unit.extract().unwrap().downcast::<T>().unwrap()
    }
    #[inline]
    fn extract_ind(boxed: &'a BlackBox<U>, ind: usize) -> Self::Owned {
        let unit = boxed.unit_get::<T>().unwrap();
        unit.extract_ind(ind).unwrap().downcast::<T>().unwrap()
    }
    #[inline]
    fn extract_many(boxed: &'a BlackBox<U>) -> Self::MultipleOwned {
        let unit = boxed.unit_get::<T>().unwrap();
        *unit.extract_many().unwrap().downcast::<Vec<T>>().unwrap()
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
    type Owned = Box<T>;
    type MultipleOwned = Vec<T>;
    #[inline]
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
    #[inline]
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
    #[inline]
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
    #[inline]
    fn extract(boxed: &'a BlackBox<U>) -> Self::Owned {
        let unit = boxed.unit_get::<T>().unwrap();
        unit.extract().unwrap().downcast::<T>().unwrap()
    }
    #[inline]
    fn extract_ind(boxed: &'a BlackBox<U>, ind: usize) -> Self::Owned {
        let unit = boxed.unit_get::<T>().unwrap();
        unit.extract_ind(ind).unwrap().downcast::<T>().unwrap()
    }
    #[inline]
    fn extract_many(boxed: &'a BlackBox<U>) -> Self::MultipleOwned {
        let unit = boxed.unit_get::<T>().unwrap();
        *unit.extract_many().unwrap().downcast::<Vec<T>>().unwrap()
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

pub trait Extract<'a, U: ?Sized>: Multiple<'a, U> {
    type Owned;
    fn extract(boxed: &'a BlackBox<U>) -> Self::Owned;
}

pub trait ExtractInd<'a, U: ?Sized>: Multiple<'a, U> + Extract<'a, U> + IndMultiple<'a, U> {
    fn extract_ind(boxed: &'a BlackBox<U>, index: Self::Index) -> Self::Owned;
}

pub trait ExtractMultiple<'a, U: ?Sized>: Multiple<'a, U> {
    type MultipleOwned;
    fn extract_many(boxed: &'a BlackBox<U>) -> Self::MultipleOwned;
}

impl<'a, U: ?Sized, T: 'static> Multiple<'a, U> for &T
where
    &'static T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Output = <&'static T as Get<'a, U>>::Output;
    fn get_many(boxed: &'a BlackBox<U>) -> Self::Output {
        <&'static T>::get(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> Multiple<'a, U> for &mut T
where
    &'static mut T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Output = <&'static mut T as Get<'a, U>>::Output;
    fn get_many(boxed: &'a BlackBox<U>) -> Self::Output {
        <&'static mut T>::get(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> SliceMany<'a, U> for &T
where
    &'static T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type SliceOutput = <&'static T as Get<'a, U>>::MultipleOutput;
    fn slice_many(boxed: &'a BlackBox<U>) -> Self::SliceOutput {
        <&'static T>::many(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> SliceMany<'a, U> for &mut T
where
    &'static mut T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type SliceOutput = <&'static mut T as Get<'a, U>>::MultipleOutput;
    fn slice_many(boxed: &'a BlackBox<U>) -> Self::SliceOutput {
        <&'static mut T>::many(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> IndMultiple<'a, U> for &T
where
    &'static T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Index = usize;
    fn ind_many(boxed: &'a BlackBox<U>, index: usize) -> Self::Output {
        <&'static T>::ind(boxed, index)
    }
}

impl<'a, U: ?Sized, T: 'static> IndMultiple<'a, U> for &mut T
where
    &'static mut T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Index = usize;
    fn ind_many(boxed: &'a BlackBox<U>, index: usize) -> Self::Output {
        <&'static mut T>::ind(boxed, index)
    }
}

impl<'a, U: ?Sized, T: 'static> Extract<'a, U> for &T
where
    &'static T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Owned = <&'static T as Get<'a, U>>::Owned;
    fn extract(boxed: &'a BlackBox<U>) -> Self::Owned {
        <&'static T as Get<'a, U>>::extract(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> Extract<'a, U> for &mut T
where
    &'static mut T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Owned = <&'static mut T as Get<'a, U>>::Owned;
    fn extract(boxed: &'a BlackBox<U>) -> Self::Owned {
        <&'static mut T as Get<'a, U>>::extract(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> ExtractInd<'a, U> for &T
where
    &'static T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    fn extract_ind(boxed: &'a BlackBox<U>, index: usize) -> Self::Owned {
        <&'static T as Get<'a, U>>::extract_ind(boxed, index)
    }
}

impl<'a, U: ?Sized, T: 'static> ExtractInd<'a, U> for &mut T
where
    &'static mut T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    fn extract_ind(boxed: &'a BlackBox<U>, index: usize) -> Self::Owned {
        <&'static mut T as Get<'a, U>>::extract_ind(boxed, index)
    }
}

impl<'a, U: ?Sized, T: 'static> ExtractMultiple<'a, U> for &T
where
    &'static T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type MultipleOwned = <&'static T as Get<'a, U>>::MultipleOwned;
    fn extract_many(boxed: &'a BlackBox<U>) -> Self::MultipleOwned {
        <&'static T as Get<'a, U>>::extract_many(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> ExtractMultiple<'a, U> for &mut T
where
    &'static mut T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type MultipleOwned = <&'static mut T as Get<'a, U>>::MultipleOwned;
    fn extract_many(boxed: &'a BlackBox<U>) -> Self::MultipleOwned {
        <&'static mut T as Get<'a, U>>::extract_many(boxed)
    }
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

        impl<'a, U: ?Sized, $first_type, $($typ),*> Extract<'a, U> for ($first_type, $($typ),*)
        where
            $(
                $typ: Get<'a, U>,
            )*
            $first_type: Get<'a, U>,
            U: Unit<'a, Owned = Box<dyn Any>>,
        {
            type Owned = ($first_type::Owned, $($typ::Owned),*);
            fn extract(boxed: &'a BlackBox<U>) -> Self::Owned {
                ($first_type::extract(boxed), $($typ::extract(boxed)),*)
            }
        }

        impl<'a, U: ?Sized, $first_type, $($typ),*> ExtractInd<'a, U> for ($first_type, $($typ),*)
        where
            $(
                $typ: Get<'a, U>,
            )*
            $first_type: Get<'a, U>,
            U: Unit<'a, Owned = Box<dyn Any>>,
        {
            fn extract_ind(boxed: &'a BlackBox<U>, ($first_ind, $($ind),*): Self::Index) -> <Self as Extract<'a, U>>::Owned {
                ($first_type::extract_ind(boxed, $first_ind), $($typ::extract_ind(boxed, $ind)),*)
            }
        }

        impl<'a, U: ?Sized, $first_type, $($typ),*> ExtractMultiple<'a, U> for ($first_type, $($typ),*)
        where
            $(
                $typ: Get<'a, U>,
            )*
            $first_type: Get<'a, U>,
            U: Unit<'a, Owned = Box<dyn Any>>,
        {
            type MultipleOwned = ($first_type::MultipleOwned, $($typ::MultipleOwned),*);
            fn extract_many(boxed: &'a BlackBox<U>) -> Self::MultipleOwned {
                ($first_type::extract_many(boxed), $($typ::extract_many(boxed)),*)
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
