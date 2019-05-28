#[doc(hidden)]
#[macro_export]
macro_rules! impl_unit {
    ($name:ident, $traitobject:ty, ($($constraint:tt)*), $storage_wrapper:ident, $mutlock:ident, $unmutlock:ident, $internal:ident) => {
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
            #[doc = "Returns whether the current storage contains a unit for a \
            given type.\nPlease refer to the proper documentation for this \
            function at [`BlackBox::has_unit`]."]
            #[inline(always)]
            pub fn has_unit<T: $($constraint)*>(&self) -> bool {
                self.$internal
                    .has_unit::<T>()
            }
            #[doc = "Inserts a value whose type is constrained into the internal storage \
            unit. This will either append or fill a unit, going from `Nope` to \
            `One`, `One` to `Many` or appending `Many`.\nPlease refer to the proper \
             documentation for this function at [`BlackBox::insert`]."]
            #[inline(always)]
            pub fn insert<T: $($constraint)*>(&self, data: T) -> Result<(), (T, $crate::ErrorDesc)> {
                self.$internal
                    .insert(data)
            }
            #[doc = "Inserts many values of homogeneous types within a [`Vec`]. This will \
            append to a previously `Many` set of values, append to the end of a `One` value \
            or replace a `Nope` value.\nPlease refer to the proper documentation for \
            function at [`BlackBox::insert_many`]."]
            #[inline(always)]
            pub fn insert_many<T: $($constraint)*>(&self, data: Vec<T>) -> Result<(), (Vec<T>, $crate::black_box::ErrorDesc)> {
                self.$internal
                    .insert_many(data)
            }

            #[inline(always)]
            pub fn get_mut<T: $($constraint)*>(&self) -> $crate::black_box::DynamicResult<$mutlock<T>> {
                self.$internal
                    .get_mut()
            }

            #[inline(always)]
            pub fn ind_mut<T: $($constraint)*>(&self, ind: usize) -> $crate::black_box::DynamicResult<$mutlock<T>> {
                self.$internal
                    .ind_mut(ind)
            }

            #[inline(always)]
            pub fn extract<T: $($constraint)*>(&self) -> $crate::black_box::DynamicResult<T> {
                self.$internal
                    .extract()
            }

            #[inline(always)]
            pub fn extract_many<T: $($constraint)*>(&self) -> $crate::black_box::DynamicResult<Vec<T>> {
                self.$internal
                    .extract_many()
            }

            #[inline(always)]
            pub fn run_for<
                'a,
                T: $($constraint)*,
                D: 'static + Any,
                F: FnMut(DynamicResult<&[T]>) -> D + 'a,
            >(
                &self,
                f: F,
            ) -> $crate::black_box::DynamicResult<D> {
                self.$internal
                    .run_for(f)
            }

            #[inline(always)]
            pub fn run_for_mut<
                'a,
                T: $($constraint)*,
                D: 'static + Any,
                F: FnMut($crate::black_box::DynamicResult<&mut Vec<T>>) -> D + 'a>(&self, f: F) -> $crate::black_box::DynamicResult<D> {
                self.$internal
                    .run_for_mut(f)
            }
        }
    };
    ($name:ident, $traitobject:ty, ($($constraint:tt)*), $storage_wrapper:ident, $mutlock:ident, $unmutlock:ident, $internal:ident, add_unmut) => {
        $crate::impl_unit!($name, $traitobject, ($($constraint)*), $storage_wrapper, $mutlock, $unmutlock, $internal);
        impl $name {
            #[inline(always)]
            pub fn get<T: $($constraint)*>(&self) -> $crate::black_box::DynamicResult<$unmutlock<T>> {
                self.$internal
                    .get()
            }

            #[inline(always)]
            pub fn ind<T: $($constraint)*>(&self, ind: usize) -> $crate::black_box::DynamicResult<$unmutlock<T>> {
                self.$internal
                    .ind(ind)
            }
        }
    };
}

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
}
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
}
