use std::hash::{BuildHasher, Hasher};

#[derive(Clone, Copy, Default)]
pub struct PassthroughHasherBuilder;

#[derive(Clone, Copy)]
pub struct PassthroughHasher(u64);

impl BuildHasher for PassthroughHasherBuilder {
    type Hasher = PassthroughHasher;
    fn build_hasher(&self) -> PassthroughHasher {
        PassthroughHasher(0)
    }
}

impl Hasher for PassthroughHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        assert!(bytes.len() >= 8);
        unsafe {
            self.0 = *(bytes.as_ptr() as *const u64);
        }
    }
}
