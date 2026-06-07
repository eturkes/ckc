//! Gate T-Canonical-Bytes (§11.3): §1.5 serializer injection,
//! `canonical_sort_key` totality, tagged-union encoding, repeated hash
//! identity, plus §1.5 strict-reading rejection of JSON null, numeric
//! tokens, and duplicate object fields/map keys/union tags.

use std::collections::{BTreeMap, BTreeSet};

use ckc_core::canon::{
    BareValue, CanonError, Canonical, ObjectEmitter, ReadError, canonical_payload_bytes,
    canonical_sort_key, emit_list, emit_map, emit_set, emit_union, emit_union_bare,
    from_canonical_bytes, read_union,
};
use ckc_core::outcome::{OperationResult, Outcome};
use ckc_core::policy::{SemanticJa, Text};
use ckc_core::scalar::{Hash, Id, Int, Rational, UInt};
use serde::de::{Error as _, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

type Ja = Text<SemanticJa>;

fn ja(s: &str) -> Ja {
    Text::new(s).unwrap()
}

fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}

fn uint(v: u64) -> UInt {
    UInt::from(v)
}

// ---------------------------------------------------------------------------
// Fixtures: a record over every §1.5 production, and a bare-tag union
// ---------------------------------------------------------------------------

/// Test-local record covering sorted object members, optional-field
/// omission, set, map, list-in-union, string, and rational productions.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    label: Ja,
    note: Option<Id>,
    ratio: Rational,
    result: OperationResult<UInt>,
    stages: BTreeMap<Int, UInt>,
    tags: BTreeSet<Id>,
}

impl Canonical for Probe {
    const TYPE_ID: &'static str = "probe";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("label", |b| self.label.emit_canonical(b))?;
        if let Some(note) = &self.note {
            obj.member("note", |b| note.emit_canonical(b))?;
        }
        obj.member("ratio", |b| self.ratio.emit_canonical(b))?;
        obj.member("result", |b| self.result.emit_canonical(b))?;
        obj.member("stages", |b| emit_map(b, self.stages.iter()))?;
        obj.member("tags", |b| emit_set(b, self.tags.iter()))?;
        obj.finish(out)
    }
}

fn probe() -> Probe {
    Probe {
        label: ja("CRP高値,要観察"),
        note: None,
        ratio: Rational::from_decimal_str("38.5").unwrap(),
        result: OperationResult::Success(vec![uint(10), uint(2)]),
        stages: BTreeMap::from([(Int::from(-1), uint(0)), (Int::from(2), uint(38))]),
        tags: BTreeSet::from([id("a"), id("b:c")]),
    }
}

const PROBE_GOLDEN: &str = r#"{"label":"CRP高値,要観察","ratio":{"den":"2","num":"77"},"result":{"tag":"success","value":["10","2"]},"stages":{"-1":"0","2":"38"},"tags":["a","b:c"]}"#;

/// Repeated-hash-identity golden over `PROBE_GOLDEN` bytes.
const PROBE_HASH_GOLDEN: &str =
    "sha256:cd9872d4fef63a689a88e70109073a72507e3c169ad8c3c269e7b1cc3a68f33f";

/// Test-local union with a bare alternative (§1.5 bare-tag rule).
#[derive(Debug, PartialEq, Eq)]
enum MixedUnion {
    Empty,
    One(UInt),
}

impl Canonical for MixedUnion {
    const TYPE_ID: &'static str = "mixed_union";

    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        match self {
            Self::Empty => emit_union_bare(out, "empty"),
            Self::One(v) => emit_union(out, "one", |b| v.emit_canonical(b)),
        }
    }
}

impl<'de> Deserialize<'de> for MixedUnion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;

        impl<'de> Visitor<'de> for V {
            type Value = MixedUnion;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("mixed_union tagged-union object")
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<MixedUnion, A::Error> {
                read_union(map, |tag, map| match tag {
                    "empty" => {
                        map.next_value::<BareValue>()?;
                        Ok(MixedUnion::Empty)
                    }
                    "one" => Ok(MixedUnion::One(map.next_value()?)),
                    _ => Err(A::Error::unknown_variant(tag, &["empty", "one"])),
                })
            }
        }

        deserializer.deserialize_map(V)
    }
}

// ---------------------------------------------------------------------------
// §1.5 serializer injection
// ---------------------------------------------------------------------------

#[test]
fn golden_probe_bytes_and_strict_roundtrip() {
    let bytes = canonical_payload_bytes(&probe()).unwrap();
    assert_eq!(std::str::from_utf8(&bytes).unwrap(), PROBE_GOLDEN);
    assert_eq!(from_canonical_bytes::<Probe>(&bytes).unwrap(), probe());
}

