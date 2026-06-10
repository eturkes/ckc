//! Foundational value types of the domain model (SPEC §4.1).
//!
//! ```text
//! Id       = lowercase ASCII matching [a-z][a-z0-9_.:-]*
//! Hash     = "sha256:" + 64 lowercase hex digits
//! Rational = exact reduced { "num": "<int>", "den": "<positive-int>" }
//! ```
//!
//! Each type validates on construction and on deserialization, so an in-memory
//! value always satisfies its grammar. Integers are arbitrary-precision and
//! serialized as decimal strings; JSON numeric tokens are rejected (SPEC §4.3).

use std::fmt;
use std::str::FromStr;

use num_bigint::BigInt;
use num_rational::BigRational;
use serde::{Deserialize, Serialize};

/// Construction/validation failure for a SPEC §4.1 value-type contract. The
/// variant names the offending type; the message carries the diagnostic
/// detail. Later units extend this with their own contract variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// An [`Id`] violated `[a-z][a-z0-9_.:-]*`.
    Id(String),
    /// A [`struct@Hash`] was not `"sha256:"` + 64 lowercase hex digits.
    Hash(String),
    /// A [`Rational`] had a non-integer part or a zero denominator.
    Rational(String),
    /// A string failed its declared [`crate::StringPolicy`] (SPEC §4.2), e.g.
    /// `identifier_ascii` received bytes outside `[a-z0-9_:./-]+`.
    StringPolicy(String),
    /// A token named no value of its fieldless enum (SPEC §4.4 value sets).
    Enum(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::Id(m) => write!(f, "invalid Id: {m}"),
            ValidationError::Hash(m) => write!(f, "invalid Hash: {m}"),
            ValidationError::Rational(m) => write!(f, "invalid Rational: {m}"),
            ValidationError::StringPolicy(m) => write!(f, "invalid string policy: {m}"),
            ValidationError::Enum(m) => write!(f, "invalid enum value: {m}"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// A stable lowercase-ASCII identifier matching `[a-z][a-z0-9_.:-]*` (SPEC §4.1).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Id(String);

impl Id {
    /// Validate `value` against the ID grammar and wrap it.
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let mut chars = value.chars();
        match chars.next() {
            Some(c) if c.is_ascii_lowercase() => {}
            _ => {
                return Err(ValidationError::Id(format!(
                    "must start with [a-z]: {value:?}"
                )));
            }
        }
        for c in chars {
            if !(c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | ':' | '-'))
            {
                return Err(ValidationError::Id(format!(
                    "character {c:?} is not in [a-z0-9_.:-]: {value:?}"
                )));
            }
        }
        Ok(Id(value))
    }

    /// Borrow the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Id {
    type Error = ValidationError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Id::new(value)
    }
}

impl FromStr for Id {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Id::new(s)
    }
}

impl From<Id> for String {
    fn from(value: Id) -> Self {
        value.0
    }
}

/// A content hash, `"sha256:"` followed by 64 lowercase hex digits (SPEC §4.1).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Hash(String);

impl Hash {
    /// Length of the lowercase-hex digest body (sha256 = 256 bits = 64 nibbles).
    const HEX_LEN: usize = 64;

    /// Validate `value` against the hash grammar and wrap it.
    pub fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let hex = value.strip_prefix("sha256:").ok_or_else(|| {
            ValidationError::Hash(format!("missing \"sha256:\" prefix: {value:?}"))
        })?;
        if hex.len() != Self::HEX_LEN {
            return Err(ValidationError::Hash(format!(
                "expected {} hex digits, found {}: {value:?}",
                Self::HEX_LEN,
                hex.len()
            )));
        }
        if let Some(c) = hex.chars().find(|&c| !matches!(c, '0'..='9' | 'a'..='f')) {
            return Err(ValidationError::Hash(format!(
                "non lowercase-hex digit {c:?}: {value:?}"
            )));
        }
        Ok(Hash(value))
    }

    /// Borrow the full `"sha256:..."` string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Hash {
    type Error = ValidationError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Hash::new(value)
    }
}

impl FromStr for Hash {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Hash::new(s)
    }
}

impl From<Hash> for String {
    fn from(value: Hash) -> Self {
        value.0
    }
}

/// Wire form of a [`Rational`]: `{ "num": "<int>", "den": "<positive-int>" }`
/// with each integer encoded as a decimal string (SPEC §4.1, §4.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RationalRepr {
    /// Numerator as a decimal integer string (may be negative or zero).
    pub num: String,
    /// Denominator as a decimal integer string (positive in well-formed input).
    pub den: String,
}

/// An exact reduced rational (SPEC §4.1), always stored gcd-normalized with a
/// positive denominator and backed by arbitrary-precision integers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "RationalRepr", into = "RationalRepr")]
pub struct Rational(BigRational);

impl Rational {
    /// Build from arbitrary-precision parts, reducing to lowest terms and moving
    /// any sign onto the numerator. A zero denominator is rejected.
    pub fn new(num: BigInt, den: BigInt) -> Result<Self, ValidationError> {
        if den == BigInt::from(0) {
            return Err(ValidationError::Rational("denominator is zero".to_string()));
        }
        Ok(Rational(BigRational::new(num, den)))
    }

