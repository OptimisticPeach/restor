use super::{
    BlackBox, Borrowed, DynamicResult, Map, MapMut, MutBorrowed, StorageUnit, Unit, Waitable
};
use std::any::Any;

///
/// The base "get" trait for acquiring data from storage. This is implemented on
/// six types, each of which have a different output. The output is dependent on
/// the type it is being implemented for.
///
/// Note that this trait should be considered "sealed" as it is already implemented
/// for all the types it should be implemented for.
///
pub trait Fetch<'a, U: Unit<'a> + ?Sized> {
    ///
    /// The type output for `Self`.
    ///
    type Output: 'a;
    ///
    /// A type exposed for type checking on the contents of the returned lock.
    ///
    type Actual;
    ///
    /// Gets data from the [`BlackBox`](./struct.BlackBox.html) depending on `Self` and `Output`.
    ///
    fn get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>;
    ///
    /// Gets data from the [`BlackBox`](./struct.BlackBox.html) depending on `Self` and `Output`.
    /// This function waits on availability for the lock.
    ///
    fn waiting_get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>
    where
        Borrowed<'a, U>: Waitable,
        MutBorrowed<'a, U>: Waitable;
}

//Single value immutable
impl<'a, T: Sized + Any + 'static, U: for<'b> Unit<'b> + ?Sized> Fetch<'a, U> for &T
where
    Borrowed<'a, U>: Map<(dyn Any), StorageUnit<T>, Func = dyn Fn(&dyn Any) -> &StorageUnit<T>>,
    <Borrowed<'a, U> as Map<dyn Any, StorageUnit<T>>>::Output:
        Map<StorageUnit<T>, T, Func = dyn Fn(&StorageUnit<T>) -> &T>,
{
    type Output = <<Borrowed<'a, U> as Map<dyn Any, StorageUnit<T>>>::Output as Map<
        StorageUnit<T>,
        T,
    >>::Output;

    type Actual = T;
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
    fn waiting_get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>
    where
        Borrowed<'a, U>: Waitable,
        MutBorrowed<'a, U>: Waitable,
    {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&dyn Any) -> &StorageUnit<T> =
            &|x| x.downcast_ref::<StorageUnit<T>>().unwrap();
        let unit = Map::<dyn Any, StorageUnit<T>>::map(unit.waiting_storage(), f);
        unit.one()?;
        let f: &dyn for<'r> Fn(&'r StorageUnit<T>) -> &'r T = &|x| x.one().unwrap();
        Ok(Map::<StorageUnit<T>, T>::map(unit, f))
    }
}

//Single value mutable
impl<'a, T: Sized + Any + 'static, U: for<'b> Unit<'b> + ?Sized> Fetch<'a, U> for &mut T
where
    MutBorrowed<'a, U>:
        MapMut<(dyn Any), StorageUnit<T>, Func = dyn Fn(&mut dyn Any) -> &mut StorageUnit<T>>,
    <MutBorrowed<'a, U> as MapMut<(dyn Any), StorageUnit<T>>>::Output:
        MapMut<StorageUnit<T>, T, Func = dyn Fn(&mut StorageUnit<T>) -> &mut T>,
{
    type Output = <<MutBorrowed<'a, U> as MapMut<(dyn Any), StorageUnit<T>>>::Output as MapMut<
        StorageUnit<T>,
        T,
    >>::Output;

    type Actual = T;
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
    fn waiting_get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>
    where
        Borrowed<'a, U>: Waitable,
        MutBorrowed<'a, U>: Waitable,
    {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&mut dyn Any) -> &mut StorageUnit<T> =
            &|x| x.downcast_mut::<StorageUnit<T>>().unwrap();
        let mut unit = MapMut::<dyn Any, StorageUnit<T>>::map(unit.waiting_storage_mut(), f);
        unit.one_mut()?;
        let f: &dyn Fn(&mut StorageUnit<T>) -> &mut T = &|x| x.one_mut().unwrap();
        Ok(MapMut::<StorageUnit<T>, T>::map(unit, f))
    }
}

