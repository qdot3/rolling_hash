//! Rolling Hashの試験的実装
//!
//! ## 基本機能
//!
//! - 高速に剰余計算ができる特別な素数を法に使用（~2^60）
//! - 実行時にランダムな基数を生成
//!
//! - `Vec`のように末尾で値の追加・削除ができる。
//! - 元の数列を持っておく必要はない。
//! 連続部分列のハッシュ値を計算するには、ハッシュ値の列があれば十分。
//! 元の数列の型がバラバラでも問題ない？
//!
//! ## メモ
//!
//! - `u64`以外の値を受け取れるようにする
//!
//! - 回文などでは同じ法・同じ基数の組・同じ値の列について、逆向きにハッシュを計算したい。
//! あとから計算するためには元の数列を保持しておく必要がある。
//! このとき、`Vec`のように振舞うことはできず、むしろ`[_]`に近い。
//! なので、`OneWay`と`TowWay`を用意し、内部的には後者は前者を２つ持てばよい。
//!
//! - `std`に倣って抽象化する必要があるか？
//!   - 向きがあるので、一方通行と双方向のものだけあればよい。
//!   - １つの法といくつかの基数についてRolling Hashを計算できれば、それを組み合わせればよい。
//!   - `Hash`は値の拡張性のために必要。
//! 自動的に`Hasher`も必要
//!
//! - 基数を複数もつときは`[Vec<_>; B]`よりも`Vec<[_; B]>`の方が早期リターンしやすくて良い。
//! キャッシュ効率もよさそう。
//! - 複数の基数に対するハッシュ値をまとめることもできるが不要。
//! 基数の数に比例してき計算時間が増大するので、少数に制限するのが良く、arrayで十分。
use std::{num::NonZero, ops::Deref};

mod math;
mod traits;
pub(crate) use math::{mul_mod, pow_mod};

mod oneway;
pub use oneway::OneWay;

mod convert;
pub use convert::Reduce;

pub(crate) mod mock;
pub(crate) use mock::cold_path;

pub(crate) mod windows;
pub(crate) use windows::Windows;

/// Specified prime number that is suitable for [`RollingHasher`].
pub struct Prime<const P: u64>;

/// A marker trait for prime numbers that are suitable for [`RollingHasher`].
pub trait SupportedPrime {}

macro_rules! supported_prime_impl {
    ($n:literal; $( (1 << $exp:literal) - $diff:literal),*$(,)?) => {
        /// Large prime numbers that is suitable for [`RollingHasher`].
        pub const PRIMES: [u64; $n] = [$( { (1 << $exp) - $diff } ),*];

        $(
            impl SupportedPrime for Prime<{ (1 << $exp) - $diff }> {}
        )*
    };
}

// # Constraints
//
// - (1 <=) DIFF <= min(64-EXP, floor(EXP/2))
// - EXP <= 62, but the largest value is 61
supported_prime_impl! {
    // the number of prime numbers. 10 will be sufficient.
    10;
    // # Constraints
    //
    // - P = 2^EXP - DIFF >> 10^9
    // - EXP <= 62
    // - (1 <=) DIFF <= min(64-EXP, floor(EXP/2))
    //
    // 2^57 - x, x < 2^9 = 128
    (1 << 57) - 111,
    (1 << 57) - 69,
    (1 << 57) - 61,
    (1 << 57) - 49,
    (1 << 57) - 25,
    (1 << 57) - 13,
    // 2^58 - x, x < 2^6 = 64
    (1 << 58) - 63,
    (1 << 58) - 57,
    (1 << 58) - 27,
    // the largest prime number
    (1 << 61) - 1,
}

/// Specifies the number of bases in [`RollingHasher`].
///
/// This sill be small.
pub struct BaseCount<const B: usize>;

/// A marker trait for supported number of bases.
pub trait SupportedBaseCount {}

macro_rules! supported_base_count_impl {
    ($( $b:literal ),+) => {$(
        impl SupportedBaseCount for BaseCount<{ $b }> {}
    )+};
}
supported_base_count_impl! { 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 }

/// A value that may be incorrect due to hash collisions.
pub struct Maybe<T>(T);

impl<T> Deref for Maybe<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Rolling hash with single prime number and some bases.
pub struct RollingHasher<const P: u64, const B: usize>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    base: [u64; B],

    source: Vec<u64>,
    hashed: Vec<[u64; B]>,
}

