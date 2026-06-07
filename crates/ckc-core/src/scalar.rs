//! SPEC §1.3 scalar domains: `Id`, `Hash`, `UInt`, `Int`, `Rational`,
//! `FeaturePath`, and the `ProofId`/`RegionId` aliases.
//!
//! Canonical encodings: `UInt`/`Int` are decimal strings (`"0"` or
//! `[1-9][0-9]*`, optional leading `-`, `-0` rejected); `Rational` is a
//! reduced `{den,num}` object with `den > 0`. Deserialization validates
//! canonical form and rejects non-canonical bytes; constructors normalize.
//! Canonical bytes are emitted solely by `crate::canon` (§1.5); serde here
//! is the validating read side.

use std::fmt;

use num_bigint::{BigInt, BigUint, Sign};
use num_integer::Integer;
use num_traits::{One, Zero};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use sha2::{Digest, Sha256};

/// `ProofId` names a `ProofNode` (§1.3).
pub type ProofId = Id;
/// `RegionId` names a `SourceRegion` (§1.3).
pub type RegionId = Id;
/// `FeaturePath` is a `List[Id]` traversed over a schema-validated payload.
pub type FeaturePath = Vec<Id>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarError {
    IdSyntax { value: String },
    HashSyntax { value: String },
    UIntSyntax { value: String },
    IntSyntax { value: String },
    ZeroDenominator,
    NotReduced { num: String, den: String },
    DecimalSyntax { value: String },
    PercentSyntax { value: String },
}

impl fmt::Display for ScalarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdSyntax { value } => write!(f, "id syntax violation: {value:?}"),
            Self::HashSyntax { value } => write!(f, "hash syntax violation: {value:?}"),
            Self::UIntSyntax { value } => write!(f, "uint syntax violation: {value:?}"),
            Self::IntSyntax { value } => write!(f, "int syntax violation: {value:?}"),
            Self::ZeroDenominator => write!(f, "rational denominator is zero"),
            Self::NotReduced { num, den } => write!(f, "rational not reduced: {num}/{den}"),
            Self::DecimalSyntax { value } => write!(f, "decimal syntax violation: {value:?}"),
            Self::PercentSyntax { value } => write!(f, "percent syntax violation: {value:?}"),
        }
    }
}

impl std::error::Error for ScalarError {}

// ---------------------------------------------------------------------------
// Id
// ---------------------------------------------------------------------------

/// Lowercase ASCII identifier matching `[a-z][a-z0-9_:-]*` (§1.3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(String);

fn is_id(s: &str) -> bool {
    let bytes = s.as_bytes();
    let Some(first) = bytes.first() else {
        return false;
    };
    first.is_ascii_lowercase()
        && bytes[1..]
            .iter()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b':' | b'-'))
}

