//! 反転したスライスも扱えるようにする
use crate::{BaseCount, Prime, RollingHasher, SupportedBaseCount, SupportedPrime};

pub struct BidirectionalRollingHash<'a, const P: u64, const B: usize>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    hasher: &'a RollingHasher<P, B>,
}

impl<'a, const P: u64, const B: usize> BidirectionalRollingHash<'a, P, B>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    pub(crate) fn new(hasher: &'a RollingHasher<P, B>) -> Self {
        todo!()
    }
}