impl<const P: u64, const B: usize> RollingHasher<P, B>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    /// P = 2^EXP - DIFF
    ///
    /// # Constraints
    ///
    /// - EXP <= 61
    /// - (1 <=) DIFF <= 2^min(64-EXP, floor(EXP/2))
    const EXP: u64 = { P.next_power_of_two().ilog2() as u64 };
    const DIFF: u64 = { P.next_power_of_two() - P };

    const LOWER_BITS: u64 = (Self::EXP + 1) / 2;
    const LOWER_MASK: u64 = (1 << Self::LOWER_BITS) - 1;

    /// Performs `lhs + rhs % P` without overflow.
    ///
    /// # Constraints
    ///
    /// `lhs, rhs < P` should be holds.
    /// Otherwise, overflow may or may not occur.
    ///
    /// # Time complexity
    ///
    /// *O*(1)
    const fn mul_mod(lhs: u64, rhs: u64) -> u64 {
        // u: ⎿ EXP / 2 ⏌
        // l: ⎾ EXP / 2 ⏋
        let (lhs_l, lhs_u) = (lhs & Self::LOWER_MASK, lhs >> Self::LOWER_BITS);
        let (rhs_l, rhs_u) = (rhs & Self::LOWER_MASK, rhs >> Self::LOWER_BITS);

        // lhs_u * rhs_u * 2^(2l) % (2^EXP - DIFF)
        //
        // (a) EXP is even (l = u)
        // = lhs_u * rhs_u * 2^EXP % (2^EXP - DIFF)
        // = lhs_u * rhs_u * DIFF % (2^EXP - DIFF)
        //
        // lhs_u * rhs_u < 2^(2u) = 2^EXP
        //
        // (b) EXP i odd (l = u + 1)
        // = lhs_u * rhs_u * 2^(EXP+1) % (2^EXP - DIFF)
        // = lhs_u * rhs_u * 2 * DIFF % (2^EXP - DIFF)
        //
        // lhs_u * rhs_u < 2^(2u) = 2^(EXP-1)
        //
        // # Constraints
        //
        // 2^EXP * DIFF < 2^64
        let uu = lhs_u * rhs_u * (Self::EXP % 2 + 1) * Self::DIFF;

        // ( lhs_u * rhs_l + lhs_l * rhs_u ) * 2^l % (2^EXP - DIFF)
        // = ( cross_u * 2^(2l) + cross_l * 2^l ) % (2^EXP - DIFF)
        //
        // (a) EXP is even (l = u)
        // = ( cross_u * 2^EXP + cross_l * 2^l ) % (2^EXP - DIFF)
        // = ( cross_u * DIFF + cross_l * 2^l ) % (2^EXP - DIFF)
        //
        // cross_u * DIFF + cross_l * 2^l )
        // < ( 2^(u+1) * DIFF + 2^(EXP+1) )
        // = ( 2^(l+1) * DIFF + 2^(EXP+1) )
        //
        // (b) EXP is odd (l = u + 1)
        // = ( cross_u * 2^(EXP+1) + cross_l * 2^l ) % (2^EXP - DIFF)
        // = ( cross_u * 2 * DIFF + cross_l * 2^l ) % (2^EXP - DIFF)
        //
        // cross_u * 2 * DIFF + cross_l * 2^l
        // < 2^(u+2) * DIFF + 2^(EXP+1)
        // = 2^(l+1) * DIFF + 2^(EXP+1)
        //
        // # Constraints
        //
        // - DIFF <= 2^u
        // - EXP <= 62
        let cross = {
            let cross = lhs_u * rhs_l + lhs_l * rhs_u;
            let (cross_l, cross_u) = (cross & Self::LOWER_MASK, cross >> Self::LOWER_BITS);
            cross_u * (Self::EXP % 2 + 1) * Self::DIFF + (cross_l << Self::LOWER_BITS)
        };

        // lhs_l * rhs_l < 2^(2l)
        //
        // # Constraints
        //
        // EXP <= 64
        let ll = lhs_l * rhs_l;

        // # Constraints
        //
        // - (1 <=) DIFF <= min(64-EXP, floor(EXP/2))
        // - EXP <= 62. Since the largest possible prime is 2^61 - 1, then EXP <= 61
        //
        // uu + cross + ll
        // < (2^EXP * DIFF) + 2^(EXP+2) + 2^(EXP+EXP%2)
        // < 2^64 + 2^63 + 2^62
        (uu % P + cross + ll) % P
    }

    /// Performs `value^exp % P` without overflow.
    ///
    /// # Constraints
    ///
    /// `value < P` should be holds.
    /// Otherwise, overflow may or may not occur.
    ///
    /// # Time complexity
    ///
    /// *O*(log *exp*)
    pub fn pow_mod(mut value: u64, mut exp: u64) -> u64 {
        let mut result = 1; // P >> 1
        while exp > 0 {
            if exp & 1 == 1 {
                result = Self::mul_mod(result, value);
            }
            exp >>= 1;
            value = Self::mul_mod(value, value);
        }
        result
    }
}

