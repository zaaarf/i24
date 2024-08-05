//! # i24: A 24-bit Signed Integer Type
//!
//! The `i24` crate provides a 24-bit signed integer type for Rust, filling the gap between
//! `i16` and `i32`. This type is particularly useful in audio processing, certain embedded
//! systems, and other scenarios where 24-bit precision is required but 32 bits would be excessive.
//!
//! ## Features
//!
//! - Efficient 24-bit signed integer representation
//! - Seamless conversion to and from `i32`
//! - Support for basic arithmetic operations with overflow checking
//! - Bitwise operations
//! - Conversions from various byte representations (little-endian, big-endian, native)
//! - Implements common traits like `Debug`, `Display`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash`
//!
//! This crate came about as a part of the [Wavers](https://crates.io/crates/wavers) project, which is a Wav file reader and writer for Rust.
//! The `i24` struct also has pyo3 bindings for use in Python. Enable the ``pyo3`` feature to use the pyo3 bindings.
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! i24 = "2.0.0"
//! ```
//!
//! Then, in your Rust code:
//!
//! ```rust
//! use i24::i24;
//!
//! let a = i24!(1000);
//! let b = i24!(2000);
//! let c = a + b;
//! assert_eq!(c.to_i32(), 3000);
//! assert_eq!(c, i24!(3000));
//! ```
//!
//! ## Safety and Limitations
//!
//! While `i24` strives to behave similarly to Rust's built-in integer types, there are some
//! important considerations:
//!
//! - The valid range for `i24` is [-8,388,608, 8,388,607].
//! - Overflow behavior in arithmetic operations matches that of `i32`.
//! - Bitwise operations are performed on the 24-bit representation.
//!
//! Always use checked arithmetic operations when dealing with untrusted input or when
//! overflow/underflow is a concern.
//!
//! ## Features
//! - **pyo3**: Enables the pyo3 bindings for the `i24` type.
//!
//! ## Contributing
//!
//! Contributions are welcome! Please feel free to submit a Pull Request. This really needs more testing and verification.
//!
//! ## License
//!
//! This project is licensed under MIT - see the [LICENSE](https://github.com/jmg049/i24/blob/main/LICENSE) file for details.

use crate::repr::I24Repr;
use bytemuck::{NoUninit, Zeroable};
use num_traits::{Num, One, Zero};
use std::fmt;
use std::fmt::{Debug, Display, LowerHex, Octal, UpperHex};
use std::hash::{Hash, Hasher};
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};
use std::{
    ops::{Neg, Not},
    str::FromStr,
};

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

/// Represents errors that can occur when working with the `i24` type.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseI24Error {
    /// An error occurred while parsing a string to an `i24`.
    ///
    /// This variant wraps the standard library's `ParseIntError`.
    ParseError(std::num::ParseIntError),

    /// The value is out of the valid range for an `i24`.
    ///
    /// Valid range for `i24` is [-8,388,608, 8,388,607].
    OutOfRange,
}

impl Display for ParseI24Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseI24Error::ParseError(e) => write!(f, "Parse error: {}", e),
            ParseI24Error::OutOfRange => write!(f, "Value out of range for i24"),
        }
    }
}

impl std::error::Error for ParseI24Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseI24Error::ParseError(e) => Some(e),
            ParseI24Error::OutOfRange => None,
        }
    }
}

impl From<std::num::ParseIntError> for ParseI24Error {
    fn from(err: std::num::ParseIntError) -> Self {
        ParseI24Error::ParseError(err)
    }
}

mod repr;

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[cfg_attr(feature = "pyo3", pyclass)]
/// An experimental 24-bit signed integer type.
///
/// It should not be used anywhere important. It is still unverified and experimental.
///
/// The type is not yet fully implemented and is not guaranteed to work.
/// Supports basic arithmetic operations and conversions to and from ``i32``.
///
/// Represents a 24-bit signed integer.
///
/// This structs layout is an unspecified implementation detail
pub struct i24(I24Repr);

// Safety: repr(transparent) and so if I24Repr is Zeroable so should i24 be
unsafe impl Zeroable for i24 where I24Repr: Zeroable {}

// Safety: repr(transparent) and so if I24Repr is NoUninit so should i24 be
unsafe impl NoUninit for i24 where I24Repr: NoUninit {}


