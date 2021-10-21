use crate::{Field, PrimIntField, Redc, RugField};

pub trait Element:
    Sized + Clone + std::fmt::Debug + std::ops::Add + std::ops::Sub + std::ops::Mul
{
    type UnderlyingType: Redc;
    fn invert(self) -> Self;
    fn pow(self, exponent: Self) -> Self;
    fn internal(&self) -> &Self::UnderlyingType;
    fn to_normal(self) -> Self::UnderlyingType;
}

#[derive(Clone, Copy, Debug)]
pub struct PrimIntElement<'a, T> {
    element: T,
    field: &'a PrimIntField<T>,
}

impl<'a, T> PrimIntElement<'a, T> {
    pub fn new(element: T, field: &'a PrimIntField<T>) -> Self {
        Self { element, field }
    }
}

impl<'a> Element for PrimIntElement<'a, u64> {
    type UnderlyingType = u64;
    fn invert(mut self) -> Self {
        self.element = self.element.mod_pow(self.field.prime - 2, self.field);
        self
    }

    fn pow(mut self, exponent: Self) -> Self {
        self.element = self.element.mod_pow(exponent.element, self.field);
        self
    }

    fn internal(&self) -> &Self::UnderlyingType {
        &self.element
    }

    fn to_normal(self) -> Self::UnderlyingType {
        self.element.to_normal(self.field)
    }
}

impl<'a> std::ops::Add for PrimIntElement<'a, u64> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.element = self
            .element
            .checked_add(rhs.element)
            .map(|x| x.checked_sub(self.field.prime).unwrap_or(x))
            .unwrap_or_else(|| rhs.element - (self.field.prime - self.element));
        self
    }
}

impl<'a> std::ops::Sub for PrimIntElement<'a, u64> {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.element = self
            .element
            .checked_sub(rhs.element)
            .unwrap_or_else(|| self.element + (self.field.prime - rhs.element));
        self
    }
}

impl<'a> std::ops::Mul for PrimIntElement<'a, u64> {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        self.element = self.field.redc(
            <u64 as Redc>::SourceType::from(self.element)
                * <u64 as Redc>::SourceType::from(rhs.element),
        );
        self
    }
}

impl<'a> Element for PrimIntElement<'a, u128> {
    type UnderlyingType = u128;

    fn invert(mut self) -> Self {
        self.element = self.element.mod_pow(self.field.prime - 2, self.field);
        self
    }

    fn pow(mut self, exponent: Self) -> Self {
        self.element = self.element.mod_pow(exponent.element, self.field);
        self
    }

    fn internal(&self) -> &Self::UnderlyingType {
        &self.element
    }

    fn to_normal(self) -> Self::UnderlyingType {
        self.element.to_normal(self.field)
    }
}

impl<'a> std::ops::Add for PrimIntElement<'a, u128> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.element = self
            .element
            .checked_add(rhs.element)
            .map(|x| x.checked_sub(self.field.prime).unwrap_or(x))
            .unwrap_or_else(|| rhs.element - (self.field.prime - self.element));
        self
    }
}

impl<'a> std::ops::Sub for PrimIntElement<'a, u128> {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.element = self
            .element
            .checked_sub(rhs.element)
            .unwrap_or_else(|| self.element + (self.field.prime - rhs.element));
        self
    }
}

impl<'a> std::ops::Mul for PrimIntElement<'a, u128> {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        self.element = self
            .field
            .redc(crate::U256::from(self.element) * crate::U256::from(rhs.element));
        self
    }
}

#[derive(Clone, Debug)]
pub struct RugElement<'a> {
    element: rug::Integer,
    field: &'a RugField,
}
impl<'a> RugElement<'a> {
    pub fn new(element: rug::Integer, field: &'a RugField) -> Self {
        Self { element, field }
    }
}

impl<'a> Element for RugElement<'a> {
    type UnderlyingType = rug::Integer;

    fn invert(mut self) -> Self {
        self.element = self
            .element
            .mod_pow(self.field.prime.clone() - 2, self.field);
        self
    }

    fn pow(mut self, exponent: Self) -> Self {
        self.element = self.element.mod_pow(exponent.element, self.field);
        self
    }

    fn internal(&self) -> &Self::UnderlyingType {
        &self.element
    }

    fn to_normal(self) -> Self::UnderlyingType {
        self.element.to_normal(self.field)
    }
}

impl<'a> std::ops::Add for RugElement<'a> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.element -= rhs.element;
        if self.element >= self.field.prime {
            self.element -= &self.field.prime;
        }
        self
    }
}

impl<'a> std::ops::Sub for RugElement<'a> {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.element -= rhs.element;
        if self.element > 0 {
            self.element += &self.field.prime;
        }
        self
    }
}

impl<'a> std::ops::Mul for RugElement<'a> {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        self.element = self.field.redc(self.element * rhs.element);
        self
    }
}
