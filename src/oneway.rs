use std::num::NonZero;

use crate::{BaseCount, Maybe, Prime, SupportedBaseCount, SupportedPrime, Windows, cold_path};

pub struct OneWay<const P: u64, const B: usize>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    base: [u64; B],
    hash: Vec<[u64; B]>,
}

impl<const P: u64, const B: usize> OneWay<P, B>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            base: std::array::from_fn(|_| rand::random_range(2..=P - 2)),
            hash: Vec::new(),
        }
    }

    /// Creates a new instance with at least the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            base: std::array::from_fn(|_| rand::random_range(2..=P - 2)),
            hash: Vec::with_capacity(capacity),
        }
    }

    /// Same as [`Vec::reserve`].
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.hash.reserve(additional);
    }

    /// Returns the number of elements in `self`.
    #[inline]
    pub const fn len(&self) -> usize {
        self.hash.len()
    }

    /// Returns `true` if `self` has a length of 0, and `false` otherwise.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.hash.is_empty()
    }

    /// Returns bases randomly generated at runtime.
    ///
    /// # Time Complexity
    ///
    /// *O*(*B*)
    pub fn base(&self) -> [u64; B] {
        self.base
    }

    pub(crate) fn get_hash(&self) -> &[[u64; B]] {
        &self.hash
    }

    /// Hashes `next` by using `self`.
    /// You can simply push the result to the `hashed` field (and `next` to the `source` field).
    ///
    /// # Constraints
    ///
    /// `next < P`
    ///
    /// # Time complexity
    ///
    /// *O*(*B*)
    #[inline]
    fn hash_next(&self, prev: &[u64; B], next: u64) -> [u64; B] {
        std::array::from_fn(|i| (Prime::<P>::mul_mod(prev[i], self.base[i]) + next) % P)
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

    /// Appends an element to the back of `self`.
    ///
    /// # Time complexity
    ///
    /// *O*(*B*)
    #[inline]
    pub fn push(&mut self, value: u64) {
        self.hash.push(if let Some(prev) = self.hash.last() {
            self.hash_next(prev, value)
        } else {
            cold_path();
            std::array::from_fn(|_| value)
        });
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

impl<const P: u64, const B: usize, T> Extend<T> for OneWay<P, B>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        todo!()
    }
}