/// creates an `i24` from a constant expression
/// will give a compile error if the expression overflows an i24
#[macro_export]
macro_rules! i24 {
    ($e: expr) => {
        const {
            match $crate::i24::from_i32($e) {
                Some(x) => x,
                None => panic!(concat!(
                    "out of range value ",
                    stringify!($e),
                    " used as an i24 constant"
                )),
            }
        }
    };
}

impl i24 {
    /// The size of this integer type in bits
    pub const BITS: u32 = 24;

    /// The smallest value that can be represented by this integer type (-2<sup>23</sup>)
    pub const MIN: i24 = i24!(I24Repr::MIN);

    /// The largest value that can be represented by this integer type (2<sup>23</sup> − 1).
    pub const MAX: i24 = i24!(I24Repr::MAX);

    #[inline(always)]
    const fn as_bits(&self) -> &u32 {
        self.0.as_bits()
    }

    #[inline(always)]
    const fn to_bits(self) -> u32 {
        self.0.to_bits()
    }

    /// Safety: see `I24Repr::from_bits`
    #[inline(always)]
    const unsafe fn from_bits(bits: u32) -> i24 {
        Self(unsafe { I24Repr::from_bits(bits) })
    }

    /// same as `Self::from_bits` but always truncates
    #[inline(always)]
    const fn from_bits_truncate(bits: u32) -> i24 {
        // the most significant byte is zeroed out
        Self(unsafe { I24Repr::from_bits(bits & I24Repr::BITS_MASK) })
    }

    /// Converts the 24-bit integer to a 32-bit signed integer.
    ///
    /// This method performs sign extension if the 24-bit integer is negative.
    ///
    /// # Returns
    ///
    /// The 32-bit signed integer representation of this `i24`.
    #[inline(always)]
    pub const fn to_i32(self) -> i32 {
        self.0.to_i32()
    }

    /// Creates an `i24` from a 32-bit signed integer.
    ///
    /// # Arguments
    ///
    /// * `n` - The 32-bit signed integer to convert.
    ///
    /// # Returns
    ///
    /// Some(i24) if n is in the valid range
    #[inline(always)]
    pub const fn from_i32(n: i32) -> Option<Self> {
        match I24Repr::from_i32(n) {
            Some(inner) => Some(Self(inner)),
            None => None,
        }
    }

    /// Creates an `i24` from a 32-bit signed integer.
    ///
    /// This method truncates the input to 24 bits if it's outside the valid range.
    ///
    /// # Arguments
    ///
    /// * `n` - The 32-bit signed integer to convert.
    ///
    /// # Returns
    ///
    /// An `i24` instance representing the input value.
    #[inline(always)]
    pub const fn wrapping_from_i32(n: i32) -> Self {
        Self(I24Repr::wrapping_from_i32(n))
    }

    /// Reverses the byte order of the integer.
    #[inline(always)]
    pub const fn swap_bytes(self) -> Self {
        Self(self.0.swap_bytes())
    }

    /// Converts self to little endian from the target's endianness.
    /// On little endian this is a no-op. On big endian the bytes are swapped.
    #[inline(always)]
    pub const fn to_le(self) -> Self {
        Self(self.0.to_le())
    }

    /// Converts self to big endian from the target's endianness.
    /// On big endian this is a no-op. On little endian the bytes are swapped.
    #[inline(always)]
    pub const fn to_be(self) -> Self {
        Self(self.0.to_be())
    }

    /// Return the memory representation of this integer as a byte array in native byte order.
    /// As the target platform's native endianness is used,
    /// portable code should use to_be_bytes or to_le_bytes, as appropriate, instead.
    #[inline(always)]
    pub const fn to_ne_bytes(self) -> [u8; 3] {
        self.0.to_ne_bytes()
    }

    /// Create a native endian integer value from its representation as a byte array in little endian.
    #[inline(always)]
    pub const fn to_le_bytes(self) -> [u8; 3] {
        self.0.to_le_bytes()
    }

    /// Return the memory representation of this integer as a byte array in big-endian (network) byte order.
    #[inline(always)]
    pub const fn to_be_bytes(self) -> [u8; 3] {
        self.0.to_be_bytes()
    }

    /// Creates an `i24` from three bytes in **native endian** order.
    ///
    /// # Arguments
    ///
    /// * `bytes` - An array of 3 bytes representing the 24-bit integer.
    ///
    /// # Returns
    ///
    /// An `i24` instance containing the input bytes.
    #[inline(always)]
    pub const fn from_ne_bytes(bytes: [u8; 3]) -> Self {
        Self(I24Repr::from_ne_bytes(bytes))
    }

