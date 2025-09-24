/// A hashable type.
pub trait RollingHash {
    fn write();

    fn hash_slice(&self);
}
