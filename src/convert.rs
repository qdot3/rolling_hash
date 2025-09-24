use crate::{Prime, SupportedPrime};

/// Reduces `v` to `v % P`.
pub trait Reduce<const P: u64>
where
    Prime<P>: SupportedPrime,
{
    type Item;

    /// Reduces `v` to `v % P`
    fn reduce(&self) -> u64;

    fn into_item(&self) -> Self::Item;
}

macro_rules! reduce_small_unsigned_impl {
    ($( $t:ty ),*) => {$(
        impl<const P: u64> Reduce<P> for $t
        where
            Prime<P>: SupportedPrime,
        {
            type Item = $t;

            fn reduce(&self) -> u64 { *self as u64 }
            fn into_item(&self) -> Self::Item { *self }
        }
    )*};
}
reduce_small_unsigned_impl!(u8, u16, u32, char, bool);

macro_rules! reduce_large_unsigned_impl {
    ($( $t:ty ),*) => {$(
        impl<const P: u64> Reduce<P> for $t
        where
            Prime<P>: SupportedPrime,
        {
            type Item = u64;

            fn reduce(&self) -> u64 { (self % P as $t) as u64 }
            fn into_item(&self) -> Self::Item { self.reduce() }
        }
    )*};
}
reduce_large_unsigned_impl!(usize, u64, u128);

macro_rules! reduce_signed_impl {
    ($( $i:ty as $u:ty ),*) => {$(
        impl<const P: u64> Reduce<P> for $i
        where
            Prime<P>: SupportedPrime,
        {
            type Item = <$u as Reduce<P>>::Item;

            fn reduce(&self) -> u64 {
                let unsigned = <$u>::from_be_bytes(self.to_be_bytes());
                <$u as Reduce<P>>::reduce(&unsigned)
            }

            fn into_item(&self) -> Self::Item {
                let unsigned = <$u>::from_be_bytes(self.to_be_bytes());
                <$u as Reduce<P>>::into_item(&unsigned)
            }
        }
    )*};
}

reduce_signed_impl!(
    i8 as u8,
    i16 as u16,
    i32 as u32,
    i64 as u64,
    i128 as u128,
    isize as usize
);
