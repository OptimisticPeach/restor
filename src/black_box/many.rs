use super::{
    BlackBox, Borrowed, DynamicResult, ErrorDesc, Map, MapMut, MutBorrowed, StorageUnit, Unit,
    UnitError,
};
use std::any::Any;

pub trait Get<'a, U: Unit<'a, Owned = Box<dyn Any + 'static>> + ?Sized> {
    type Output: 'a;
    type MultipleOutput: 'a;
    type Owned: 'static;
    type MultipleOwned: 'static;
    fn get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>;
    fn many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::MultipleOutput>;
    fn ind(boxed: &'a BlackBox<U>, ind: usize) -> DynamicResult<Self::Output>;
    fn extract(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Owned>;
    fn extract_ind(boxed: &'a BlackBox<U>, ind: usize) -> DynamicResult<Self::Owned>;
    fn extract_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::MultipleOwned>;
}

impl<'a, T: Any + 'static, U: for<'b> Unit<'b, Owned = Box<dyn Any + 'static>> + ?Sized> Get<'a, U>
    for &T
where
    Borrowed<'a, U>: Map<(dyn Any), StorageUnit<T>, Func = dyn Fn(&dyn Any) -> &StorageUnit<T>>,
    <Borrowed<'a, U> as Map<dyn Any, StorageUnit<T>>>::Output:
        Map<StorageUnit<T>, T, Func = dyn Fn(&StorageUnit<T>) -> &T>
            + Map<StorageUnit<T>, [T], Func = dyn Fn(&StorageUnit<T>) -> &[T]>,
{
    type Output = <<Borrowed<'a, U> as Map<dyn Any, StorageUnit<T>>>::Output as Map<
        StorageUnit<T>,
        T,
    >>::Output;
    type MultipleOutput = <<Borrowed<'a, U> as Map<dyn Any, StorageUnit<T>>>::Output as Map<
        StorageUnit<T>,
        [T],
    >>::Output;
    type Owned = Box<T>;
    type MultipleOwned = Vec<T>;
    #[inline]
    fn get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output> {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&dyn Any) -> &StorageUnit<T> =
            &|x| x.downcast_ref::<StorageUnit<T>>().unwrap();
        let unit = Map::<dyn Any, StorageUnit<T>>::map(unit.storage()?, f);
        unit.one()?;
        let f: &dyn for<'r> Fn(&'r StorageUnit<T>) -> &'r T = &|x| x.one().unwrap();
        Ok(Map::<StorageUnit<T>, T>::map(unit, f))
    }
    #[inline]
    fn many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::MultipleOutput> {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&dyn Any) -> &StorageUnit<T> =
            &|x| x.downcast_ref::<StorageUnit<T>>().unwrap();
        let unit = Map::<dyn Any, StorageUnit<T>>::map(unit.storage()?, f);
        unit.many()?;
        let f: &dyn Fn(&StorageUnit<T>) -> &[T] = &|x| &x.many().unwrap()[..];
        Ok(Map::<StorageUnit<T>, [T]>::map(unit, f))
    }
    #[inline]
    fn ind(boxed: &'a BlackBox<U>, ind: usize) -> DynamicResult<Self::Output> {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&dyn Any) -> &StorageUnit<T> =
            &|x| x.downcast_ref::<StorageUnit<T>>().unwrap();
        let unit = Map::<dyn Any, StorageUnit<T>>::map(unit.storage()?, f);
        unit.many()?
            .get(ind)
            .ok_or(ErrorDesc::Unit(UnitError::OutOfBounds))?;
        let f: &dyn Fn(&StorageUnit<T>) -> &T = &move |x| &x.many().unwrap()[ind];
        Ok(Map::<StorageUnit<T>, T>::map(unit, f))
    }
    #[inline]
    fn extract(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Owned> {
        let unit = boxed.unit_get::<T>()?;
        Ok(unit.extract()?.downcast::<T>().unwrap())
    }
    #[inline]
    fn extract_ind(boxed: &'a BlackBox<U>, ind: usize) -> DynamicResult<Self::Owned> {
        let unit = boxed.unit_get::<T>()?;
        Ok(unit.extract_ind(ind)?.downcast::<T>().unwrap())
    }
    #[inline]
    fn extract_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::MultipleOwned> {
        let unit = boxed.unit_get::<T>()?;
        Ok(*unit.extract_many()?.downcast::<Vec<T>>().unwrap())
    }
}

impl<'a, T: Any + 'static, U: for<'b> Unit<'b, Owned = Box<dyn Any + 'static>> + ?Sized> Get<'a, U>
    for &mut T
