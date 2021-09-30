pub mod element;

use element::{PrimIntElement, RugElement};
use num_traits::{PrimInt, WrappingMul};
use twoword::TwoWord;
pub trait Field<T: Redc> {
    fn redc(&self, value: T::SourceType) -> T;
}

pub trait Redc: Sized {
    type SourceType;
    type FieldType;

    fn setup_field(self) -> Self::FieldType;
    fn to_montgomery(self, field: &Self::FieldType) -> Self;
    fn to_montgomery_unchecked(self, field: &Self::FieldType) -> Self;
    fn to_normal(self, field: &Self::FieldType) -> Self;
    fn mod_pow(self, exponent: Self, field: &Self::FieldType) -> Self;
}

/// Using hensel lifting to calculate `prime_inverse` for `prime_inverse` * prime = -1 mod R
/// With R being 2**(bits of T)
fn p_calc_prime_inverse<T>(prime: T) -> T
where
    T: PrimInt + std::fmt::Display + WrappingMul + std::ops::ShlAssign + std::ops::BitOrAssign,
{
    let two = T::one() + T::one();
    assert!(
        prime % two != T::zero(),
        "Prime {} needs to be coprime to base 2**x, but is not (cannot be divisible by 2)",
        prime
    );
    let mut last_power = two;
    let mut current_power = two + two;
    let mut prime_inv_mod = T::one();
    let mut mod_mask = current_power - T::one();

    while last_power != T::zero() {
        let prime_mod = prime & mod_mask;
        if prime_mod.wrapping_mul(&prime_inv_mod) & mod_mask != mod_mask {
            prime_inv_mod |= last_power;
        }

        last_power = current_power;
        current_power <<= T::one();
        mod_mask <<= T::one();
        mod_mask |= T::one();
    }
    prime_inv_mod
}

fn p_calc_r_squared_u64(prime: u64) -> u64 {
    let r_mod = ((u64::MAX % prime) + 1) % prime;
    let r_squared = (<u64 as Redc>::SourceType::from(r_mod)
        * <u64 as Redc>::SourceType::from(r_mod))
        % <u64 as Redc>::SourceType::from(prime);
    #[allow(clippy::cast_possible_truncation)]
    {
        r_squared as u64
    }
}

impl PrimIntField<u64> {
    // Convert to montgomery representation, and use a wrapper to do arithmetic without needing to do manual redc operations
    pub fn wrap_element(&self, element: u64) -> PrimIntElement<'_, u64> {
        PrimIntElement::new(element.to_montgomery(self), self)
    }
    pub fn raw_element(&self, element: u64) -> PrimIntElement<'_, u64> {
        PrimIntElement::new(element, self)
    }
}

impl Field<u64> for PrimIntField<u64> {
    fn redc(&self, value: <u64 as Redc>::SourceType) -> u64 {
        let prime_bits = u64::MAX.count_ones();
        let value_mod_r = value % (1 << prime_bits);
        let value_times_n_prime =
            value_mod_r * <u64 as Redc>::SourceType::from(self.prime_inverted);
        let m = value_times_n_prime % (1 << prime_bits);
        let m_times_prime = m * (<u64 as Redc>::SourceType::from(self.prime));
        let (mut tw, carry) = m_times_prime.overflowing_add(value);
        tw /= 1 << prime_bits;
        if carry {
            tw += 1 << prime_bits;
        }
        if tw >= <u64 as Redc>::SourceType::from(self.prime) {
            tw -= <u64 as Redc>::SourceType::from(self.prime);
        }
        #[allow(clippy::cast_possible_truncation)]
        {
            tw as u64
        }
    }
}

impl Redc for u64 {
    type SourceType = u128;
    type FieldType = PrimIntField<Self>;

    fn setup_field(self) -> Self::FieldType {
        Self::FieldType {
            prime: self,
            prime_inverted: p_calc_prime_inverse(self),
            r_squared: p_calc_r_squared_u64(self),
        }
    }

    fn to_montgomery_unchecked(self, field: &Self::FieldType) -> Self {
        debug_assert!(self <= field.prime);
        field.redc(Self::SourceType::from(self) * Self::SourceType::from(field.r_squared))
    }

    fn to_montgomery(self, field: &Self::FieldType) -> Self {
        (self % field.prime).to_montgomery_unchecked(field)
    }

    fn to_normal(self, field: &Self::FieldType) -> Self {
        field.redc(Self::SourceType::from(self))
    }

    fn mod_pow(self, mut exponent: Self, field: &Self::FieldType) -> Self {
        let mut power = self;
        let mut result = if exponent % 2 == 0 {
            1u64.to_montgomery_unchecked(field)
        } else {
            self
        };
        exponent >>= 1;
        while exponent != 0 {
            power = field.redc(u128::from(power) * u128::from(power));
            if exponent % 2 == 1 {
                result = field.redc(u128::from(result) * u128::from(power));
            }
            exponent >>= 1;
        }
        result
    }
}

fn p_calc_r_squared_u128(prime: u128) -> u128 {
    let r_mod = ((u128::MAX % prime) + 1) % prime;
    let r_squared =
        (<u128 as Redc>::SourceType::mult(r_mod, r_mod)) % <u128 as Redc>::SourceType::from(prime);
    r_squared.lower
}

impl Redc for u128 {
    type SourceType = TwoWord<Self>;
    type FieldType = PrimIntField<Self>;

    fn setup_field(self) -> Self::FieldType {
        Self::FieldType {
            prime: self,
            prime_inverted: p_calc_prime_inverse(self),
            r_squared: p_calc_r_squared_u128(self),
        }
    }

