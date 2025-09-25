use std::{cell::OnceCell, num::NonZero};

use crate::{BaseCount, OneWay, Prime, SupportedBaseCount, SupportedPrime};

pub(crate) struct Windows<'a, const P: u64, const B: usize>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    hash: &'a [[u64; B]],
    size: NonZero<usize>,

    base_or_offset: [u64; B],
    base_pow_size: OnceCell<[u64; B]>,
}

impl<'a, const P: u64, const B: usize> Windows<'a, P, B>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    pub(crate) fn new(hasher: &'a OneWay<P, B>, size: NonZero<usize>) -> Self {
        Self {
            hash: hasher.get_hash(),
            size,
            base_or_offset: hasher.base(),
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
        if self.size.get() > self.hash.len() {
            None
        } else {
            let base_pow_size = self.base_pow_size.get_or_init(|| {
                let pow = std::array::from_fn(|i| {
                    Prime::<P>::pow_mod(self.base_or_offset[i], self.size.get() as u64)
                });
                // initialized only once
                self.base_or_offset.fill(0);
                pow
            });

            let ret = std::array::from_fn(|i| {
                (self.hash[self.size.get() - 1][i] + P
                    - Prime::<P>::mul_mod(self.base_or_offset[i], base_pow_size[i]))
                    % P
            });

            self.base_or_offset = self.hash[0];
            self.hash = &self.hash[1..];

            Some(ret)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.hash.len().saturating_sub(self.size.get() - 1);
        (size, Some(size))
    }
}

impl<'a, const P: u64, const B: usize> DoubleEndedIterator for Windows<'a, P, B>
where
    Prime<P>: SupportedPrime,
    BaseCount<B>: SupportedBaseCount,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.size.get().cmp(&self.hash.len()) {
            std::cmp::Ordering::Less => {
                let base_pow_size = self.base_pow_size.get_or_init(|| {
                    let pow = std::array::from_fn(|i| {
                        Prime::<P>::pow_mod(self.base_or_offset[i], self.size.get() as u64)
                    });
                    // initialized only once
                    self.base_or_offset.fill(0);
                    pow
                });

                let ret = std::array::from_fn(|i| {
                    (self.hash[self.hash.len() - 1][i] + P
                        - Prime::<P>::mul_mod(
                            self.hash[self.hash.len() - self.size.get() - 1][i],
                            base_pow_size[i],
                        ))
                        % P
                });

                self.hash = &self.hash[..self.hash.len() - 1];

                Some(ret)
            }
            std::cmp::Ordering::Equal => {
                let ret = self.hash[self.size.get() - 1];
                self.hash = &self.hash[..self.size.get() - 1];
                Some(ret)
            }
            std::cmp::Ordering::Greater => None,
        }
    }
}
