/// Specified prime number that is suitable for [`RollingHasher`].
pub struct Prime<const P: u64>;

/// A marker trait for prime numbers that are suitable for [`RollingHasher`].
/*
! # Constraints
!
! - (1 <=) DIFF <= min(64-EXP, floor(EXP/2))
! - EXP <= 61
*/
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

/// **FIXME**: These methods should be a [SupportedPrime]`s ones.
impl<const P: u64> Prime<P>
where
    Prime<P>: SupportedPrime,
{
    /// Performs `lhs + rhs % P` without overflow.
    ///
    /// # Constraints
    ///
    /// - `lhs, rhs < P`. Otherwise, overflow may or may not occur.
    /// - `P` is limited. See [SupportedPrime].
    ///
    /// # Time complexity
    ///
    /// *O*(1)
    pub(crate) const fn mul_mod(lhs: u64, rhs: u64) -> u64 {
        let (exp, diff, bits_l, mask_l) = const {
            // P = 2^EXP - DIFF
            //
            // # Constraints
            //
            // - EXP <= 61
            // - (1 <=) DIFF <= 2^min(64-EXP, floor(EXP/2))
            let exp = P.next_power_of_two().ilog2() as u64;
            let diff = (1 << exp) - P;

            // u: ⎿ EXP / 2 ⏌
            // l: ⎾ EXP / 2 ⏋
            let bits_l = (exp + 1) / 2;
            let mask_l = (1 << bits_l) - 1;

            (exp, diff, bits_l, mask_l)
        };

        let (lhs_l, lhs_u) = (lhs & mask_l, lhs >> bits_l);
        let (rhs_l, rhs_u) = (rhs & mask_l, rhs >> bits_l);
        // ==================================================

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
        let uu = lhs_u * rhs_u * (exp % 2 + 1) * diff;

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
            let (cross_l, cross_u) = (cross & mask_l, cross >> bits_l);
            cross_u * (exp % 2 + 1) * diff + (cross_l << bits_l)
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
    /// See [mul_mod](Self::mul_mod).
    ///
    /// # Time complexity
    ///
    /// *O*(log *exp*)
    pub(crate) const fn pow_mod(mut value: u64, mut exp: u64) -> u64 {
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
