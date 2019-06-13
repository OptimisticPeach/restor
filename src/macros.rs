#[doc(hidden)]
#[macro_export]
macro_rules! impl_unit {
    ($name:ident, $traitobject:ty, ($($constraint:tt)*), $storage_wrapper:ident($unit:ty$(,)?), $mutlock:ident, $unmutlock:ident, $internal:ident) => {
        impl $name {
            #[doc = "Adds a storage unit for the given type.\n\
            This will not add another unit in the case that it already exists.\n\n"]
            #[inline(always)]
            pub fn allocate_for<T: $($constraint)*>(&mut self) {
                self.$internal
                    .data
                    .entry(::std::any::TypeId::of::<T>())
                    .or_insert_with(|| Box::new($storage_wrapper::new($crate::black_box::StorageUnit::<T>::new())));
            }
            #[doc = "Please refer to the documentation for this function at [`BlackBox::has_unit`]."]
            #[inline(always)]
            pub fn has_unit<T: $($constraint)*>(&self) -> bool {
                self.$internal
                    .has_unit::<T>()
            }
            #[doc = "Please refer to the documentation for this function at [`BlackBox::insert`]."]
            #[inline(always)]
            pub fn insert<T: $($constraint)*>(&self, data: T) -> Result<(), (T, $crate::ErrorDesc)> {
                self.$internal
                    .insert(data)
            }
            #[doc = "Please refer to the documentation for this function at [`BlackBox::insert_many`]."]
            #[inline(always)]
            pub fn insert_many<T: $($constraint)*>(&self, data: Vec<T>) -> Result<(), (Vec<T>, $crate::black_box::ErrorDesc)> {
                self.$internal
                    .insert_many(data)
            }

            #[doc = "Please refer to the documentation for this function at [`BlackBox::run_for_mut`]."]
            #[inline(always)]
            pub fn run_for_mut<
                T: $($constraint)*,
                D: 'static + Any,
                F: FnMut(&mut Vec<T>) -> D
            >(
                &self,
                f: F
            ) -> $crate::black_box::DynamicResult<D> {
                self.$internal
                    .run_for_mut(f)
            }

            #[doc = "Please refer to the documentation for this function at [`BlackBox::get`]."]
            #[inline(always)]
            pub fn get<
                'a,
                T: $crate::FetchMultiple<'a, $unit>,
            > (&'a self) -> $crate::black_box::DynamicResult<T::Output>
            where <T as $crate::FetchMultiple<'a, $unit>>::Actual: $($constraint)*{
                self.$internal
                    .get::<T>()
            }
        }
    };
    ($name:ident, $traitobject:ty, ($($constraint:tt)*), $storage_wrapper:ident($unit:ty$(,)?), $mutlock:ident, $unmutlock:ident, $internal:ident, add_unmut $(, $($rest:tt)*)?) => {
        $crate::impl_unit!($name, $traitobject, ($($constraint)*), $storage_wrapper($unit), $mutlock, $unmutlock, $internal $(, $( $rest)*)?);
        impl $name {
            #[inline(always)]
            pub fn run_for<
                T: $($constraint)*,
                D: 'static + Any,
                F: FnMut(&[T]) -> D,
            >(
                &self,
                f: F,
            ) -> $crate::black_box::DynamicResult<D> {
                self.$internal
                    .run_for(f)
            }
        }
    };
    ($name:ident, $traitobject:ty, ($($constraint:tt)*), $storage_wrapper:ident($unit:ty$(,)?), $mutlock:ident, $unmutlock:ident, $internal:ident, add_waiting $(, $($rest:tt)*)?) => {
        $crate::impl_unit!($name, $traitobject, ($($constraint)*), $storage_wrapper($unit), $mutlock, $unmutlock, $internal $(, $( $rest)*)?);
        impl $name {
            #[doc = "Please refer to the documentation for this function at [`BlackBox::waiting_get`]."]
            #[inline(always)]
            pub fn waiting_get<'a, T: $crate::black_box::FetchMultiple<'a, $unit>>(&'a self) -> $crate::black_box::DynamicResult<T::Output>
            where
                $crate::black_box::Borrowed<'a, $unit>: $crate::black_box::Waitable,
                $crate::black_box::MutBorrowed<'a, $unit>: $crate::black_box::Waitable,
                <T as $crate::FetchMultiple<'a, $unit>>::Actual: $($constraint)*
            {
                self.$internal
                    .waiting_get::<T>()
            }
        }
    };
}

///
/// A shorthand for unwrapping a `Result` into an `Ok(x)`.
///
/// The following syntaxes do the following:
///
/// - `(expr)` -> `x`
/// - `(expr, val)` -> `x == val; x`
/// - `(expr, val, *)` -> `*x == val; x`
/// - `(expr, val, [ind])` -> `x[ind] == val; x`
///
#[macro_export]
macro_rules! ok {
    ($e:expr) => {
        match $e {
            Ok(x) => x,
            Err(e) => panic!("Expected `Ok` but instead found `Err({:?})`", e),
        }
    };
    ($e:expr, $other:expr) => {
        match $e {
            Ok(x) => {
                assert_eq!(x, $other);
                x
            }
            Err(e) => panic!("Expected `Ok` but instead found `Err({:?})`", e),
        }
    };
    ($e:expr, $other:expr, *) => {
        match $e {
            Ok(x) => {
                assert_eq!(*x, $other);
                x
            }
            Err(e) => panic!("Expected `Ok` but instead found `Err({:?})`", e),
        }
    };
    ($e:expr, $other:expr, [$i:expr]) => {
        match $e {
            Ok(x) => {
                assert_eq!(x[$i], $other);
                x
            }
            Err(e) => panic!("Expected `Ok` but instead found `Err({:?})`", e),
        }
    };
}

///
/// A shorthand for unwrapping a `Result` into an `Err(x)`.
///
/// The following syntaxes do the following:
///
/// - `(expr)` -> `x`
/// - `(expr, val)` -> `x == val; x`
/// - `(expr, val, *)` -> `*x == val; x`
/// - `(expr, val, [ind])` -> `x[ind] == val; x`
///
#[macro_export]
macro_rules! err {
    ($e:expr) => {
        match $e {
            Ok(_) => panic!("Expected `Err` but instead found `Ok(_)`"),
            Err(x) => x,
        }
    };
    ($e:expr, $other:expr) => {
        match $e {
            Ok(x) => panic!("Expected `Err` but instead found `Ok(_)`"),
            Err(e) => {
                assert_eq!(e, $other);
                e
            }
        }
    };
    ($e:expr, $other:expr, *) => {
        match $e {
            Ok(x) => panic!("Expected `Err` but instead found `Ok(_)`"),
            Err(e) => {
                assert_eq!(*e, $other);
                e
            }
        }
    };
    ($e:expr, $other:expr, [i]) => {
        match $e {
            Ok(x) => panic!("Expected `Err` but instead found `Ok(_)`"),
            Err(e) => {
                assert_eq!(e[i], $other);
                e
            }
        }
    };
}