where
    MutBorrowed<'a, U>:
        MapMut<(dyn Any), StorageUnit<T>, Func = dyn Fn(&mut dyn Any) -> &mut StorageUnit<T>>,
    <MutBorrowed<'a, U> as MapMut<(dyn Any), StorageUnit<T>>>::Output:
        MapMut<StorageUnit<T>, [T], Func = dyn Fn(&mut StorageUnit<T>) -> &mut [T]>
            + MapMut<StorageUnit<T>, T, Func = dyn Fn(&mut StorageUnit<T>) -> &mut T>,
{
    type Output = <<MutBorrowed<'a, U> as MapMut<(dyn Any), StorageUnit<T>>>::Output as MapMut<
        StorageUnit<T>,
        T,
    >>::Output;
    type MultipleOutput =
        <<MutBorrowed<'a, U> as MapMut<(dyn Any), StorageUnit<T>>>::Output as MapMut<
            StorageUnit<T>,
            [T],
        >>::Output;
    type Owned = Box<T>;
    type MultipleOwned = Vec<T>;
    #[inline]
    fn get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output> {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&mut dyn Any) -> &mut StorageUnit<T> =
            &|x| x.downcast_mut::<StorageUnit<T>>().unwrap();
        let mut unit = MapMut::<dyn Any, StorageUnit<T>>::map(unit.storage_mut()?, f);
        unit.one_mut()?;
        let f: &dyn Fn(&mut StorageUnit<T>) -> &mut T = &|x| x.one_mut().unwrap();
        Ok(MapMut::<StorageUnit<T>, T>::map(unit, f))
    }
    #[inline]
    fn many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::MultipleOutput> {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&mut dyn Any) -> &mut StorageUnit<T> =
            &|x| x.downcast_mut::<StorageUnit<T>>().unwrap();
        let mut unit = MapMut::<dyn Any, StorageUnit<T>>::map(unit.storage_mut()?, f);
        unit.many_mut()?;
        let f: &dyn Fn(&mut StorageUnit<T>) -> &mut [T] = &|x| &mut x.many_mut().unwrap()[..];
        Ok(MapMut::<StorageUnit<T>, [T]>::map(unit, f))
    }
    #[inline]
    fn ind(boxed: &'a BlackBox<U>, ind: usize) -> DynamicResult<Self::Output> {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&mut dyn Any) -> &mut StorageUnit<T> =
            &|x| x.downcast_mut::<StorageUnit<T>>().unwrap();
        let mut unit = MapMut::<dyn Any, StorageUnit<T>>::map(unit.storage_mut()?, f);
        unit.many_mut()?
            .get_mut(ind)
            .ok_or(ErrorDesc::Unit(UnitError::OutOfBounds))?;
        let f: &dyn Fn(&mut StorageUnit<T>) -> &mut T = &move |x| &mut x.many_mut().unwrap()[ind];
        Ok(MapMut::<StorageUnit<T>, T>::map(unit, f))
    }
    #[inline]
    fn extract(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Owned> {
        let unit = boxed.unit_get::<T>()?;
        Ok(unit.extract()?.downcast::<T>().unwrap())
    }
    #[inline]
    fn extract_ind(boxed: &'a BlackBox<U>, ind: usize) -> DynamicResult<Self::Owned> {
        let unit = boxed.unit_get::<T>()?;
        Ok(unit.extract_ind(ind)?.downcast::<T>().unwrap())
    }
    #[inline]
    fn extract_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::MultipleOwned> {
        let unit = boxed.unit_get::<T>()?;
        Ok(*unit.extract_many()?.downcast::<Vec<T>>().unwrap())
    }
}

pub trait Multiple<'a, U: ?Sized> {
    type Output: 'a;
    fn get_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>;
}

pub trait SliceMany<'a, U: ?Sized>: Multiple<'a, U> {
    type SliceOutput: 'a;
    fn slice_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::SliceOutput>;
}

pub trait IndMultiple<'a, U: ?Sized>: Multiple<'a, U> {
    type Index;
    fn ind_many(
        boxed: &'a BlackBox<U>,
        index: Self::Index,
    ) -> DynamicResult<<Self as Multiple<'a, U>>::Output>;
}

