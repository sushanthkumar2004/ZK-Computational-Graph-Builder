use std::ops::*;

// We want our field to support the 4 operations  
// and Send, Sync so that we can use parallel iterators over it. 
pub trait Field: std::fmt::Debug + PartialEq + std::marker::Sized + Mul<Output=Self> + Add<Output=Self> + Sub<Output=Self> + Div<Output=Self> + From<u64> + Sync + Send + Clone + Copy {}

// Allows us to declare GaloisField<p> = Z/pZ where p is a prime. 
// Note that this doesnt actually enforce p to be prime, but
// otherwise it's not a field. 
impl<const MODULUS: u64> Field for GaloisField<MODULUS> {}

// value stores the reduced value mod MODULUS
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct GaloisField<const MODULUS: u64>{
    pub value: u64
}

// allow us to convert u64 into field element using standard syntax. 
impl<const MODULUS: u64> From<u64> for GaloisField<MODULUS> {
    fn from(value: u64) -> Self {
        GaloisField{
            value: value % MODULUS
        }
    }
}

// the usual field operations 
impl<const MODULUS: u64> Add for GaloisField<MODULUS> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        GaloisField {
            value: (self.value + rhs.value) % MODULUS
        }
    }
}

impl<const MODULUS: u64> Sub for GaloisField<MODULUS> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        GaloisField {
            value: (self.value - rhs.value) % MODULUS
        }
    }
}

impl<const MODULUS: u64> Mul for GaloisField<MODULUS> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        GaloisField {
            value: (self.value * rhs.value) % MODULUS
        }
    }
}

impl<const MODULUS: u64> Div for GaloisField<MODULUS> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        GaloisField {
            value: (self.value * reciprocal(rhs.value, MODULUS)) % MODULUS
        }
    }
}


// functions to assist in field dvision. 
// Returns x,y such that ax + by = gcd(a,b)
pub fn extended_euclidean(a: i128, b: i128) -> [i128; 3] {
    if a == 0 {
        return [b, 0, 1];
    }
    let [gcd, x1, y1] = extended_euclidean(b % a, a);
    [gcd, y1 - (b / a) * x1, x1]
}

// assumes that a < modulus, and computes 1/a. 
// throws division by zero error is a % modulus == 0
pub fn reciprocal(a: u64, modulus: u64) -> u64 {
    if a % modulus == 0 {
        panic!("Attempted division by zero in field with {:?} modulo {:?}", a, modulus);
    }

    let [_, x, _] = extended_euclidean((a % modulus) as i128, modulus as i128);
    ((x + modulus as i128) % modulus as i128).try_into().unwrap()
}