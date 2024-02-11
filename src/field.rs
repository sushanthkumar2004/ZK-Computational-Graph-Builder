use std::{ops::*};

pub trait Field: std::fmt::Debug + PartialEq + std::marker::Sized + Mul<Output=Self> + Add<Output=Self> + From<u64> + Sync + Send + Clone + Copy {
    type Output = Self;
}


impl<const MODULUS: u64> Field for GaloisField<MODULUS> {
    type Output = Self;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GaloisField<const MODULUS: u64>{
    pub value: u64
}

impl<const MODULUS: u64> From<u64> for GaloisField<MODULUS> {
    fn from(value: u64) -> Self {
        GaloisField{
            value: value % MODULUS
        }
    }
}

impl<const MODULUS: u64> Add for GaloisField<MODULUS> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        GaloisField{
            value: (self.value + rhs.value) % MODULUS
        }
    }
}

impl<const MODULUS: u64> Mul for GaloisField<MODULUS> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        GaloisField{
            value: (self.value * rhs.value) % MODULUS
        }
    }
}