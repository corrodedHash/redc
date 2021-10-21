// Required for `U128`
#![allow(clippy::reversed_empty_ranges)]
// Silencing both macros
#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]

uint::construct_uint! {
    pub struct U128(2);
}

uint::construct_uint! {
    pub struct U256(4);
}

impl U128 {
    /// Multiplies two 256-bit integers to produce full 512-bit integer.
    /// Overflow is not possible.
    #[inline(always)]
    pub fn full_mul_u128(self, other: u128) -> U256 {
        let o_w = Self::from(other);
        U256(uint::uint_full_mul_reg!(U128, 2, self, o_w))
    }
}
impl U256 {
    #[inline]
    pub const fn high_u128(&self) -> u128 {
        let &Self(ref arr) = self;
        ((arr[3] as u128) << 64) + arr[2] as u128
    }
}