//Slice immutable
impl<'a, T: Sized + Any + 'static, U: for<'b> Unit<'b> + ?Sized> Fetch<'a, U> for &[T]
where
    Borrowed<'a, U>: Map<(dyn Any), StorageUnit<T>, Func = dyn Fn(&dyn Any) -> &StorageUnit<T>>,
    <Borrowed<'a, U> as Map<dyn Any, StorageUnit<T>>>::Output:
        Map<StorageUnit<T>, [T], Func = dyn Fn(&StorageUnit<T>) -> &[T]>,
{
    type Output = <<Borrowed<'a, U> as Map<dyn Any, StorageUnit<T>>>::Output as Map<
        StorageUnit<T>,
        [T],
    >>::Output;

    type Actual = T;
    #[inline]
    fn get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output> {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&dyn Any) -> &StorageUnit<T> =
            &|x| x.downcast_ref::<StorageUnit<T>>().unwrap();
        let unit = Map::<dyn Any, StorageUnit<T>>::map(unit.storage()?, f);
        unit.many()?;
        let f: &dyn for<'r> Fn(&'r StorageUnit<T>) -> &'r [T] = &|x| x.many().unwrap();
        Ok(Map::<StorageUnit<T>, [T]>::map(unit, f))
    }
    #[inline]
    fn waiting_get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>
    where
        Borrowed<'a, U>: Waitable,
        MutBorrowed<'a, U>: Waitable,
    {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&dyn Any) -> &StorageUnit<T> =
            &|x| x.downcast_ref::<StorageUnit<T>>().unwrap();
        let unit = Map::<dyn Any, StorageUnit<T>>::map(unit.waiting_storage(), f);
        unit.many()?;
        let f: &dyn for<'r> Fn(&'r StorageUnit<T>) -> &'r [T] = &|x| x.many().unwrap();
        Ok(Map::<StorageUnit<T>, [T]>::map(unit, f))
    }
}

//Slice mutable
impl<'a, T: Sized + Any + 'static, U: for<'b> Unit<'b> + ?Sized> Fetch<'a, U> for &mut [T]
where
    MutBorrowed<'a, U>:
        MapMut<(dyn Any), StorageUnit<T>, Func = dyn Fn(&mut dyn Any) -> &mut StorageUnit<T>>,
    <MutBorrowed<'a, U> as MapMut<(dyn Any), StorageUnit<T>>>::Output:
        MapMut<StorageUnit<T>, [T], Func = dyn Fn(&mut StorageUnit<T>) -> &mut [T]>,
{
    type Output = <<MutBorrowed<'a, U> as MapMut<(dyn Any), StorageUnit<T>>>::Output as MapMut<
        StorageUnit<T>,
        [T],
    >>::Output;

    type Actual = T;
    #[inline]
    fn get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output> {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&mut dyn Any) -> &mut StorageUnit<T> =
            &|x| x.downcast_mut::<StorageUnit<T>>().unwrap();
        let mut unit = MapMut::<dyn Any, StorageUnit<T>>::map(unit.storage_mut()?, f);
        unit.many_mut()?;
        let f: &dyn Fn(&mut StorageUnit<T>) -> &mut [T] = &|x| x.many_mut().unwrap();
        Ok(MapMut::<StorageUnit<T>, [T]>::map(unit, f))
    }
    #[inline]
    fn waiting_get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>
    where
        Borrowed<'a, U>: Waitable,
        MutBorrowed<'a, U>: Waitable,
    {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&mut dyn Any) -> &mut StorageUnit<T> =
            &|x| x.downcast_mut::<StorageUnit<T>>().unwrap();
        let mut unit = MapMut::<dyn Any, StorageUnit<T>>::map(unit.waiting_storage_mut(), f);
        unit.many_mut()?;
        let f: &dyn Fn(&mut StorageUnit<T>) -> &mut [T] = &|x| x.many_mut().unwrap();
        Ok(MapMut::<StorageUnit<T>, [T]>::map(unit, f))
    }
}

//Own single
impl<'a, T: Sized + Any + 'static, U: for<'b> Unit<'b> + ?Sized> Fetch<'a, U> for Box<T>
where
    MutBorrowed<'a, U>:
        MapMut<(dyn Any), StorageUnit<T>, Func = dyn Fn(&mut dyn Any) -> &mut StorageUnit<T>>,
{
    type Output = T;

    type Actual = T;

    #[inline]
    fn get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output> {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&mut dyn Any) -> &mut StorageUnit<T> =
            &|x| x.downcast_mut::<StorageUnit<T>>().unwrap();
        let mut unit = MapMut::<dyn Any, StorageUnit<T>>::map(unit.storage_mut()?, f);
        unit.extract_one()
    }
    #[inline]
    fn waiting_get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>
    where
        Borrowed<'a, U>: Waitable,
        MutBorrowed<'a, U>: Waitable,
    {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&mut dyn Any) -> &mut StorageUnit<T> =
            &|x| x.downcast_mut::<StorageUnit<T>>().unwrap();
        let mut unit = MapMut::<dyn Any, StorageUnit<T>>::map(unit.waiting_storage_mut(), f);
        unit.extract_one()
    }
}

