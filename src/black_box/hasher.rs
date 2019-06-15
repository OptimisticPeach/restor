use std::hash::{BuildHasher, Hasher};

#[derive(Clone, Copy, Default)]
pub struct PassthroughHasherBuilder;

#[derive(Clone, Copy)]
pub struct PassthroughHasher(u64);

impl BuildHasher for PassthroughHasherBuilder {
    type Hasher = PassthroughHasher;
    #[inline(always)]
    fn build_hasher(&self) -> PassthroughHasher {
        PassthroughHasher(0)
    }
}

impl Hasher for PassthroughHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.0
    }
    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        assert!(bytes.len() >= 8);
        unsafe {
            self.0 = *(bytes.as_ptr() as *const u64);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::black_box::hasher::PassthroughHasherBuilder;
    use std::hash::{BuildHasher, Hash, Hasher};

    #[test]
    pub fn passthrough_hasher() {
        for i in 0u64..0xffffu64 {
            let mut hasher = PassthroughHasherBuilder.build_hasher();
            i.hash(&mut hasher);
            assert_eq!(hasher.finish(), i);
        }
    }
}
