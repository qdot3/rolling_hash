//! 反転したスライスも扱えるようにする
use crate::{BaseCount, Prime, RollingHasher, SupportedBaseCount, SupportedPrime};

pub struct BidirectionalRollingHash<'a, const P: u64, const B: usize>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    hasher: &'a RollingHasher<P, B>,
}
