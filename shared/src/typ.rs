use core::convert::From;
use core::ops::{Add, AddAssign, Rem};

#[derive(Clone, Copy)]
pub struct Digit(u8);

impl Digit {
    pub fn new(v: u8) -> Digit {
        Digit(v % 10)
    }
}

impl Add for Digit {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Add<u8> for Digit {
    type Output = Self;

    fn add(self, rhs: u8) -> Self {
        Self(self.0 + rhs)
    }
}

impl AddAssign for Digit {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = (self.0 + rhs.0) % 10;
    }
}

impl AddAssign<u8> for Digit {
    fn add_assign(&mut self, rhs: u8) {
        self.0 = (self.0 + rhs) % 10;
    }
}

impl Rem for Digit {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self {
        Self(self.0 % rhs.0)
    }
}

impl From<Digit> for usize {
    fn from(value: Digit) -> Self {
        return value.0 as usize;
    }
}
