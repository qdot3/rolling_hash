//! Rolling Hashの試験的実装
//!
#![doc = include_str!("../blueprint.md")]
use std::ops::Deref;

mod prime;
pub use prime::{PRIMES, Prime, SupportedPrime};

mod oneway;
pub use oneway::OneWay;

pub(crate) mod mock;
pub(crate) use mock::cold_path;

pub(crate) mod windows;
pub(crate) use windows::Windows;

/// Specifies the number of bases in [`RollingHasher`].
///
/// This sill be small.
pub struct BaseCount<const B: usize>;

/// A marker trait for supported number of bases.
pub trait SupportedBaseCount {}

macro_rules! supported_base_count_impl {
    ($( $b:literal ),+) => {$(
        impl SupportedBaseCount for BaseCount<$b> {}
    )+};
}
supported_base_count_impl! { 2, 3, 4, 5, 6, 7, 8, 9, 10 }

/// A value that may be incorrect due to hash collisions.
pub struct Maybe<T>(T);

impl<T> Deref for Maybe<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
