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
pub(crate) const fn mul_mod<const P: u64>(lhs: u64, rhs: u64) -> u64 {
    // ==================================================
    // values here will be evaluated in compile time

    // P = 2^EXP - DIFF
    //
    // # Constraints
    //
    // - EXP <= 61
    // - (1 <=) DIFF <= 2^min(64-EXP, floor(EXP/2))
    let exp = const { P.next_power_of_two().ilog2() as u64 };
    let diff = const { P.next_power_of_two() - P };

    let _check1 = const { 61 - P.next_power_of_two().ilog2() };
    let _check2 = const { P.next_power_of_two() * (P.next_power_of_two() - P) };
    let _check3 = const { (1 << P.next_power_of_two().ilog2() / 2) - (P.next_power_of_two() - P) };

    // u: ⎿ EXP / 2 ⏌
    // l: ⎾ EXP / 2 ⏋
    let lower_bits = (exp + 1) / 2;
    let lower_mask = (1 << lower_bits) - 1;

    let (lhs_l, lhs_u) = (lhs & lower_mask, lhs >> lower_bits);
    let (rhs_l, rhs_u) = (rhs & lower_mask, rhs >> lower_bits);
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
        let (cross_l, cross_u) = (cross & lower_mask, cross >> lower_bits);
        cross_u * (exp % 2 + 1) * diff + (cross_l << lower_bits)
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
pub(crate) const fn pow_mod<const P: u64>(mut value: u64, mut exp: u64) -> u64 {
    let mut result = 1; // P >> 1
    while exp > 0 {
        if exp & 1 == 1 {
            result = mul_mod::<P>(result, value);
        }
        exp >>= 1;
        value = mul_mod::<P>(value, value);
    }
    result
}
