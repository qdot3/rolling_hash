/// A hashable type.
pub trait RollingHash {
    fn write(&self);

    fn hash_slice(&self);
}