    /// Creates an `i24` from three bytes in **little-endian** order.
    ///
    /// # Arguments
    ///
    /// * `bytes` - An array of 3 bytes representing the 24-bit integer in little-endian order.
    ///
    /// # Returns
    ///
    /// An `i24` instance containing the input bytes.
    #[inline(always)]
    pub const fn from_le_bytes(bytes: [u8; 3]) -> Self {
        Self(I24Repr::from_le_bytes(bytes))
    }

    /// Creates an `i24` from three bytes in **big-endian** order.
    ///
    /// # Arguments
    ///
    /// * `bytes` - An array of 3 bytes representing the 24-bit integer in big-endian order.
    ///
    /// # Returns
    ///
    /// An `i24` instance with the bytes in little-endian order.
    #[inline(always)]
    pub const fn from_be_bytes(bytes: [u8; 3]) -> Self {
        Self(I24Repr::from_be_bytes(bytes))
    }

    /// Performs checked addition.
    ///
    /// # Arguments
    ///
    /// * `other` - The `i24` to add to this value.
    ///
    /// # Returns
    ///
    /// `Some(i24)` if the addition was successful, or `None` if it would overflow.
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.to_i32()
            .checked_add(other.to_i32())
            .and_then(Self::from_i32)
    }

    /// Performs checked subtraction.
    ///
    /// # Arguments
    ///
    /// * `other` - The `i24` to subtract from this value.
    ///
    /// # Returns
    ///
    /// `Some(i24)` if the subtraction was successful, or `None` if it would overflow.
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.to_i32()
            .checked_sub(other.to_i32())
            .and_then(Self::from_i32)
    }

    /// Performs checked multiplication.
    ///
    /// # Arguments
    ///
    /// * `other` - The `i24` to multiply with this value.
    ///
    /// # Returns
    ///
    /// `Some(i24)` if the multiplication was successful, or `None` if it would overflow.
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        self.to_i32()
            .checked_mul(other.to_i32())
            .and_then(Self::from_i32)
    }

    /// Performs checked division.
    ///
    /// # Arguments
    ///
    /// * `other` - The `i24` to divide this value by.
    ///
    /// # Returns
    ///
    /// `Some(i24)` if the division was successful, or `None` if the divisor is zero or if the division would overflow.
    pub fn checked_div(self, other: Self) -> Option<Self> {
        self.to_i32()
            .checked_div(other.to_i32())
            .and_then(Self::from_i32)
    }

    /// Performs checked integer remainder.
    ///
    /// # Arguments
    ///
    /// * `other` - The `i24` to divide this value by.
    ///
    /// # Returns
    ///
    /// `Some(i24)` if the remainder operation was successful, or `None` if the divisor is zero or if the division would overflow.
    pub fn checked_rem(self, other: Self) -> Option<Self> {
        self.to_i32()
            .checked_rem(other.to_i32())
            .and_then(Self::from_i32)
    }
}

impl One for i24 {
    fn one() -> Self {
        const {
            match i24::from_i32(1) {
                Some(x) => x,
                None => unreachable!(),
            }
        }
    }
}

impl Zero for i24 {
    #[inline(always)]
    fn zero() -> Self {
        Self::zeroed()
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        Self::zeroed() == *self
    }
}

macro_rules! from_str {
    ($meth: ident($($args: tt)*)) => {
        i32::$meth($($args)*)
            .map_err(ParseI24Error::ParseError)
            .and_then(|x| i24::from_i32(x).ok_or(ParseI24Error::OutOfRange))
    };
}

impl Num for i24 {
    type FromStrRadixErr = ParseI24Error;
    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        from_str!(from_str_radix(str, radix))
    }
}

impl FromStr for i24 {
    type Err = ParseI24Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        from_str!(from_str(str))
    }
}

#[cfg(feature = "pyo3")]
unsafe impl numpy::Element for i24 {
    const IS_COPY: bool = true;