//Own many
impl<'a, T: Sized + Any + 'static, U: for<'b> Unit<'b> + ?Sized> Fetch<'a, U> for Vec<T>
where
    MutBorrowed<'a, U>:
        MapMut<(dyn Any), StorageUnit<T>, Func = dyn Fn(&mut dyn Any) -> &mut StorageUnit<T>>,
{
    type Output = Vec<T>;

    type Actual = T;
    #[inline]
    fn get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output> {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&mut dyn Any) -> &mut StorageUnit<T> =
            &|x| x.downcast_mut::<StorageUnit<T>>().unwrap();
        let mut unit = MapMut::<dyn Any, StorageUnit<T>>::map(unit.storage_mut()?, f);
        unit.extract_many()
    }
    #[inline]
    fn waiting_get(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>
    where
        Borrowed<'a, U>: Waitable,
        MutBorrowed<'a, U>: Waitable,
    {
        let unit = boxed.unit_get::<T>()?;
        let f: &dyn Fn(&mut dyn Any) -> &mut StorageUnit<T> =
            &|x| x.downcast_mut::<StorageUnit<T>>().unwrap();
        let mut unit = MapMut::<dyn Any, StorageUnit<T>>::map(unit.waiting_storage_mut(), f);
        unit.extract_many()
    }
}

///
/// An abstraction over [`Fetch`] which works over multiple types, and the
/// six types which have [`Fetch`] pre-implemented. This is therefore implemented
/// for the following types:
///
/// - `&T`
/// - `&mut T`
/// - `&[T]`
/// - `&mut [T]`
/// - `Box<T>`
/// - `Vec<T>`
/// - `(A,)`
/// - `(A, B)`
/// - `(A, B, C)`
/// - `(A, B, C, D)`
/// - ...
/// - `(A, B, C, D, E, F, G, H, I, J, K)`
///
/// Where each one of the type parameters in the tuple versions must implement
/// [`Fetch`].
///
/// [`Fetch`]: ./trait.Fetch.html
///
pub trait FetchMultiple<'a, U: ?Sized + Unit<'a>> {
    type Output: 'a;
    type Actual;
    fn get_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>;
    fn waiting_get_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>
    where
        Borrowed<'a, U>: Waitable,
        MutBorrowed<'a, U>: Waitable;
}

//TODO: Make this less atrocious
macro_rules! impl_single {
    () => {};
    (($first:ty, $fmap:ident, ($($f_constraints:tt)+)) $(, ($typed:ty, $map:ident, ($($constraints:tt)+)))*) => {
        //One
        impl<'a, U: ?Sized, T: Sized + Any + 'static> FetchMultiple<'a, U> for $first
        where
            U: for<'b> Unit<'b>,
            $($f_constraints)+
            $first: Fetch<'a, U>,
        {
            type Output = <$first as Fetch<'a, U>>::Output;
            type Actual = <$first as Fetch<'a, U>>::Actual;
            #[inline]
            fn get_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output> {
                <$first>::get(boxed)
            }
            #[inline]
            fn waiting_get_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>
            where Borrowed<'a, U>: Waitable, MutBorrowed<'a, U>: Waitable {
                <$first>::waiting_get(boxed)
            }
        }

        impl_single!($(($typed, $map, ($($constraints)+))),*);
    };
}

