use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

/// Content hash in the format `sha256:<hex>`.
#[derive(
    Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct ContentHash(pub String);

impl ContentHash {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Serialize a `serde_json::Value` to canonical JSON bytes.
///
/// Key properties:
/// - Object keys sorted by UTF-16 code-unit comparison (RFC 8785).
/// - No insignificant whitespace.
/// - Integers serialize via itoa (`to_string`); floats via `ryu-js`, the
///   ECMAScript `Number::toString` algorithm (ECMA-262 7.1.12.1) that RFC 8785
///   mandates -- so integer-valued floats render as `38`, not `38.0`, and the
///   exponent thresholds follow ECMAScript rather than Rust's `Display`.
///   Strings use serde_json's escaping. The result is full RFC 8785 number
///   conformance and cross-run byte-stability.
pub fn canonical_json_bytes(value: &Value) -> Vec<u8> {
    let mut buf = Vec::new();
    write_canonical(value, &mut buf);
    buf
}

/// Convenience: serialize any `Serialize` type to canonical JSON bytes.
pub fn to_canonical_bytes<T: Serialize>(value: &T) -> Vec<u8> {
    let v = serde_json::to_value(value).expect("CKC types must be serializable");
    canonical_json_bytes(&v)
}

/// Compute `sha256:<hex>` content hash of the canonical JSON form.
pub fn content_hash<T: Serialize>(value: &T) -> ContentHash {
    let bytes = to_canonical_bytes(value);
    let digest = Sha256::digest(&bytes);
    let mut hex = String::with_capacity(7 + 64);
    hex.push_str("sha256:");
    for b in digest.iter() {
        write!(hex, "{b:02x}").unwrap();
    }
    ContentHash(hex)
}

fn write_canonical(value: &Value, buf: &mut Vec<u8>) {
    match value {
        Value::Null => buf.extend_from_slice(b"null"),
        Value::Bool(true) => buf.extend_from_slice(b"true"),
        Value::Bool(false) => buf.extend_from_slice(b"false"),
        Value::Number(n) => {
            // RFC 8785 numbers: integers (i64/u64) already match ECMAScript's
            // Number::toString, so itoa via `to_string` is exact. Floats route
            // through ryu-js (ECMA-262 7.1.12.1), so integer-valued floats render
            // as "38" not "38.0". `Value::Number` is always finite, hence
            // `format_finite`.
            if n.is_f64() {
                let f = n.as_f64().expect("is_f64 implies a finite f64");
                let mut ryu = ryu_js::Buffer::new();
                buf.extend_from_slice(ryu.format_finite(f).as_bytes());
            } else {
                buf.extend_from_slice(n.to_string().as_bytes());
            }
        }
        Value::String(s) => {
            let escaped = serde_json::to_string(s).expect("string serializable");
            buf.extend_from_slice(escaped.as_bytes());
        }
        Value::Array(arr) => {
            buf.push(b'[');
            for (i, v) in arr.iter().enumerate() {
                if i > 0 {
                    buf.push(b',');
                }
                write_canonical(v, buf);
            }
            buf.push(b']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| utf16_cmp(a, b));
            buf.push(b'{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    buf.push(b',');
                }
                let escaped_key = serde_json::to_string(*key).expect("key serializable");
                buf.extend_from_slice(escaped_key.as_bytes());
                buf.push(b':');
                write_canonical(&map[*key], buf);
            }
            buf.push(b'}');
        }
    }
}

/// RFC 8785 key comparison: lexicographic order of UTF-16 code units.
fn utf16_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sorts_object_keys() {
        let v = json!({"c": 3, "a": 1, "b": 2});
        assert_eq!(canonical_json_bytes(&v), b"{\"a\":1,\"b\":2,\"c\":3}");
    }

    #[test]
    fn nested_objects_sorted_recursively() {
        let v = json!({"z": {"b": 2, "a": 1}, "a": []});
        assert_eq!(
            canonical_json_bytes(&v),
            b"{\"a\":[],\"z\":{\"a\":1,\"b\":2}}"
        );
    }

    #[test]
    fn no_whitespace() {
        let v = json!({"key": [1, 2, 3]});
        assert_eq!(canonical_json_bytes(&v), b"{\"key\":[1,2,3]}");
    }

    #[test]
    fn string_escaping() {
        let v = json!({"a": "hello\nworld"});
        assert_eq!(canonical_json_bytes(&v), br#"{"a":"hello\nworld"}"#);
    }

    #[test]
    fn japanese_keys_sort_by_utf16() {
        // Japanese identifiers in CKC data — JCS mandates UTF-16 code unit
        // ordering, distinct from byte ordering. Sort order matters.
        let v = json!({"薬": 1, "病": 2, "検": 3});
        let bytes = canonical_json_bytes(&v);
        assert_eq!(bytes, "{\"検\":3,\"病\":2,\"薬\":1}".as_bytes());
    }

    #[test]
    fn integer_valued_floats_drop_the_decimal() {
        // The RFC 8785 number bug this fix closes: ECMAScript Number::toString
        // renders 38.0 as "38". A literal with a decimal point parses as f64.
        let v: Value = serde_json::from_str("38.0").unwrap();
        assert!(v.is_f64(), "a decimal-point literal parses as f64");
        assert_eq!(canonical_json_bytes(&v), b"38");
    }

    #[test]
    fn floats_use_shortest_ecmascript_form() {
        assert_eq!(canonical_json_bytes(&json!(0.1)), b"0.1");
        assert_eq!(canonical_json_bytes(&json!(1.5)), b"1.5");
        assert_eq!(canonical_json_bytes(&json!(-0.5)), b"-0.5");
        // Magnitude thresholds follow ECMAScript, not Rust's Display: large and
        // small magnitudes switch to exponential with an explicit sign.
        let big: Value = serde_json::from_str("1e21").unwrap();
        assert_eq!(canonical_json_bytes(&big), b"1e+21");
        let small: Value = serde_json::from_str("1e-7").unwrap();
        assert_eq!(canonical_json_bytes(&small), b"1e-7");
    }

    #[test]
    fn integers_serialize_without_a_decimal() {
        assert_eq!(canonical_json_bytes(&json!(38)), b"38");
        assert_eq!(canonical_json_bytes(&json!(-7)), b"-7");
        assert_eq!(canonical_json_bytes(&json!(0)), b"0");
        assert_eq!(
            canonical_json_bytes(&json!(u64::MAX)),
            b"18446744073709551615"
        );
    }

    #[test]
    fn content_hash_has_sha256_prefix_and_64_hex() {
        let h = content_hash(&json!({"hello": "world"}));
        assert!(h.as_str().starts_with("sha256:"));
        assert_eq!(h.as_str().len(), 7 + 64);
    }
}
