//! Property invariants for §1.3 rationals and §1.4 policies.

use ckc_core::policy::StringPolicy;
use ckc_core::scalar::Rational;
use num_bigint::{BigInt, BigUint};
use num_integer::Integer;
use num_traits::{One, Zero};
use proptest::prelude::*;

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
        prop_assert!(!r.den().is_zero());
        if r.num().is_zero() {
            prop_assert!(r.den().is_one());
        } else {
            prop_assert!(r.num().magnitude().gcd(r.den()).is_one());
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
}