    fn get_dtype_bound(py: Python<'_>) -> Bound<'_, numpy::PyArrayDescr> {
        numpy::dtype_bound::<i24>(py)
    }
}

macro_rules! impl_bin_op {
    ($(impl $op: ident = $assign: ident $assign_fn: ident { $($impl: tt)* })+) => {$(
        impl_bin_op!(impl $op = $assign $assign_fn for i24 { $($impl)* });
        impl_bin_op!(impl $op = $assign $assign_fn for &i24 { $($impl)* });
    )+};

    (impl $op: ident = $assign: ident $assign_fn: ident for $ty:ty {
         fn $meth: ident($self: tt, $other: ident) {
            $($impl: tt)*
         }
    }) => {
        impl $op<$ty> for i24 {
            type Output = Self;

            #[inline(always)]
            fn $meth($self, $other: $ty) -> Self {
                $($impl)*
            }
        }

        impl $op<$ty> for &i24 {
            type Output = i24;

            #[inline(always)]
            fn $meth(self, other: $ty) -> i24 {
                <i24 as $op<$ty>>::$meth(*self, other)
            }
        }

        impl $assign<$ty> for i24 {
            #[inline(always)]
            fn $assign_fn(&mut self, rhs: $ty) {
                *self = $op::$meth(*self, rhs)
            }
        }
    };
}

impl_bin_op! {
    impl Add = AddAssign add_assign {
        fn add(self, other) {
            // we use twos compliment and so signed and unsigned addition are strictly the same
            // so no need to cast to an i32
            Self::from_bits_truncate(self.to_bits().wrapping_add(other.to_bits()))
        }
    }

    impl Sub = SubAssign sub_assign {
        fn sub(self, other) {
            // we use twos compliment and so signed and unsigned subtraction are strictly the same
            // so no need to cast to an i32
            Self::from_bits_truncate(self.to_bits().wrapping_sub(other.to_bits()))
        }
    }

    impl Mul = MulAssign mul_assign {
        fn mul(self, other) {
            // we use twos compliment and so signed and unsigned non-widening multiplication are strictly the same
            // so no need to cast to an i32
            Self::from_bits_truncate(self.to_bits().wrapping_mul(other.to_bits()))
        }
    }

    impl Div = DivAssign div_assign {
        fn div(self, other) {
            let result = self.to_i32().wrapping_div(other.to_i32());
            Self::wrapping_from_i32(result)
        }
    }

    impl Rem = RemAssign rem_assign {
        fn rem(self, other) {
            let result = self.to_i32().wrapping_rem(other.to_i32());
            Self::wrapping_from_i32(result)
        }
    }


    impl BitAnd = BitAndAssign bitand_assign {
        fn bitand(self, rhs) {
            let bits = self.to_bits() & rhs.to_bits();
            // Safety:
            // since we and 2 values that both have the most significant byte set to zero
            // the output will always have the most significant byte set to zero
            unsafe { i24::from_bits(bits) }
        }
    }

    impl BitOr = BitOrAssign bitor_assign {
        fn bitor(self, rhs) {
            let bits = self.to_bits() | rhs.to_bits();
            // Safety:
            // since we and 2 values that both have the most significant byte set to zero
            // the output will always have the most significant byte set to zero
            unsafe { i24::from_bits(bits) }
        }
    }

    impl BitXor = BitXorAssign bitxor_assign {
        fn bitxor(self, rhs) {
            let bits = self.to_bits() ^ rhs.to_bits();
            // Safety:
            // since we and 2 values that both have the most significant byte set to zero
            // the output will always have the most significant byte set to zero
            unsafe { i24::from_bits(bits) }
        }
    }
}

impl Neg for i24 {
    type Output = Self;

    #[inline(always)]
    fn neg(self) -> Self {
        // this is how you negate twos compliment numbers
        i24::from_bits_truncate((!self.to_bits()) + 1)
    }
}

impl Not for i24 {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        i24::from_bits_truncate(!self.to_bits())
    }
}

impl Shl<u32> for i24 {
    type Output = Self;

    #[inline(always)]
    fn shl(self, rhs: u32) -> Self::Output {
        Self::from_bits_truncate(self.to_bits() << rhs)
    }
}

impl Shr<u32> for i24 {
    type Output = Self;

    #[inline(always)]
    fn shr(self, rhs: u32) -> Self::Output {
        // Safety:
        // we do a logical shift right by 8 at the end
        // and so the most significant octet/byte is set to 0

        // logic:
        // <8 bits empty> <i24 sign bit> <rest>
        // we shift everything up by 8
        // <i24 sign bit on i32 sign bit> <rest> <8 bits empty>
        // then we do a sign shift
        // <sign bit * n> <i24 sign bit> <rest> <8 - n bits empty>
        // after we shift everything down by 8
        // <8 bits empty> <sign bit * n> <sign bit> <first 23 - n bits of rest>
        unsafe { Self::from_bits(((self.to_bits() << 8) as i32 >> rhs) as u32 >> 8) }
    }
}

