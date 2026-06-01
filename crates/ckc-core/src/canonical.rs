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
/// - Numbers/strings use serde_json's deterministic itoa/ryu formatting:
///   byte-stable across runs and RFC 8785-equal for integers, but diverging
///   from RFC 8785's ECMAScript number form for some non-integer floats
///   (see the canonical-number known issue in `.agent/memory.md`). CKC's
///   contract is cross-run byte-stability, which this satisfies.
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
            let s = n.to_string();
            buf.extend_from_slice(s.as_bytes());
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
    fn content_hash_has_sha256_prefix_and_64_hex() {
        let h = content_hash(&json!({"hello": "world"}));
        assert!(h.as_str().starts_with("sha256:"));
        assert_eq!(h.as_str().len(), 7 + 64);
    }
}