impl Id {
    pub fn new(value: &str) -> Result<Self, ScalarError> {
        if is_id(value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(ScalarError::IdSyntax {
                value: value.to_owned(),
            })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Id::new(&s).map_err(D::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Hash
// ---------------------------------------------------------------------------

/// `"sha256:"` followed by 64 lowercase hex digits (§1.3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash(String);

fn hex_lower(bytes: &[u8]) -> String {
    use fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
            write!(acc, "{b:02x}").expect("write to String is infallible");
            acc
        })
}

impl Hash {
    /// SHA-256 of `bytes`, rendered canonically.
    pub fn of_bytes(bytes: &[u8]) -> Self {
        Self(format!("sha256:{}", hex_lower(&Sha256::digest(bytes))))
    }

    pub fn parse(value: &str) -> Result<Self, ScalarError> {
        let err = || ScalarError::HashSyntax {
            value: value.to_owned(),
        };
        let digits = value.strip_prefix("sha256:").ok_or_else(err)?;
        if digits.len() == 64
            && digits
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        {
            Ok(Self(value.to_owned()))
        } else {
            Err(err())
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Hash::parse(&s).map_err(D::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// UInt / Int
// ---------------------------------------------------------------------------

fn is_canonical_uint(s: &str) -> bool {
    match s.as_bytes() {
        [] => false,
        [b'0'] => true,
        [first, rest @ ..] => matches!(first, b'1'..=b'9') && rest.iter().all(u8::is_ascii_digit),
    }
}

/// Nonnegative integer, encoded as canonical decimal string (§1.3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UInt(BigUint);

impl UInt {
    pub fn new(value: BigUint) -> Self {
        Self(value)
    }

    pub fn parse_canonical(s: &str) -> Result<Self, ScalarError> {
        if is_canonical_uint(s) {
            Ok(Self(s.parse().expect("canonical uint parses")))
        } else {
            Err(ScalarError::UIntSyntax {
                value: s.to_owned(),
            })
        }
    }

    pub fn value(&self) -> &BigUint {
        &self.0
    }

    pub fn to_decimal(&self) -> String {
        self.0.to_string()
    }
}

impl From<u64> for UInt {
    fn from(v: u64) -> Self {
        Self(BigUint::from(v))
    }
}

impl<'de> Deserialize<'de> for UInt {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        UInt::parse_canonical(&s).map_err(D::Error::custom)
    }
}

/// Integer, encoded as canonical decimal string with optional `-` (§1.3).
/// `-0` is non-canonical and rejected.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Int(BigInt);

impl Int {
    pub fn new(value: BigInt) -> Self {
        Self(value)
    }

    pub fn parse_canonical(s: &str) -> Result<Self, ScalarError> {
        let err = || ScalarError::IntSyntax {
            value: s.to_owned(),
        };
        let body = s.strip_prefix('-').unwrap_or(s);
        if !is_canonical_uint(body) || (s.starts_with('-') && body == "0") {
            return Err(err());
        }
        Ok(Self(s.parse().expect("canonical int parses")))
    }

    pub fn value(&self) -> &BigInt {
        &self.0
    }

    pub fn to_decimal(&self) -> String {
        self.0.to_string()
    }
}

impl From<i64> for Int {
    fn from(v: i64) -> Self {
        Self(BigInt::from(v))
    }
}

impl<'de> Deserialize<'de> for Int {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Int::parse_canonical(&s).map_err(D::Error::custom)
    }
}

// ---------------------------------------------------------------------------
// Rational
// ---------------------------------------------------------------------------

/// Exact reduced rational (§1.3): `den > 0`, `gcd(|num|, den) = 1`, zero is
/// `{num:"0", den:"1"}`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rational {
    den: UInt,
    num: Int,
}

/// Unreduced `(num, den)` from a decimal source form; shared by decimal and
/// percent conversion. Leading zeros are accepted in source forms: conversion
/// is exact base-10 place value, canonicality applies to the encoding.
fn decimal_parts(s: &str) -> Result<(BigInt, BigUint), ScalarError> {
    let err = || ScalarError::DecimalSyntax {
        value: s.to_owned(),
    };
    let (negative, body) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    let (int_digits, frac_digits) = match body.split_once('.') {
        Some((i, f)) => (i, f),
        None => (body, ""),
    };
    if int_digits.is_empty()
        || !int_digits.bytes().all(|b| b.is_ascii_digit())
        || (body.contains('.') && frac_digits.is_empty())
        || !frac_digits.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(err());
    }
    let mut digits = String::with_capacity(int_digits.len() + frac_digits.len());
    digits.push_str(int_digits);
    digits.push_str(frac_digits);
    let magnitude: BigUint = digits.parse().map_err(|_| err())?;
    let sign = if negative && !magnitude.is_zero() {
        Sign::Minus
    } else {
        Sign::Plus
    };
    let num = BigInt::from_biguint(sign, magnitude);
    let den = BigUint::from(10u32).pow(u32::try_from(frac_digits.len()).map_err(|_| err())?);
    Ok((num, den))
}

impl Rational {
    /// Normalizing constructor: reduces by gcd, canonicalizes zero.
    pub fn new(num: BigInt, den: BigUint) -> Result<Self, ScalarError> {
        if den.is_zero() {
            return Err(ScalarError::ZeroDenominator);
        }
        if num.is_zero() {
            return Ok(Self {
                den: UInt::new(BigUint::one()),
                num: Int::new(BigInt::zero()),
            });
        }
        let g = num.magnitude().gcd(&den);
        let reduced_num = BigInt::from_biguint(num.sign(), num.magnitude() / &g);
        Ok(Self {
            den: UInt::new(den / g),
            num: Int::new(reduced_num),
        })
    }

    /// Validating constructor for already-canonical parts; rejects
    /// non-reduced or zero-denominator input as non-canonical bytes.
    pub fn from_canonical_parts(num: Int, den: UInt) -> Result<Self, ScalarError> {
        if den.value().is_zero() {
            return Err(ScalarError::ZeroDenominator);
        }
        let reduced = if num.value().is_zero() {
            den.value().is_one()
        } else {
            num.value().magnitude().gcd(den.value()).is_one()
        };
        if reduced {
            Ok(Self { den, num })
        } else {
            Err(ScalarError::NotReduced {
                num: num.to_decimal(),
                den: den.to_decimal(),
            })
        }
    }

    /// Exact conversion of a decimal source form by base-10 place value
    /// (§1.3), e.g. `"38.5"` → `77/2`.
    pub fn from_decimal_str(s: &str) -> Result<Self, ScalarError> {
        let (num, den) = decimal_parts(s)?;
        Self::new(num, den)
    }

    /// Exact conversion of a percent source form by denominator
    /// multiplication by 100 (§1.3), e.g. `"20%"` → `1/5`.
    pub fn from_percent_str(s: &str) -> Result<Self, ScalarError> {
        let body = s
            .strip_suffix('%')
            .ok_or_else(|| ScalarError::PercentSyntax {
                value: s.to_owned(),
            })?;
        let (num, den) = decimal_parts(body).map_err(|_| ScalarError::PercentSyntax {
            value: s.to_owned(),
        })?;
        Self::new(num, den * BigUint::from(100u32))
    }

    pub fn zero() -> Self {
        Self {
            den: UInt::new(BigUint::one()),
            num: Int::new(BigInt::zero()),
        }
    }

    /// Canonical numerator part; `.value()` reaches the `BigInt`.
    pub fn num(&self) -> &Int {
        &self.num
    }

    /// Canonical denominator part; `.value()` reaches the `BigUint`.
    pub fn den(&self) -> &UInt {
        &self.den
    }
}

impl<'de> Deserialize<'de> for Rational {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            den: UInt,
            num: Int,
        }
        let wire = Wire::deserialize(deserializer)?;
        Rational::from_canonical_parts(wire.num, wire.den).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rational(num: i64, den: u64) -> Rational {
        Rational::new(BigInt::from(num), BigUint::from(den)).unwrap()
    }

    #[test]
    fn id_syntax() {
        for ok in ["a", "upm-m0", "z9:_-x"] {
            assert!(Id::new(ok).is_ok(), "{ok}");
        }
        for bad in ["", "A", "9a", "a B", "a.b", "ＡＢ", "-a"] {
            assert!(Id::new(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn hash_known_answer_and_syntax() {
        assert_eq!(
            Hash::of_bytes(b"").as_str(),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert!(Hash::parse(Hash::of_bytes(b"x").as_str()).is_ok());
        for bad in [
            "sha256:ABC",
            "sha256:e3b0",
            "md5:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ] {
            assert!(Hash::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn uint_int_canonical_form() {
        assert!(UInt::parse_canonical("0").is_ok());
        assert!(UInt::parse_canonical("10").is_ok());
        for bad in ["", "00", "01", "-1", "1 "] {
            assert!(UInt::parse_canonical(bad).is_err(), "{bad}");
        }
        assert!(Int::parse_canonical("-3").is_ok());
        assert!(Int::parse_canonical("0").is_ok());
        for bad in ["-0", "007", "+1", ""] {
            assert!(Int::parse_canonical(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn rational_normalization() {
        assert_eq!(rational(6, 8), rational(3, 4));
        assert_eq!(rational(-6, 8), rational(-3, 4));
        assert_eq!(rational(0, 7), Rational::zero());
        assert!(Rational::new(BigInt::from(1), BigUint::zero()).is_err());
    }

    #[test]
    fn rational_decimal_conversion_is_exact() {
        assert_eq!(Rational::from_decimal_str("38.5").unwrap(), rational(77, 2));
        assert_eq!(Rational::from_decimal_str("90").unwrap(), rational(90, 1));
        assert_eq!(Rational::from_decimal_str("0.50").unwrap(), rational(1, 2));
        assert_eq!(Rational::from_decimal_str("08").unwrap(), rational(8, 1));
        assert_eq!(
            Rational::from_decimal_str("-0.0").unwrap(),
            Rational::zero()
        );
        for bad in ["", "-", ".5", "5.", "1e3", "+1", "1.2.3", "38。5"] {
            assert!(Rational::from_decimal_str(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn rational_percent_conversion_is_exact() {
        assert_eq!(Rational::from_percent_str("20%").unwrap(), rational(1, 5));
        assert_eq!(
            Rational::from_percent_str("0.5%").unwrap(),
            rational(1, 200)
        );
        assert_eq!(Rational::from_percent_str("100%").unwrap(), rational(1, 1));
        for bad in ["%", "5", "5 %", "20%%"] {
            assert!(Rational::from_percent_str(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn rational_deserialize_validates_canonical_parts() {
        assert!(serde_json::from_str::<Rational>(r#"{"den":"2","num":"77"}"#).is_ok());
        for bad in [
            r#"{"den":"4","num":"2"}"#,
            r#"{"den":"0","num":"1"}"#,
            r#"{"den":"2","num":"-0"}"#,
            r#"{"den":"7","num":"0"}"#,
        ] {
            assert!(serde_json::from_str::<Rational>(bad).is_err(), "{bad}");
        }
    }
}