impl ShrAssign<u32> for i24 {
    #[inline(always)]
    fn shr_assign(&mut self, rhs: u32) {
        *self = Shr::shr(*self, rhs)
    }
}

impl ShlAssign<u32> for i24 {
    #[inline(always)]
    fn shl_assign(&mut self, rhs: u32) {
        *self = Shl::shl(*self, rhs)
    }
}

macro_rules! impl_fmt {
    ($(impl $name: path)+) => {$(
        impl $name for i24 {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <i32 as $name>::fmt(&self.to_i32(), f)
            }
        }
    )*};
}

macro_rules! impl_bits_fmt {
    ($(impl $name: path)+) => {$(
        impl $name for i24 {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <u32 as $name>::fmt(self.as_bits(), f)
            }
        }
    )*};
}

impl_fmt! {
    impl Display
    impl Debug
}

impl_bits_fmt! {
    impl UpperHex
    impl LowerHex

    impl Octal
    impl fmt::Binary
}

impl Hash for i24 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        I24Repr::hash(&self.0, state)
    }

    fn hash_slice<H: Hasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        // i24 is repr(transparent)
        I24Repr::hash_slice(
            unsafe { std::mem::transmute::<&[Self], &[I24Repr]>(data) },
            state,
        )
    }
}

#[cfg(test)]
mod i24_tests {
    use super::*;

    #[test]
    fn test_arithmetic_operations() {
        let a = i24!(100);
        let b = i24!(50);

        assert_eq!((a + b).to_i32(), 150);
        assert_eq!((a - b).to_i32(), 50);
        assert_eq!((a * b).to_i32(), 5000);
        assert_eq!((a / b).to_i32(), 2);
        assert_eq!((a % b).to_i32(), 0);
    }

    #[test]
    fn test_negative_operations() {
        let a = i24!(100);
        let b = i24!(-50);

        assert_eq!((a + b).to_i32(), 50);
        assert_eq!((a - b).to_i32(), 150);
        assert_eq!((a * b).to_i32(), -5000);
        assert_eq!((a / b).to_i32(), -2);
    }

    #[test]
    fn test_bitwise_operations() {
        let a = i24!(0b101010);
        let b = i24!(0b110011);

        assert_eq!((a & b).to_i32(), 0b100010);
        assert_eq!((a | b).to_i32(), 0b111011);
        assert_eq!((a ^ b).to_i32(), 0b011001);
        assert_eq!((a << 2).to_i32(), 0b10101000);
        assert_eq!((a >> 2).to_i32(), 0b1010);
    }

    #[test]
    fn test_checked_addition() {
        assert_eq!(i24!(10).checked_add(i24!(20)), Some(i24!(30)));
        assert_eq!(i24!(10).checked_add(i24!(-20)), Some(i24!(-10)));
        // Overflow cases
        assert_eq!(i24::MAX.checked_add(i24::one()), None);
        assert_eq!((i24::MAX - i24::one()).checked_add(i24::one() * i24!(2)), None);
    }

    #[test]
    fn test_checked_subtraction() {
        assert_eq!(i24!(10).checked_sub(i24!(20)), Some(i24!(-10)));
        assert_eq!(i24!(10).checked_sub(i24!(-20)), Some(i24!(30)));
        
        // Overflow cases
        assert_eq!(i24::MIN.checked_sub(i24::one()), None);
        assert_eq!((i24::MIN + i24::one()).checked_sub(i24::one() * i24!(2)), None);
    }

    #[test]
    fn test_checked_division() {
        assert_eq!(i24!(20).checked_div(i24!(5)), Some(i24!(4)));
        assert_eq!(i24!(20).checked_div(i24!(0)), None);
    }
    
    #[test]
    fn test_checked_multiplication() {
        assert_eq!(i24!(5).checked_mul(i24!(6)), Some(i24!(30)));
        assert_eq!(i24::MAX.checked_mul(i24!(2)), None);
    }
    
    #[test]
    fn test_checked_remainder() {
        assert_eq!(i24!(20).checked_rem(i24!(5)), Some(i24!(0)));
        assert_eq!(i24!(20).checked_rem(i24!(0)), None);
    }
    
    #[test]
    fn test_unary_operations() {
        let a = i24!(100);

        assert_eq!((-a).to_i32(), -100);
        assert_eq!((!a).to_i32(), -101);
    }

    #[test]
    fn test_from_i32() {
        assert_eq!(i24!(0).to_i32(), 0);
        assert_eq!(i24!(8388607).to_i32(), 8388607); // Max positive value
        assert_eq!(i24!(-8388608).to_i32(), -8388608); // Min negative value
    }

