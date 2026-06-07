//! Property invariants for §1.3 rationals, §1.4 policies, and §1.5
//! canonical bytes.

use ckc_core::canon::{canonical_payload_bytes, from_canonical_bytes};
use ckc_core::outcome::OperationResult;
use ckc_core::policy::{RawSource, StringPolicy, Text};
use ckc_core::scalar::{Hash, Rational, UInt};
use num_bigint::{BigInt, BigUint};
use num_integer::Integer;
use num_traits::{One, Zero};
use proptest::prelude::*;

fn arb_result() -> impl Strategy<Value = OperationResult<UInt>> {
    let hashes = proptest::collection::btree_set(
        any::<Vec<u8>>().prop_map(|b| Hash::of_bytes(&b)),
        0..4usize,
    );
    prop_oneof![
        proptest::collection::vec(any::<u64>().prop_map(UInt::from), 1..4usize)
            .prop_map(OperationResult::Success),
        hashes.clone().prop_map(OperationResult::Residual),
        hashes.prop_map(OperationResult::Invalid),
    ]
}

proptest! {
    /// Every policy is idempotent and deterministic over arbitrary input;
    /// identifier_ascii additionally accepts only fixed points.
    #[test]
    fn policies_idempotent_and_deterministic(input in any::<String>()) {
        for policy in StringPolicy::ALL {
            match policy.normalize(&input) {
                Ok(once) => {
                    prop_assert_eq!(policy.normalize(&once).unwrap(), once.clone(), "{}", policy.id());
                    prop_assert_eq!(policy.normalize(&input).unwrap(), once, "{}", policy.id());
                }
                Err(_) => prop_assert_eq!(policy, StringPolicy::IdentifierAscii),
            }
        }
    }

    /// Rational::new output satisfies §1.3 invariants.
    #[test]
    fn rational_new_reduces(num in any::<i128>(), den in 1u64..) {
        let r = Rational::new(BigInt::from(num), BigUint::from(den)).unwrap();
        prop_assert!(!r.den().value().is_zero());
        if r.num().value().is_zero() {
            prop_assert!(r.den().value().is_one());
        } else {
            prop_assert!(r.num().value().magnitude().gcd(r.den().value()).is_one());
        }
    }

    /// Decimal conversion is exact base-10 place value.
    #[test]
    fn decimal_conversion_exact(int_part in 0u64..1_000_000, frac in 0u32..1000, neg in any::<bool>()) {
        let s = format!("{}{}.{:03}", if neg { "-" } else { "" }, int_part, frac);
        let scaled = i128::from(int_part) * 1000 + i128::from(frac);
        let signed = if neg { -scaled } else { scaled };
        let expected = Rational::new(BigInt::from(signed), BigUint::from(1000u32)).unwrap();
        prop_assert_eq!(Rational::from_decimal_str(&s).unwrap(), expected);
    }

    /// §1.5 injection: equal canonical bytes ⇔ equal values (tagged unions,
    /// hash sets, lists, scalars compose).
    #[test]
    fn canonical_bytes_injective(a in arb_result(), b in arb_result()) {
        let ba = canonical_payload_bytes(&a).unwrap();
        let bb = canonical_payload_bytes(&b).unwrap();
        prop_assert_eq!(ba == bb, a == b);
    }

    /// §1.5 strict reading accepts exactly the emitted canonical bytes.
    #[test]
    fn strict_read_roundtrips(x in arb_result()) {
        let bytes = canonical_payload_bytes(&x).unwrap();
        prop_assert_eq!(from_canonical_bytes::<OperationResult<UInt>>(&bytes).unwrap(), x);
    }

    /// §1.5 string encoding is injective over arbitrary content (raw_source
    /// admits every scalar value, including quotes, backslashes, controls).
    #[test]
    fn string_encoding_injective(a in any::<String>(), b in any::<String>()) {
        let ta = Text::<RawSource>::new(&a).unwrap();
        let tb = Text::<RawSource>::new(&b).unwrap();
        let ba = canonical_payload_bytes(&ta).unwrap();
        let bb = canonical_payload_bytes(&tb).unwrap();
        prop_assert_eq!(ba == bb, a == b);
    }
}