    fn to_montgomery_unchecked(self, field: &Self::FieldType) -> Self {
        debug_assert!(self <= field.prime);
        field.redc(Self::SourceType::from(self) * Self::SourceType::from(field.r_squared))
    }

    fn to_montgomery(self, field: &Self::FieldType) -> Self {
        (self % field.prime).to_montgomery_unchecked(field)
    }

    fn to_normal(self, field: &Self::FieldType) -> Self {
        field.redc(Self::SourceType {
            higher: 0,
            lower: self,
        })
    }

    fn mod_pow(self, mut exponent: Self, field: &Self::FieldType) -> Self {
        let mut power = self;
        let mut result = if exponent % 2 == 0 {
            1u128.to_montgomery_unchecked(field)
        } else {
            self
        };
        exponent >>= 1;
        while exponent != 0 {
            power = field.redc(Self::SourceType::mult(power, power));
            if exponent % 2 == 1 {
                result = field.redc(Self::SourceType::mult(result, power));
            }
            exponent >>= 1;
        }
        result
    }
}

impl PrimIntField<u128> {
    // Convert to montgomery representation, and use a wrapper to do arithmetic without needing to do manual redc operations
    pub fn wrap_element(&self, element: u128) -> PrimIntElement<'_, u128> {
        PrimIntElement::new(element.to_montgomery(self), self)
    }
    pub fn raw_element(&self, element: u128) -> PrimIntElement<'_, u128> {
        PrimIntElement::new(element, self)
    }
}

impl Field<u128> for PrimIntField<u128> {
    fn redc(&self, value: <u128 as Redc>::SourceType) -> u128 {
        use num_traits::ops::overflowing::OverflowingAdd;

        let value_mod_r = value.lower;
        let value_times_n_prime =
            <u128 as Redc>::SourceType::mult(value_mod_r, self.prime_inverted);
        let m = value_times_n_prime.lower;
        let m_times_prime = <u128 as Redc>::SourceType::mult(m, self.prime);
        let (tw, carry) = m_times_prime.overflowing_add(&value);
        let mut tw_higher = tw.higher;
        if carry {
            tw_higher += u128::MAX - self.prime + 1;
        } else if tw_higher >= self.prime {
            tw_higher -= self.prime;
        }
        tw_higher
    }
}

fn rug_calc_prime_inverse(prime: rug::Integer) -> rug::Integer {
    assert!(
        prime.clone() % 2 != 0,
        "Prime {:#?} needs to be coprime to base 2**x, but is not (cannot be divisible by 2)",
        prime
    );
    let mut last_power = rug::Integer::from(2);
    let mut current_power = rug::Integer::from(4);
    let mut prime_inv_mod = rug::Integer::from(1);
    let mut mod_mask = rug::Integer::from(0b11);
    let r = prime.clone().next_power_of_two();
    while last_power != r {
        let prime_mod = prime.clone() & mod_mask.clone();
        if (prime_mod * prime_inv_mod.clone()) & mod_mask.clone() != mod_mask.clone() {
            prime_inv_mod |= last_power;
        }

        last_power = current_power.clone();
        current_power <<= 1;
        mod_mask <<= 1;
        mod_mask |= 1;
    }
    prime_inv_mod
}

impl RugField {
    // Convert to montgomery representation, and use a wrapper to do arithmetic without needing to do manual redc operations
    pub fn wrap_element(&self, element: rug::Integer) -> RugElement<'_> {
        RugElement::new(element.to_montgomery(self), self)
    }
}

impl Field<rug::Integer> for RugField {
    fn redc(&self, value: <rug::Integer as Redc>::SourceType) -> rug::Integer {
        let tw = ((((value.clone().keep_bits(self.r_count)) * &self.prime_inverted)
            .keep_bits(self.r_count))
            * &self.prime
            + value)
            >> self.r_count;
        if tw >= self.prime {
            tw - &self.prime
        } else {
            tw
        }
    }
}

impl Redc for rug::Integer {
    type SourceType = Self;
    type FieldType = RugField;

    fn setup_field(self) -> Self::FieldType {
        let r = self.clone().next_power_of_two();
        let r_count = r.find_one(0).unwrap();
        let r_squared = r.square() % &self;
        Self::FieldType {
            prime: self.clone(),
            prime_inverted: rug_calc_prime_inverse(self),
            r_squared,
            r_count,
        }
    }

    fn to_montgomery_unchecked(self, field: &Self::FieldType) -> Self {
        debug_assert!(self <= field.prime);
        field.redc(self * &field.r_squared)
    }

    fn to_montgomery(self, field: &Self::FieldType) -> Self {
        field.redc((self % &field.prime) * &field.r_squared)
    }

    fn to_normal(self, field: &Self::FieldType) -> Self {
        field.redc(self)
    }

    fn mod_pow(self, mut exponent: Self, field: &Self::FieldType) -> Self {
        let mut result = if exponent.clone() % 2 == 0 {
            Self::from(1).to_montgomery_unchecked(field)
        } else {
            self.clone()
        };
        let mut power = self;
        exponent >>= 1;
        while exponent != 0 {
            power = field.redc(power.square());
            if exponent.clone() % 2 == 1 {
                result = field.redc(result * power.clone());
            }
            exponent >>= 1;
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct RugField {
    prime: rug::Integer,
    prime_inverted: rug::Integer,
    r_squared: rug::Integer,
    r_count: u32,
}

#[derive(Debug, Clone)]
pub struct PrimIntField<T> {
    prime: T,
    prime_inverted: T,
    r_squared: T,
}

#[cfg(test)]
mod tests {
   use super::p_calc_prime_inverse;
#[test]
fn test_prime_inverse() {
    assert_eq!(p_calc_prime_inverse(23u64), 3_208_129_404_123_400_281);
}
}