pub trait Extract<'a, U: ?Sized>: Multiple<'a, U> {
    type Owned;
    fn extract(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Owned>;
}

pub trait ExtractInd<'a, U: ?Sized>: Multiple<'a, U> + Extract<'a, U> + IndMultiple<'a, U> {
    fn extract_ind(boxed: &'a BlackBox<U>, index: Self::Index) -> DynamicResult<Self::Owned>;
}

pub trait ExtractMultiple<'a, U: ?Sized>: Multiple<'a, U> {
    type MultipleOwned;
    fn extract_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::MultipleOwned>;
}

impl<'a, U: ?Sized, T: 'static> Multiple<'a, U> for &T
where
    &'static T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Output = <&'static T as Get<'a, U>>::Output;
    fn get_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output> {
        <&'static T>::get(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> Multiple<'a, U> for &mut T
where
    &'static mut T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Output = <&'static mut T as Get<'a, U>>::Output;
    fn get_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output> {
        <&'static mut T>::get(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> SliceMany<'a, U> for &T
where
    &'static T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type SliceOutput = <&'static T as Get<'a, U>>::MultipleOutput;
    fn slice_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::SliceOutput> {
        <&'static T>::many(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> SliceMany<'a, U> for &mut T
where
    &'static mut T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type SliceOutput = <&'static mut T as Get<'a, U>>::MultipleOutput;
    fn slice_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::SliceOutput> {
        <&'static mut T>::many(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> IndMultiple<'a, U> for &T
where
    &'static T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Index = usize;
    fn ind_many(boxed: &'a BlackBox<U>, index: usize) -> DynamicResult<Self::Output> {
        <&'static T>::ind(boxed, index)
    }
}

impl<'a, U: ?Sized, T: 'static> IndMultiple<'a, U> for &mut T
where
    &'static mut T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Index = usize;
    fn ind_many(boxed: &'a BlackBox<U>, index: usize) -> DynamicResult<Self::Output> {
        <&'static mut T>::ind(boxed, index)
    }
}

impl<'a, U: ?Sized, T: 'static> Extract<'a, U> for &T
where
    &'static T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Owned = <&'static T as Get<'a, U>>::Owned;
    fn extract(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Owned> {
        <&'static T as Get<'a, U>>::extract(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> Extract<'a, U> for &mut T
where
    &'static mut T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type Owned = <&'static mut T as Get<'a, U>>::Owned;
    fn extract(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Owned> {
        <&'static mut T as Get<'a, U>>::extract(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> ExtractInd<'a, U> for &T
where
    &'static T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    fn extract_ind(boxed: &'a BlackBox<U>, index: usize) -> DynamicResult<Self::Owned> {
        <&'static T as Get<'a, U>>::extract_ind(boxed, index)
    }
}

impl<'a, U: ?Sized, T: 'static> ExtractInd<'a, U> for &mut T
where
    &'static mut T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    fn extract_ind(boxed: &'a BlackBox<U>, index: usize) -> DynamicResult<Self::Owned> {
        <&'static mut T as Get<'a, U>>::extract_ind(boxed, index)
    }
}

impl<'a, U: ?Sized, T: 'static> ExtractMultiple<'a, U> for &T
where
    &'static T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type MultipleOwned = <&'static T as Get<'a, U>>::MultipleOwned;
    fn extract_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::MultipleOwned> {
        <&'static T as Get<'a, U>>::extract_many(boxed)
    }
}

impl<'a, U: ?Sized, T: 'static> ExtractMultiple<'a, U> for &mut T
where
    &'static mut T: Get<'a, U>,
    U: Unit<'a, Owned = Box<dyn Any>>,
{
    type MultipleOwned = <&'static mut T as Get<'a, U>>::MultipleOwned;
    fn extract_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::MultipleOwned> {
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
            fn get_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output> {
                Ok(($first_type::get(boxed)?, $($typ::get(boxed)?),*))
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
            fn slice_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::SliceOutput> {
                Ok(($first_type::many(boxed)?, $($typ::many(boxed)?),*))
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
            fn ind_many(boxed: &'a BlackBox<U>, ($first_ind, $($ind),*): Self::Index) -> DynamicResult<Self::Output> {
                Ok(($first_type::ind(boxed, $first_ind)?, $($typ::ind(boxed, $ind)?),*))
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
            fn extract(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Owned> {
                Ok(($first_type::extract(boxed)?, $($typ::extract(boxed)?),*))
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
            fn extract_ind(boxed: &'a BlackBox<U>, ($first_ind, $($ind),*): Self::Index) -> DynamicResult<<Self as Extract<'a, U>>::Owned> {
                Ok(($first_type::extract_ind(boxed, $first_ind)?, $($typ::extract_ind(boxed, $ind)?),*))
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
            fn extract_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::MultipleOwned> {
                Ok(($first_type::extract_many(boxed)?, $($typ::extract_many(boxed)?),*))
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
