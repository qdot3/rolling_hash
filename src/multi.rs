pub struct HashCount<const N: usize>;

pub trait SupportedHashCount {}

macro_rules! supported_hash_count_impl {
    () => {};
    ( $first:literal $( $rest:literal )*) => {
        impl SupportedHashCount for HashCount<$first> {}

        supported_hash_count_impl!($( $rest )*);
    };
}
supported_hash_count_impl!(1 2 3 4 5 6 7 8 9 10);

pub struct RollingHash<const N: usize>
where
    HashCount<N>: SupportedHashCount,
{
    primes: [u64; N],
    bases: [u64; N],
    hashed: [Vec<u64>; N],
}

impl<const N: usize> RollingHash<N>
where
    HashCount<N>: SupportedHashCount,
{
    pub fn find(&self, sub_slice: &[u64]) -> Option<usize> {
        todo!()
    }
}