#[test]
fn injection_separates_structure_from_content() {
    // Quoting separates element boundaries from content bytes.
    let mut one = Vec::new();
    emit_list(&mut one, &[ja("a,b")]).unwrap();
    let mut two = Vec::new();
    emit_list(&mut two, &[ja("a"), ja("b")]).unwrap();
    assert_ne!(one, two);
    // An escaped quote in content cannot fake a string boundary.
    let mut tricky = Vec::new();
    emit_list(&mut tricky, &[ja(r#"a","b"#)]).unwrap();
    assert_ne!(tricky, two);
    assert_eq!(tricky, br#"["a\",\"b"]"#.to_vec());
    // Optional omission is a distinct encoding from any present value.
    let with_note = Probe {
        note: Some(id("n1")),
        ..probe()
    };
    assert_ne!(
        canonical_payload_bytes(&probe()).unwrap(),
        canonical_payload_bytes(&with_note).unwrap()
    );
}

// ---------------------------------------------------------------------------
// canonical_sort_key totality
// ---------------------------------------------------------------------------

#[test]
fn sort_key_totality_across_types() {
    // One key per implemented type, including unions and records: all keys
    // mutually comparable, distinct values map to distinct keys.
    let keys = vec![
        canonical_sort_key(&true).unwrap(),
        canonical_sort_key(&Hash::of_bytes(b"")).unwrap(),
        canonical_sort_key(&id("a")).unwrap(),
        canonical_sort_key(&Int::from(38)).unwrap(),
        canonical_sort_key(&uint(38)).unwrap(),
        canonical_sort_key(&Rational::from_decimal_str("0.5").unwrap()).unwrap(),
        canonical_sort_key(&ja("CRP")).unwrap(),
        canonical_sort_key(&Outcome::Invalid).unwrap(),
        canonical_sort_key(&OperationResult::<UInt>::Unsupported(BTreeSet::new())).unwrap(),
        canonical_sort_key(&probe()).unwrap(),
    ];
    let mut sorted = keys.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), keys.len(), "distinct values, distinct keys");
    // Equal payload bytes under different declared types stay distinct.
    let int = canonical_sort_key(&Int::from(38)).unwrap();
    let nat = canonical_sort_key(&uint(38)).unwrap();
    assert_eq!(int.payload, nat.payload);
    assert_ne!(int, nat);
}

// ---------------------------------------------------------------------------
// Tagged-union encoding
// ---------------------------------------------------------------------------

#[test]
fn union_golden_bytes_and_enum_strings() {
    // List payload keeps semantic order.
    let success: OperationResult<UInt> = OperationResult::Success(vec![uint(10), uint(2)]);
    assert_eq!(
        canonical_payload_bytes(&success).unwrap(),
        br#"{"tag":"success","value":["10","2"]}"#.to_vec()
    );
    // Set payload sorts by canonical bytes regardless of insertion order.
    let (ha, hb) = (Hash::of_bytes(b"a"), Hash::of_bytes(b"b"));
    let residual: OperationResult<UInt> =
        OperationResult::Residual(BTreeSet::from([ha.clone(), hb.clone()]));
    let (lo, hi) = if ha.as_str() < hb.as_str() {
        (&ha, &hb)
    } else {
        (&hb, &ha)
    };
    assert_eq!(
        String::from_utf8(canonical_payload_bytes(&residual).unwrap()).unwrap(),
        format!(r#"{{"tag":"residual","value":["{lo}","{hi}"]}}"#)
    );
    // E-enum encoding decision: all-bare enums are strings, not tagged
    // objects; the persisted success name is ok (§1.7).
    assert_eq!(
        canonical_payload_bytes(&Outcome::Ok).unwrap(),
        br#""ok""#.to_vec()
    );
    assert_eq!(
        from_canonical_bytes::<Outcome>(br#""invalid""#).unwrap(),
        Outcome::Invalid
    );
    assert!(matches!(
        from_canonical_bytes::<Outcome>(br#""success""#),
        Err(ReadError::Parse { .. })
    ));
}

#[test]
fn bare_tag_encodes_empty_object_value() {
    let bytes = canonical_payload_bytes(&MixedUnion::Empty).unwrap();
    assert_eq!(bytes, br#"{"tag":"empty","value":{}}"#.to_vec());
    assert_eq!(
        from_canonical_bytes::<MixedUnion>(&bytes).unwrap(),
        MixedUnion::Empty
    );
    let one = canonical_payload_bytes(&MixedUnion::One(uint(7))).unwrap();
    assert_eq!(
        from_canonical_bytes::<MixedUnion>(&one).unwrap(),
        MixedUnion::One(uint(7))
    );
    // A bare value is exactly {}.
    assert!(matches!(
        from_canonical_bytes::<MixedUnion>(br#"{"tag":"empty","value":{"x":"1"}}"#),
        Err(ReadError::Parse { .. })
    ));
}

// ---------------------------------------------------------------------------
// Repeated hash identity (A.10)
// ---------------------------------------------------------------------------

#[test]
fn repeated_hash_identity_over_canonical_bytes() {
    let first = canonical_payload_bytes(&probe()).unwrap();
    let second = canonical_payload_bytes(&probe()).unwrap();
    assert_eq!(first, second, "repeated emission is byte-identical");
    assert_eq!(Hash::of_bytes(&first), Hash::of_bytes(&second));
    assert_eq!(Hash::of_bytes(&first).as_str(), PROBE_HASH_GOLDEN);
}

// ---------------------------------------------------------------------------
// §1.5 strict reading
// ---------------------------------------------------------------------------

/// One byte-level mutation of the golden probe bytes, guarded against
/// silently inapplicable patterns.
fn mutated(from: &str, to: &str) -> String {
    let out = PROBE_GOLDEN.replace(from, to);
    assert_ne!(out, PROBE_GOLDEN, "mutation must apply: {from}");
    out
}

#[test]
fn strict_reading_rejects_at_parse() {
    // §1.5: JSON null and numeric tokens on required fields, duplicate
    // object fields, unknown fields, duplicate/unknown/misordered union
    // tags, missing union value, empty success.
    let parse_rejects = [
        mutated(r#"{"den":"2","num":"77"}"#, "null"),
        mutated(r#"{"den":"2","num":"77"}"#, "38.5"),
        mutated(r#""den":"2""#, r#""den":2"#),
        mutated(
            r#""label":"CRP高値,要観察","#,
            r#""label":"x","label":"x","#,
        ),
        mutated(r#""tags":["a","b:c"]"#, r#""tags":["a","b:c"],"zz":"1""#),
        mutated(r#""tag":"success","#, r#""tag":"success","tag":"success","#),
        mutated(r#""tag":"success""#, r#""tag":"victory""#),
        mutated(
            r#"{"tag":"success","value":["10","2"]}"#,
            r#"{"value":["10","2"],"tag":"success"}"#,
        ),
        mutated(
            r#"{"tag":"success","value":["10","2"]}"#,
            r#"{"tag":"success"}"#,
        ),
        mutated(r#""value":["10","2"]"#, r#""value":[]"#),
    ];
    for bytes in &parse_rejects {
        assert!(
            matches!(
                from_canonical_bytes::<Probe>(bytes.as_bytes()),
                Err(ReadError::Parse { .. })
            ),
            "{bytes}"
        );
    }
    // Top-level null, numeric token, and non-canonical scalar strings reuse
    // the validating constructors (§1.3/§1.4).
    assert!(matches!(
        from_canonical_bytes::<UInt>(b"null"),
        Err(ReadError::Parse { .. })
    ));
    assert!(matches!(
        from_canonical_bytes::<UInt>(b"38"),
        Err(ReadError::Parse { .. })
    ));
    assert!(matches!(
        from_canonical_bytes::<UInt>(br#""007""#),
        Err(ReadError::Parse { .. })
    ));
    assert!(
        matches!(
            from_canonical_bytes::<Rational>(br#"{"den":"4","num":"2"}"#),
            Err(ReadError::Parse { .. })
        ),
        "Rational::from_canonical_parts rejects unreduced parts"
    );
    assert!(
        matches!(
            from_canonical_bytes::<Ja>("\"ＣＲＰ\"".as_bytes()),
            Err(ReadError::Parse { .. })
        ),
        "Text::from_canonical rejects unnormalized bytes"
    );
}

#[test]
fn strict_reading_rejects_non_canonical_re_serialization() {
    let backslash = 92u8 as char;
    let non_canonical = [
        // JSON null on an optional field reads as absent; absence re-emits
        // as omission.
        mutated(r#""label""#, r#""note":null,"label""#),
        // Duplicate map key collapses on read.
        mutated(
            r#""stages":{"-1":"0","2":"38"}"#,
            r#""stages":{"-1":"0","-1":"0","2":"38"}"#,
        ),
        // Record member order is canonical (parse tolerates, compare rejects).
        mutated(
            r#""label":"CRP高値,要観察","ratio":{"den":"2","num":"77"}"#,
            r#""ratio":{"den":"2","num":"77"},"label":"CRP高値,要観察""#,
        ),
        // Set elements are sorted and duplicates collapse.
        mutated(r#"["a","b:c"]"#, r#"["b:c","a"]"#),
        mutated(r#"["a","b:c"]"#, r#"["a","a","b:c"]"#),
        // Whitespace and trailing bytes are non-canonical.
        mutated(r#""tags":["a","b:c"]"#, r#""tags": ["a","b:c"]"#),
        format!("{PROBE_GOLDEN}\n"),
        // Unnecessary escape: the canonical encoding of C is the raw byte.
        mutated("CRP", &format!("{backslash}u0043RP")),
    ];
    for bytes in &non_canonical {
        assert!(
            matches!(
                from_canonical_bytes::<Probe>(bytes.as_bytes()),
                Err(ReadError::NonCanonical)
            ),
            "{bytes}"
        );
    }
}
