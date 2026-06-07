//! SPEC §1.7 `OperationResult<T>` and §2 `Outcome`: the first CKC unions.
//!
//! `OperationResult<T>` is the typed implementation generic for the §1.7
//! total-function convention — a payload-carrying tagged union under the
//! §1.5 `{"tag","value"}` encoding. The persisted `Outcome` enum uses `ok`
//! for `success` and the same names for non-success statuses; it is an
//! all-bare `E` enum and string-encodes per the `crate::canon` encoding
//! decision.

use std::collections::BTreeSet;
use std::fmt;
use std::marker::PhantomData;

use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::canon::{
    CanonError, Canonical, emit_list, emit_set, emit_string, emit_union, read_union,
};
use crate::scalar::Hash;

// ---------------------------------------------------------------------------
// Outcome (§2)
// ---------------------------------------------------------------------------

const OUTCOME_IDS: [&str; 6] = [
    "ok",
    "residual",
    "ambiguity",
    "incoherence",
    "unsupported",
    "invalid",
];

/// §2 `E Outcome`, in listing order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    Ok,
    Residual,
    Ambiguity,
    Incoherence,
    Unsupported,
    Invalid,
}

impl Outcome {
    /// §2 listing order.
    pub const ALL: [Self; 6] = [
        Self::Ok,
        Self::Residual,
        Self::Ambiguity,
        Self::Incoherence,
        Self::Unsupported,
        Self::Invalid,
    ];

    /// Variant symbol exactly as SPEC writes it.
    pub fn id(self) -> &'static str {
        OUTCOME_IDS[self as usize]
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|o| o.id() == id)
    }

    /// §1.7 primary-status selection rank, ascending:
    /// `invalid > incoherence > unsupported > ambiguity > residual > ok`.
    /// That order differs from the §2 listing (`incoherence` listed before
    /// `unsupported`), so the rank is explicit rather than a derived `Ord`;
    /// choose the primary status of multiple emitted facts by
    /// `max_by_key(Outcome::primacy)`.
    pub fn primacy(self) -> u8 {
        match self {
            Self::Ok => 0,
            Self::Residual => 1,
            Self::Ambiguity => 2,
            Self::Unsupported => 3,
            Self::Incoherence => 4,
            Self::Invalid => 5,
        }
    }
}

impl Canonical for Outcome {
    const TYPE_ID: &'static str = "outcome";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        emit_string(out, self.id());
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Outcome {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_id(&s).ok_or_else(|| D::Error::unknown_variant(&s, &OUTCOME_IDS))
    }
}

// ---------------------------------------------------------------------------
// OperationResult (§1.7)
// ---------------------------------------------------------------------------

const RESULT_TAGS: [&str; 6] = [
    "success",
    "residual",
    "ambiguity",
    "incoherence",
    "unsupported",
    "invalid",
];

/// §1.7 `E OperationResult[T]`, the typed implementation generic for the
/// total-function convention. `success` carries one or more canonical
/// values or accepted payload hashes of type `T`; the other variants carry
/// the §1.7-named hash sets, emitted sorted by `canonical_sort_key`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationResult<T> {
    /// One or more values (§1.7); the strict reader rejects an empty list.
    Success(Vec<T>),
    Residual(BTreeSet<Hash>),
    Ambiguity(BTreeSet<Hash>),
    Incoherence(BTreeSet<Hash>),
    Unsupported(BTreeSet<Hash>),
    Invalid(BTreeSet<Hash>),
}

impl<T> OperationResult<T> {
    /// §1.5 constructor tag.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Success(_) => "success",
            Self::Residual(_) => "residual",
            Self::Ambiguity(_) => "ambiguity",
            Self::Incoherence(_) => "incoherence",
            Self::Unsupported(_) => "unsupported",
            Self::Invalid(_) => "invalid",
        }
    }

    /// §1.7 persisted status: `ok` for `success`, same names otherwise.
    pub fn outcome(&self) -> Outcome {
        match self {
            Self::Success(_) => Outcome::Ok,
            Self::Residual(_) => Outcome::Residual,
            Self::Ambiguity(_) => Outcome::Ambiguity,
            Self::Incoherence(_) => Outcome::Incoherence,
            Self::Unsupported(_) => Outcome::Unsupported,
            Self::Invalid(_) => Outcome::Invalid,
        }
    }
}

impl<T: Canonical> Canonical for OperationResult<T> {
    /// Provisional flat id; instantiation-level symbol ids land with the
    /// M0.0.3 registry.
    const TYPE_ID: &'static str = "operation_result";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        debug_assert!(
            !matches!(self, Self::Success(v) if v.is_empty()),
            "§1.7 success carries one or more values"
        );
        emit_union(out, self.tag(), |b| match self {
            Self::Success(values) => emit_list(b, values),
            Self::Residual(h)
            | Self::Ambiguity(h)
            | Self::Incoherence(h)
            | Self::Unsupported(h)
            | Self::Invalid(h) => emit_set(b, h),
        })
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for OperationResult<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for V<T> {
            type Value = OperationResult<T>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("§1.7 OperationResult tagged-union object")
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                read_union(map, |tag, map| match tag {
                    "success" => {
                        let values: Vec<T> = map.next_value()?;
                        if values.is_empty() {
                            return Err(A::Error::custom(
                                "§1.7 success carries one or more values",
                            ));
                        }
                        Ok(OperationResult::Success(values))
                    }
                    "residual" => Ok(OperationResult::Residual(map.next_value()?)),
                    "ambiguity" => Ok(OperationResult::Ambiguity(map.next_value()?)),
                    "incoherence" => Ok(OperationResult::Incoherence(map.next_value()?)),
                    "unsupported" => Ok(OperationResult::Unsupported(map.next_value()?)),
                    "invalid" => Ok(OperationResult::Invalid(map.next_value()?)),
                    _ => Err(A::Error::unknown_variant(tag, &RESULT_TAGS)),
                })
            }
        }

        deserializer.deserialize_map(V(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_ids_roundtrip_and_primacy_orders() {
        for o in Outcome::ALL {
            assert_eq!(Outcome::from_id(o.id()), Some(o));
        }
        assert_eq!(Outcome::from_id("success"), None, "persisted name is ok");
        // §1.7: invalid > incoherence > unsupported > ambiguity > residual > ok.
        let mut by_primacy = Outcome::ALL;
        by_primacy.sort_by_key(|o| o.primacy());
        assert_eq!(
            by_primacy,
            [
                Outcome::Ok,
                Outcome::Residual,
                Outcome::Ambiguity,
                Outcome::Unsupported,
                Outcome::Incoherence,
                Outcome::Invalid,
            ]
        );
    }

    #[test]
    fn operation_result_maps_tags_and_outcomes() {
        let success: OperationResult<Hash> = OperationResult::Success(vec![Hash::of_bytes(b"x")]);
        assert_eq!(success.tag(), "success");
        assert_eq!(success.outcome(), Outcome::Ok);
        let invalid: OperationResult<Hash> = OperationResult::Invalid(BTreeSet::new());
        assert_eq!(invalid.tag(), "invalid");
        assert_eq!(invalid.outcome(), Outcome::Invalid);
    }
}
