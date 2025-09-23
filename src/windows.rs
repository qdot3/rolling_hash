use std::{cell::OnceCell, num::NonZero};

use crate::{BaseCount, Prime, RollingHasher, SupportedBaseCount, SupportedPrime};

pub(crate) struct Windows<'a, const P: u64, const B: usize>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    hashed: &'a [[u64; B]],
    size: NonZero<usize>,

    base_or_offset: [u64; B],
    base_pow_size: OnceCell<[u64; B]>,
}

impl<'a, const P: u64, const B: usize> Windows<'a, P, B>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    pub(crate) fn new(hasher: &'a RollingHasher<P, B>, size: NonZero<usize>) -> Self {
        Self {
            hashed: &hasher.hashed,
            size,
            base_or_offset: hasher.base,
            base_pow_size: OnceCell::new(),
        }
    }
}

impl<'a, const P: u64, const B: usize> ExactSizeIterator for Windows<'a, P, B>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
}

impl<'a, const P: u64, const B: usize> Iterator for Windows<'a, P, B>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    type Item = [u64; B];

    fn next(&mut self) -> Option<Self::Item> {
        if self.size.get() > self.hashed.len() {
            None
        } else {
            let base_pow_size = self.base_pow_size.get_or_init(|| {
                let pow = std::array::from_fn(|i| {
                    RollingHasher::<P, B>::pow_mod(self.base_or_offset[i], self.size.get() as u64)
                });
                // initialized only once
                self.base_or_offset.fill(0);
                pow
            });

            let ret = std::array::from_fn(|i| {
                (self.hashed[self.size.get() - 1][i] + P
                    - RollingHasher::<P, B>::mul_mod(self.base_or_offset[i], base_pow_size[i]))
                    % P
            });

            self.base_or_offset = self.hashed[0];
            self.hashed = &self.hashed[1..];

            Some(ret)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.hashed.len().saturating_sub(self.size.get() - 1);
        (size, Some(size))
    }
}

impl<'a, const P: u64, const B: usize> DoubleEndedIterator for Windows<'a, P, B>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.size.get().cmp(&self.hashed.len()) {
            std::cmp::Ordering::Less => {
                let base_pow_size = self.base_pow_size.get_or_init(|| {
                    let pow = std::array::from_fn(|i| {
                        RollingHasher::<P, B>::pow_mod(
                            self.base_or_offset[i],
                            self.size.get() as u64,
                        )
                    });
                    // initialized only once
                    self.base_or_offset.fill(0);
                    pow
                });

                let ret = std::array::from_fn(|i| {
                    (self.hashed[self.hashed.len() - 1][i] + P
                        - RollingHasher::<P, B>::mul_mod(
                            self.hashed[self.hashed.len() - self.size.get() - 1][i],
                            base_pow_size[i],
                        ))
                        % P
                });

                self.hashed = &self.hashed[..self.hashed.len() - 1];

                Some(ret)
            }
            std::cmp::Ordering::Equal => {
                let ret = self.hashed[self.size.get() - 1];
                self.hashed = &self.hashed[..self.size.get() - 1];
                Some(ret)
            }
            std::cmp::Ordering::Greater => None,
        }
    }
}