    #[test]
    fn test_from_bytes() {
        assert_eq!(i24::from_ne_bytes([0x01, 0x02, 0x03]).to_i32(), 0x030201);
        assert_eq!(i24::from_le_bytes([0x01, 0x02, 0x03]).to_i32(), 0x030201);
        assert_eq!(i24::from_be_bytes([0x01, 0x02, 0x03]).to_i32(), 0x010203);
    }

    #[test]
    fn test_to_i32() {
        let a = i24::from_ne_bytes([0xFF, 0xFF, 0x7F]);
        assert_eq!(a.to_i32(), 8388607); // Max positive value

        let b = i24::from_le_bytes([0x00, 0x00, 0x80]);
        assert_eq!(b.to_i32(), -8388608); // Min negative value
    }

    #[test]
    fn test_zero_and_one() {
        assert_eq!(i24::zero().to_i32(), 0);
        assert_eq!(i24::one().to_i32(), 1);
    }
    #[test]
    fn test_from_str() {
        assert_eq!(i24::from_str("100").unwrap().to_i32(), 100);
        assert_eq!(i24::from_str("-100").unwrap().to_i32(), -100);
        assert_eq!(
            i24::from_str(&format!("{}", i24::MAX)).unwrap().to_i32(),
            i24::MAX.to_i32()
        );
        assert_eq!(
            i24::from_str(&format!("{}", i24::MIN)).unwrap().to_i32(),
            i24::MIN.to_i32()
        );
        assert_eq!(
            i24::from_str("8388608").unwrap_err(),
            ParseI24Error::OutOfRange
        );
        assert_eq!(
            i24::from_str("-8388609").unwrap_err(),
            ParseI24Error::OutOfRange
        );
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", i24!(100)), "100");
        assert_eq!(format!("{}", i24!(-100)), "-100");
    }

    #[test]
    fn test_wrapping_behavior() {
        assert_eq!(i24::MAX + i24::one(), i24::MIN);
        assert_eq!(i24::MAX + i24::one() + i24::one(), i24::MIN + i24::one());

        assert_eq!(i24::MIN - i24::one(), i24::MAX);
        assert_eq!(i24::MIN - (i24::one() + i24::one()), i24::MAX - i24::one());

        assert_eq!(-i24::MIN, i24::MIN)
    }

    #[test]
    fn discriminant_optimization() {
        // this isn't guaranteed by rustc, but this should still hold true
        // if this fails because rustc stops doing it, just remove this test
        // otherwise check why this isn't working
        assert_eq!(size_of::<i24>(), size_of::<Option<i24>>());
        assert_eq!(size_of::<i24>(), size_of::<Option<Option<i24>>>());
        assert_eq!(size_of::<i24>(), size_of::<Option<Option<Option<i24>>>>());
        assert_eq!(
            size_of::<i24>(),
            size_of::<Option<Option<Option<Option<i24>>>>>()
        );
    }

    #[test]
    fn test_shift_operations() {
        let a = i24!(0b1);

        // Left shift
        assert_eq!((a << 23).to_i32(), -8388608); // 0x800000, which is the minimum negative value
        assert_eq!((a << 24).to_i32(), 0); // Shifts out all bits

        // Right shift
        let b = i24!(-1); // All bits set
        assert_eq!((b >> 1).to_i32(), -1); // Sign extension
        assert_eq!((b >> 23).to_i32(), -1); // Still all bits set due to sign extension
        assert_eq!((b >> 24).to_i32(), -1); // No change after 23 bits

        // Edge case: maximum positive value
        let c = i24!(0x7FFFFF); // 8388607
        assert_eq!((c << 1).to_i32(), -2); // 0xFFFFFE in 24-bit, which is -2 when sign-extended

        // Edge case: minimum negative value
        let d = i24::MIN; // (-0x800000)
        assert_eq!((d >> 1).to_i32(), -0x400000);
        assert_eq!((d >> 2).to_i32(), -0x200000);
        assert_eq!((d >> 3).to_i32(), -0x100000);
        assert_eq!((d >> 4).to_i32(), -0x080000);

        // Additional test for left shift wrapping
        assert_eq!((c << 1).to_i32(), -2); // 0xFFFFFE
        assert_eq!((c << 2).to_i32(), -4); // 0xFFFFFC
        assert_eq!((c << 3).to_i32(), -8); // 0xFFFFF8
    }
}