impl_single!(
    (
        &'a T, Map, (
            Borrowed<'a, U>: Map<dyn Any, StorageUnit<T>, Func = dyn Fn(&dyn Any) -> &StorageUnit<T>>,
            <Borrowed<'a, U> as Map<dyn Any, StorageUnit<T>>>::Output: Map<StorageUnit<T>, T, Func = dyn Fn(&StorageUnit<T>) -> &T>,
        )
    ),
    (
        &'a mut T, MapMut, (
            MutBorrowed<'a, U>: MapMut<dyn Any, StorageUnit<T>, Func = dyn Fn(&mut dyn Any) -> &mut StorageUnit<T>>,
            <MutBorrowed<'a, U> as MapMut<dyn Any, StorageUnit<T>>>::Output: MapMut<StorageUnit<T>, T, Func = dyn Fn(&mut StorageUnit<T>) -> &mut T>,
        )
    ),
    (
        &'a [T], Map, (
            Borrowed<'a, U>: Map<dyn Any, StorageUnit<T>, Func = dyn Fn(&dyn Any) -> &StorageUnit<T>>,
            <Borrowed<'a, U> as Map<dyn Any, StorageUnit<T>>>::Output: Map<StorageUnit<T>, [T], Func = dyn Fn(&StorageUnit<T>) -> &[T]>,
        )
    ),
    (
        &'a mut [T], MapMut, (
            MutBorrowed<'a, U>: MapMut<dyn Any, StorageUnit<T>, Func = dyn Fn(&mut dyn Any) -> &mut StorageUnit<T>>,
            <MutBorrowed<'a, U> as MapMut<dyn Any, StorageUnit<T>>>::Output: MapMut<StorageUnit<T>, [T], Func = dyn Fn(&mut StorageUnit<T>) -> &mut [T]>,
        )
    ),
    (
        Box<T>, MapMut, (
            MutBorrowed<'a, U>: MapMut<dyn Any, StorageUnit<T>, Func = dyn Fn(&mut dyn Any) -> &mut StorageUnit<T>>,
        )
    ),
    (
        Vec<T>, MapMut, (
            MutBorrowed<'a, U>: MapMut<dyn Any, StorageUnit<T>, Func = dyn Fn(&mut dyn Any) -> &mut StorageUnit<T>>,
        )
    )
);

macro_rules! impl_tuple {
    () => {};
    ($first_type:ident $(, $typ:ident)* ) => {
        impl<'a, U: ?Sized, $first_type, $($typ),*> FetchMultiple<'a, U> for ($first_type, $($typ),*)
        where
            $(
                $typ: Fetch<'a, U>,
            )*
            $first_type: Fetch<'a, U>,
            U: Unit<'a>,
        {
            type Output = ($first_type::Output, $($typ::Output),*);
            type Actual = ($first_type::Actual, $($typ::Actual),*);
            #[inline]
            fn get_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output> {
                Ok(($first_type::get(boxed)?, $($typ::get(boxed)?),*))
            }
            #[inline]
            fn waiting_get_many(boxed: &'a BlackBox<U>) -> DynamicResult<Self::Output>
            where Borrowed<'a, U>: Waitable, MutBorrowed<'a, U>: Waitable{
                Ok(($first_type::waiting_get(boxed)?, $($typ::waiting_get(boxed)?),*))
            }
        }

        impl_tuple!($($typ),*);
    }
}

impl_tuple!(A, B, C, D, E, F, G, H, I, J, K);

#[cfg(test)]
mod tests {
    #![allow(unused)]
    use lazy_static::*;
    use crate::{make_storage, RwLockStorage};
    lazy_static!{
        static ref storage: RwLockStorage = make_storage!(RwLockStorage: GetOneType,
        many_types::Type0,
        many_types::Type1,
        many_types::Type2,
        many_types::Type3,
        many_types::Type4,
        many_types::Type5);
    }
    #[derive(Clone, Copy, Debug)]
    struct GetOneType;
    #[test]
    fn get_one_type() {
        storage.insert(GetOneType).unwrap();
        {
            storage.get::<&GetOneType>().unwrap();
        }
        {
            storage.get::<&mut GetOneType>().unwrap();
        }
        {
            storage.get::<Box<GetOneType>>().unwrap();
        }
        storage.insert_many(vec![GetOneType; 20]).unwrap();
        {
            storage.get::<&[GetOneType]>().unwrap();
        }
        {
            storage.get::<&mut [GetOneType]>().unwrap();
        }
        {
            storage.get::<Vec<GetOneType>>().unwrap();
        }
    }
    mod many_types {
        #[derive(Clone, Copy, Debug)]
        pub struct Type0;

        #[derive(Clone, Copy, Debug)]
        pub struct Type1;

        #[derive(Clone, Copy, Debug)]
        pub struct Type2;

        #[derive(Clone, Copy, Debug)]
        pub struct Type3;

        #[derive(Clone, Copy, Debug)]
        pub struct Type4;

        #[derive(Clone, Copy, Debug)]
        pub struct Type5;
    }
    #[test]
    fn get_many_types() {
        use many_types::*;
        storage.insert(Type0).unwrap();
        storage.insert(Type1).unwrap();
        storage.insert(Type2).unwrap();
        storage.insert_many(vec![Type3; 10]).unwrap();
        storage.insert_many(vec![Type4; 10]).unwrap();
        storage.insert_many(vec![Type5; 10]).unwrap();

        let (t0, t1, t2, t3, t4, t5) =
            storage.get::<(
                &Type0,
                &mut Type1,
                Box<Type2>,
                &[Type3],
                &mut [Type4],
                Vec<Type5>
            )>().unwrap();
    }
}
