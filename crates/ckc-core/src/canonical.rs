use crate::id::ContentHash;
use serde::Serialize;
use serde_json::Value;

/// Serialize a value to RFC 8785 (JCS) canonical JSON bytes.
///
/// Guarantees: deterministic key ordering (UTF-16 code unit sort),
/// no insignificant whitespace, zero-normalization for IEEE 754 ±0.
pub fn to_canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let v = serde_json::to_value(value)?;
    let mut buf = Vec::new();
    write_canonical(&v, &mut buf);
    Ok(buf)
}

/// Compute SHA-256 content hash of the canonical JSON representation.
pub fn content_hash_of<T: Serialize>(value: &T) -> Result<ContentHash, serde_json::Error> {
    let bytes = to_canonical_json(value)?;
    Ok(ContentHash::from_bytes(&bytes))
}

fn write_canonical(value: &Value, buf: &mut Vec<u8>) {
    match value {
        Value::Null => buf.extend_from_slice(b"null"),
        Value::Bool(true) => buf.extend_from_slice(b"true"),
        Value::Bool(false) => buf.extend_from_slice(b"false"),
        Value::Number(n) => {
            // RFC 8785 §3.2.2.3: both -0 and 0.0 serialize as "0"
            if n.as_f64() == Some(0.0) {
                buf.push(b'0');
            } else {
                buf.extend_from_slice(n.to_string().as_bytes());
            }
        }
        Value::String(s) => {
            buf.extend_from_slice(serde_json::to_string(s).unwrap().as_bytes());
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
            buf.push(b'{');
            let mut keys: Vec<&String> = map.keys().collect();
            // RFC 8785: sort keys by UTF-16 code unit values
            keys.sort_by(|a, b| {
                a.encode_utf16()
                    .collect::<Vec<u16>>()
                    .cmp(&b.encode_utf16().collect::<Vec<u16>>())
            });
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    buf.push(b',');
                }
                buf.extend_from_slice(serde_json::to_string(*key).unwrap().as_bytes());
                buf.push(b':');
                write_canonical(&map[*key], buf);
            }
            buf.push(b'}');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sorted_keys() {
        let v = json!({"c": 3, "a": 1, "b": 2});
        let bytes = to_canonical_json(&v).unwrap();
        assert_eq!(bytes, br#"{"a":1,"b":2,"c":3}"#);
    }

    #[test]
    fn no_whitespace() {
        let v = json!({"key": [1, 2, 3]});
        let s = String::from_utf8(to_canonical_json(&v).unwrap()).unwrap();
        assert!(!s.contains(' '));
        assert!(!s.contains('\n'));
        assert!(!s.contains('\t'));
    }

    #[test]
    fn nested_objects_sorted() {
        let v = json!({"z": {"b": 1, "a": 0}, "a": true});
        let bytes = to_canonical_json(&v).unwrap();
        assert_eq!(bytes, br#"{"a":true,"z":{"a":0,"b":1}}"#);
    }

    #[test]
    fn null_and_bool() {
        let v = json!({"t": true, "n": null, "f": false});
        let bytes = to_canonical_json(&v).unwrap();
        assert_eq!(bytes, br#"{"f":false,"n":null,"t":true}"#);
    }

    #[test]
    fn array_order_preserved() {
        let v = json!([3, 1, 2]);
        let bytes = to_canonical_json(&v).unwrap();
        assert_eq!(bytes, b"[3,1,2]");
    }

    #[test]
    fn empty_containers() {
        assert_eq!(to_canonical_json(&json!({})).unwrap(), b"{}");
        assert_eq!(to_canonical_json(&json!([])).unwrap(), b"[]");
    }

    #[test]
    fn string_escaping() {
        let v = json!({"k": "line\nnewline\ttab"});
        let bytes = to_canonical_json(&v).unwrap();
        assert_eq!(bytes, br#"{"k":"line\nnewline\ttab"}"#);
    }

    #[test]
    fn deterministic_hash_independent_of_key_order() {
        let v1 = json!({"b": 2, "a": 1});
        let v2 = json!({"a": 1, "b": 2});
        let h1 = content_hash_of(&v1).unwrap();
        let h2 = content_hash_of(&v2).unwrap();
        assert_eq!(h1, h2);
        assert!(h1.as_str().starts_with("sha256:"));
    }

    #[test]
    fn struct_canonical() {
        #[derive(Serialize)]
        struct Example {
            zebra: u32,
            alpha: String,
        }
        let e = Example {
            zebra: 42,
            alpha: "test".into(),
        };
        let bytes = to_canonical_json(&e).unwrap();
        assert_eq!(bytes, br#"{"alpha":"test","zebra":42}"#);
    }

    #[test]
    fn integer_zero_canonical() {
        let v = json!({"x": 0});
        let bytes = to_canonical_json(&v).unwrap();
        assert_eq!(bytes, br#"{"x":0}"#);
    }
}