    /// Parse decimal-string parts (the wire form) into a reduced rational.
    pub fn from_parts(num: &str, den: &str) -> Result<Self, ValidationError> {
        let n = num
            .parse::<BigInt>()
            .map_err(|e| ValidationError::Rational(format!("numerator {num:?}: {e}")))?;
        let d = den
            .parse::<BigInt>()
            .map_err(|e| ValidationError::Rational(format!("denominator {den:?}: {e}")))?;
        Rational::new(n, d)
    }

    /// Borrow the reduced numerator.
    pub fn numer(&self) -> &BigInt {
        self.0.numer()
    }

    /// Borrow the reduced (positive) denominator.
    pub fn denom(&self) -> &BigInt {
        self.0.denom()
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.0.numer(), self.0.denom())
    }
}

impl TryFrom<RationalRepr> for Rational {
    type Error = ValidationError;
    fn try_from(value: RationalRepr) -> Result<Self, Self::Error> {
        Rational::from_parts(&value.num, &value.den)
    }
}

impl From<Rational> for RationalRepr {
    fn from(value: Rational) -> Self {
        RationalRepr {
            num: value.0.numer().to_string(),
            den: value.0.denom().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_accepts_valid() {
        for s in [
            "a",
            "method.foo",
            "pipe.layered_ckcir_to_smt",
            "x0:y-z_w.v",
            "schema.ir_bundle",
        ] {
            assert!(Id::new(s).is_ok(), "{s:?} should be valid");
        }
    }

    #[test]
    fn id_rejects_invalid() {
        for s in [
            "",
            "0x",
            "Foo",
            "_lead",
            ".lead",
            "-lead",
            ":lead",
            "has space",
            "a/b",
            "café",
        ] {
            assert!(
                matches!(Id::new(s), Err(ValidationError::Id(_))),
                "{s:?} should be invalid"
            );
        }
    }

    #[test]
    fn id_serde_roundtrip_and_validates() {
        let id = Id::new("pipe.direct_rule_to_smt").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"pipe.direct_rule_to_smt\"");
        assert_eq!(serde_json::from_str::<Id>(&json).unwrap(), id);
        assert!(serde_json::from_str::<Id>("\"Bad Id\"").is_err());
    }

    const H: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn hash_accepts_valid() {
        assert!(Hash::new(H).is_ok());
        assert!(
            Hash::new("sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789")
                .is_ok()
        );
    }

    #[test]
    fn hash_rejects_invalid() {
        assert!(Hash::new("").is_err()); // empty
        assert!(Hash::new("md5:0123").is_err()); // wrong prefix
        assert!(Hash::new("sha256:abc").is_err()); // too short
        assert!(Hash::new(format!("{H}0")).is_err()); // too long
        assert!(Hash::new(format!("sha256:{}", "ABCDEF0123456789".repeat(4))).is_err()); // uppercase
        assert!(Hash::new(format!("sha256:{}", "g".repeat(64))).is_err()); // non-hex
    }

    #[test]
    fn hash_serde_roundtrip() {
        let h = Hash::new(H).unwrap();
        let json = serde_json::to_string(&h).unwrap();
        assert_eq!(json, format!("\"{H}\""));
        assert_eq!(serde_json::from_str::<Hash>(&json).unwrap(), h);
    }

    #[test]
    fn rational_reduces_and_normalizes_sign() {
        let cases = [
            ("2", "4", 1, 2),
            ("1", "-2", -1, 2),
            ("-3", "-6", 1, 2),
            ("0", "5", 0, 1),
            ("6", "-4", -3, 2),
        ];
        for (num, den, want_n, want_d) in cases {
            let r = Rational::from_parts(num, den).unwrap();
            assert_eq!(r.numer(), &BigInt::from(want_n), "{num}/{den} numer");
            assert_eq!(r.denom(), &BigInt::from(want_d), "{num}/{den} denom");
        }
    }

    #[test]
    fn rational_rejects_zero_denominator() {
        assert!(matches!(
            Rational::from_parts("5", "0"),
            Err(ValidationError::Rational(_))
        ));
        assert!(matches!(
            Rational::new(BigInt::from(5), BigInt::from(0)),
            Err(ValidationError::Rational(_))
        ));
    }

    #[test]
    fn rational_rejects_non_integer_parts() {
        assert!(Rational::from_parts("abc", "1").is_err());
        assert!(Rational::from_parts("1", "").is_err());
        assert!(Rational::from_parts("1.5", "2").is_err());
        assert!(Rational::from_parts("1", "2.0").is_err());
    }

    #[test]
    fn rational_serde_reduces_to_object() {
        let r: Rational = serde_json::from_str(r#"{"num":"2","den":"4"}"#).unwrap();
        let repr: RationalRepr = serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
        assert_eq!(repr.num, "1");
        assert_eq!(repr.den, "2");
    }

    #[test]
    fn rational_serde_rejects_numeric_tokens() {
        // SPEC §4.3: JSON numeric tokens are rejected; the wire form is string-encoded.
        assert!(serde_json::from_str::<Rational>(r#"{"num":2,"den":4}"#).is_err());
    }

    #[test]
    fn rational_handles_big_integers() {
        let big = "123456789012345678901234567890";
        let r = Rational::from_parts(big, "1").unwrap();
        assert_eq!(r.numer(), &big.parse::<BigInt>().unwrap());
        assert_eq!(r.denom(), &BigInt::from(1));
    }

    #[test]
    fn rational_display() {
        assert_eq!(Rational::from_parts("6", "-4").unwrap().to_string(), "-3/2");
    }
}
