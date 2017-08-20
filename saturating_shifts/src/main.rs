use std::ops::{Shl, Shr};
fn saturating_shl<T: Shl<u32, Output = T> + PartialOrd + From<u32>>(x: T, y: u32) -> T {
    if y < 32 {
        x << y
    } 
    else {
        // There seems to be no generic way to specify T can be
        // zero and just return zero here. Any way to just make
        // this the T::MIN constant?
        T::from(u32::min_value())
    }
}

fn saturating_shr<T: Shr<u32, Output = T> + PartialOrd + From<u32>>(x: T, y: u32) -> T {
    if y < 32 {
        x >> y
    } 
    else {
        T::from(u32::min_value())
    }
}

// This below would seem to be a better approach.

pub trait SaturatingShl<RHS> {
    type Output;
    fn saturating_shl(self, rhs: RHS) -> Self::Output;
}

macro_rules! saturating_shl_impl {
    ($t:ty, $f:ty, $w:expr) => (
        impl SaturatingShl<$f> for $t {
            type Output = $t;

            #[inline]
            fn saturating_shl(self, rhs: $f) -> $t {
                if rhs < $w { self << rhs } else { 0 }
            }
        }
    )
}

pub trait SaturatingShr<RHS> {
    type Output;
    fn saturating_shr(self, rhs: RHS) -> Self::Output;
}

macro_rules! saturating_shr_impl {
    ($t:ty, $f:ty, $w:expr) => (
        impl SaturatingShr<$f> for $t {
            type Output = $t;

            #[inline]
            fn saturating_shr(self, rhs: $f) -> $t {
                if rhs < $w { self >> rhs } else { 0 }
            }
        }
    )
}

saturating_shl_impl!(u64, u8, 64);
saturating_shr_impl!(u64, u8, 64);

fn main() {
    println!("{}", saturating_shl(1u64, 1));
    println!("{}", saturating_shr(1u64, 1));
    println!("{}", 1u64.saturating_shl(1));
    println!("{}", 1u64.saturating_shr(1));
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;
    use super::*;
    
    macro_rules! saturating_shl_tests {
        ($($name:ident: ($lhs:ty, $rhs:ty),)*) => {
        $(
            #[test]
            fn $name() {
                let a: $lhs = 0b1;
                let z: $rhs = 0;
                let w: $rhs = 8 * size_of::<$lhs>() as $rhs;
                assert_eq!(a.saturating_shl(z), a);
                assert_eq!(a.saturating_shl(w-1), a << w-1);
                assert_eq!(a.saturating_shl(w), 0);
                assert_eq!(a.saturating_shl(z.wrapping_sub(1)), 0);
            }
        )*
        }
    }
    
    macro_rules! saturating_shr_tests {
        ($($name:ident: ($lhs:ty, $rhs:ty),)*) => {
        $(
            #[test]
            fn $name() {
                let a: $lhs = 0b1;
                let z: $rhs = 0;
                let w: $rhs = 8 * size_of::<$lhs>() as $rhs;
                assert_eq!(a.saturating_shr(z), a);
                assert_eq!(a.saturating_shr(w-1), a >> w-1);
                assert_eq!(a.saturating_shr(w), 0);
                assert_eq!(a.saturating_shr(z.wrapping_sub(1)), 0);
            }
        )*
        }
    }

    saturating_shl_tests! {
        saturating_shl_u64_u8: (u64, u8),
    }

    saturating_shr_tests! {
        saturating_shr_u64_u8: (u64, u8),
    }
}