impl<const P: u64, const B: usize> RollingHasher<P, B>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            base: std::array::from_fn(|_| rand::random_range(2..=P - 2)),
            source: Vec::new(),
            hashed: Vec::new(),
        }
    }

    /// Creates a new instance with at least the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            base: std::array::from_fn(|_| rand::random_range(2..=P - 2)),
            source: Vec::with_capacity(capacity),
            hashed: Vec::with_capacity(capacity),
        }
    }

    /// Same as [`Vec::reserve`].
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.hashed.reserve(additional);
        self.source.reserve(additional);
    }

    /// Returns the number of elements in `self`.
    #[inline]
    pub const fn len(&self) -> usize {
        self.source.len()
    }

    /// Returns `true` if `self` has a length of 0, and `false` otherwise.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    /// Hashes `next` by using this hasher.
    /// You can simply push the result to the `hashed` field (and `next` to the `source` field).
    ///
    /// # Constraints
    ///
    /// `next % P`
    ///
    /// # Time complexity
    ///
    /// *O*(*B*)
    #[inline]
    fn hash_next(&self, prev: &[u64; B], next: u64) -> [u64; B] {
        std::array::from_fn(|i| (Self::mul_mod(prev[i], self.base[i]) + next) % P)
    }

    /// Hashes `slice` by using `self`.
    ///
    /// # Time complexity
    ///
    /// *O*(*BM*), where *M* is `slice.len()`.
    fn hash_slice(
        &self,
        slice: &[u64], /* intentional: iterator may skip some elements */
    ) -> [u64; B] {
        slice
            .into_iter()
            .fold([0; B], |prev, next| self.hash_next(&prev, next % P))
    }

    /// Helper function to calculated `hashed` field.
    ///
    /// # Time complexity
    ///
    /// *O*(*BM*), where *M* is `iter.count()`.
    fn scan_and_hash_iter<'a>(&self, iter: impl IntoIterator<Item = &'a u64>) -> Vec<[u64; B]> {
        iter.into_iter()
            .scan([0; B], |prev, next| {
                *prev = self.hash_next(&prev, next % P);
                Some(*prev)
            })
            // intentional: all elements should be allocated in this order
            .collect()
    }

    /// Appends an element to the back of `self`.
    ///
    /// # Time complexity
    ///
    /// *O*(*B*)
    #[inline]
    pub fn push(&mut self, value: u64) {
        self.hashed.push(if let Some(prev) = self.hashed.last() {
            self.hash_next(prev, value)
        } else {
            cold_path();
            std::array::from_fn(|_| value)
        });
        self.source.push(value);
    }

    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    ///
    /// # Time complexity
    ///
    /// *O*(*BM*), where *M* is `other.len()`
    pub fn append(&mut self, other: &mut Vec<u64>) {
        self.reserve(other.len());
        for value in other.drain(..) {
            self.push(value);
        }
    }

    /// # Panics
    ///
    /// Panics if `size` is `0`.
    ///
    /// # Time complexity
    ///
    /// *O*(*B*)
    fn windows(&self, size: usize) -> Windows<'_, P, B> {
        let size = NonZero::new(size).expect("window size must be non-zero");
        Windows::new(self, size)
    }

    /// Searches for an sub slice in `self`, returning its index.
    ///
    /// # Time complexity
    ///
    /// *O*(*BN*), where *N* is `self.len()`.
    pub fn position(&self, slice: &[u64]) -> Option<Maybe<usize>> {
        let target = self.hash_slice(slice);
        self.windows(slice.len())
            .position(|sub_slice| sub_slice == target)
            .map(|i| Maybe(i))
    }

    /// Searches for sub slice in `self` from the right, returning its index.
    ///
    /// # Time complexity
    ///
    /// *O*(*BN*), where *N* is `self.len()`.
    pub fn rposition(&self, slice: &[u64]) -> Option<Maybe<usize>> {
        let target = self.hash_slice(slice);
        self.windows(slice.len())
            .rposition(|sub_slice| sub_slice == target)
            .map(|i| Maybe(i))
    }

    /// Searches for sub slice in `self`, returning all indexes.
    ///
    /// # Time complexity
    ///
    /// *O*(*BN*), where *N* is `self.len()`.
    pub fn positions(&self, slice: &[u64]) -> impl Iterator<Item = Maybe<usize>> {
        let target = self.hash_slice(slice);
        self.windows(slice.len())
            .enumerate()
            .filter_map(move |(i, sub_slice)| (sub_slice == target).then_some(Maybe(i)))
    }

    /// Counts sub slices in `self`.
    ///
    /// # Time complexity
    ///
    /// *O*(*BN*), where *N* is `self.len()`.
    pub fn count(&self, slice: &[u64]) -> Maybe<usize> {
        let target = self.hash_slice(slice);
        Maybe(
            self.windows(slice.len())
                .filter(|sub_slice| sub_slice == &target)
                .count(),
        )
    }
}
