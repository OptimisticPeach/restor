#[doc(hidden)]
#[macro_export]
macro_rules! impl_unit {
    ($name:ident, $traitobject:ty, ($($constraint:tt)*), $storage_wrapper:ident, $mutlock:ident, $unmutlock:ident, $internal:ident) => {
        impl $name {
            pub fn allocate_for<T: $($constraint)*>(&mut self) {
                self.$internal
                    .data
                    .entry(::std::any::TypeId::of::<T>())
                    .or_insert_with(|| Box::new($storage_wrapper::new($crate::black_box::StorageUnit::<T>::new())));
            }

            pub fn has_unit<T: $($constraint)*>(&self) -> bool {
                self.$internal
                    .has_unit::<T>()
            }

            pub fn insert<T: $($constraint)*>(&self, data: T) -> Result<(), (T, $crate::ErrorDesc)> {
                self.$internal
                    .insert(data)
            }

            pub fn insert_many<T: $($constraint)*>(&self, data: Vec<T>) -> Result<(), (Vec<T>, $crate::black_box::ErrorDesc)> {
                self.$internal
                    .insert_many(data)
            }

            pub fn get_mut<T: $($constraint)*>(&self) -> $crate::black_box::DynamicResult<$mutlock<T>> {
                self.$internal
                    .get_mut()
            }

            pub fn ind_mut<T: $($constraint)*>(&self, ind: usize) -> $crate::black_box::DynamicResult<$mutlock<T>> {
                self.$internal
                    .ind_mut(ind)
            }

            pub fn extract<T: $($constraint)*>(&self) -> $crate::black_box::DynamicResult<T> {
                self.$internal
                    .extract()
            }

            pub fn extract_many<T: $($constraint)*>(&self) -> $crate::black_box::DynamicResult<Vec<T>> {
                self.$internal
                    .extract_many()
            }

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
        }
    };
    ($name:ident, $traitobject:ty, ($($constraint:tt)*), $storage_wrapper:ident, $mutlock:ident, $unmutlock:ident, $internal:ident, add_unmut) => {
        impl $name {
            pub fn allocate_for<T: $($constraint)*>(&mut self) {
                self.$internal
                    .data
                    .entry(::std::any::TypeId::of::<T>())
                    .or_insert_with(|| Box::new($storage_wrapper::new($crate::black_box::StorageUnit::<T>::new())));
            }

            pub fn has_unit<T: $($constraint)*>(&self) -> bool {
                self.$internal
                    .has_unit::<T>()
            }

            pub fn insert<T: $($constraint)*>(&self, data: T) -> Result<(), (T, $crate::ErrorDesc)> {
                self.$internal
                    .insert(data)
            }

            pub fn insert_many<T: $($constraint)*>(&self, data: Vec<T>) -> Result<(), (Vec<T>, $crate::black_box::ErrorDesc)> {
                self.$internal
                    .insert_many(data)
            }

            pub fn get_mut<T: $($constraint)*>(&self) -> $crate::black_box::DynamicResult<$mutlock<T>> {
                self.$internal
                    .get_mut()
            }

            pub fn ind_mut<T: $($constraint)*>(&self, ind: usize) -> $crate::black_box::DynamicResult<$mutlock<T>> {
                self.$internal
                    .ind_mut(ind)
            }

            pub fn extract<T: $($constraint)*>(&self) -> $crate::black_box::DynamicResult<T> {
                self.$internal
                    .extract()
            }

            pub fn extract_many<T: $($constraint)*>(&self) -> $crate::black_box::DynamicResult<Vec<T>> {
                self.$internal
                    .extract_many()
            }

            pub fn get<T: $($constraint)*>(&self) -> $crate::black_box::DynamicResult<$unmutlock<T>> {
                self.$internal
                    .get()
            }

            pub fn ind<T: $($constraint)*>(&self, ind: usize) -> $crate::black_box::DynamicResult<$unmutlock<T>> {
                self.$internal
                    .ind(ind)
            }

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
        }
    };
}
