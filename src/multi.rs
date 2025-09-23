//! 名前を変える
pub struct PrimeCount<const N: usize>;

pub trait SupportedPrimeCount {}

macro_rules! supported_hash_count_impl {
    () => {};
    ( $first:literal $( $rest:literal )*) => {
        impl SupportedPrimeCount for PrimeCount<$first> {}

        supported_hash_count_impl!($( $rest )*);
    };
}
supported_hash_count_impl!(1 2 3 4 5 6 7 8 9 10);

pub struct RollingHash<const N: usize>
where
    PrimeCount<N>: SupportedPrimeCount,
{
    primes: [u64; N],
    bases: [u64; N],
    hashed: [Vec<u64>; N],
}

impl<const N: usize> RollingHash<N>
where
    PrimeCount<N>: SupportedPrimeCount,
{
    pub fn find(&self, sub_slice: &[u64]) -> Option<usize> {
        todo!()
    }
}
