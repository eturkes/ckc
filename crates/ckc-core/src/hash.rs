//! Wrapper content and policy hashes (SPEC §4.3, §4.4).
//!
//! SPEC §4.3 fixes `content_hash = sha256(canonical_payload_bytes(payload))`.
//! [`content_hash`] is that single function, wrapping the digest as the [`Hash`]
//! value type; [`hash_bytes`] exposes the underlying sha256-over-raw-bytes step
//! for the §4.4 `_hash` fields that declare raw-byte hashing. The descriptor
//! [`CanonicalizationPolicy`] names the canonical-bytes policy, and
//! [`canonicalization_policy_hash`] is its content hash — the value an artifact
//! wrapper records to pin the policy version that sealed it.

use sha2::{Digest, Sha256};

use crate::canon::{CanonError, Canonical, ObjectEmitter, canonical_payload_bytes, emit_string};
use crate::id::Hash;

/// Lowercase hex digits, mirroring the canonical writer's escape table.
const HEX: &[u8; 16] = b"0123456789abcdef";

/// SPEC §4.3 `content_hash = sha256(canonical_payload_bytes(payload))`, wrapped as
/// the [`struct@Hash`] value type. The one mechanism for an artifact's content hash;
/// fails only when `value` cannot be canonicalized.
pub fn content_hash<T: Canonical>(value: &T) -> Result<Hash, CanonError> {
    Ok(hash_bytes(&canonical_payload_bytes(value)?))
}

/// sha256 of raw bytes as a [`struct@Hash`] (`"sha256:"` + 64 lowercase hex digits). The
/// primitive behind [`content_hash`], and the entry point for §4.4 `_hash` fields
/// that declare raw-byte hashing rather than canonical-payload hashing.
pub fn hash_bytes(bytes: &[u8]) -> Hash {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity("sha256:".len() + 2 * digest.len());
    hex.push_str("sha256:");
    for &byte in digest.iter() {
        hex.push(char::from(HEX[usize::from(byte >> 4)]));
        hex.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Hash::new(hex).expect("sha256 digest is 64 lowercase hex digits")
}

/// SPEC §4.4 `canonicalization_policy_hash`: the [`content_hash`] of the
/// [`CanonicalizationPolicy`] descriptor. It changes iff the canonical-bytes
/// policy version changes, so a stored value pins the policy under which an
/// artifact was sealed.
pub fn canonicalization_policy_hash() -> Hash {
    content_hash(&CanonicalizationPolicy::M1).expect("policy descriptor canonicalizes")
}

/// Stable identity of the SPEC §4.3 canonical-JSON byte policy: an `id` naming the
/// policy family and a `version` locked per milestone. It is itself a
/// [`Canonical`] value, so [`canonicalization_policy_hash`] is an ordinary
/// [`content_hash`] over it; bumping `version` when §4.3 changes re-keys every
/// downstream policy hash.
pub struct CanonicalizationPolicy {
    id: &'static str,
    version: &'static str,
}

impl CanonicalizationPolicy {
    /// The M1 canonical-JSON byte policy (SPEC §4.3).
    pub const M1: CanonicalizationPolicy = CanonicalizationPolicy {
        id: "ckc.canonical_json",
        version: "ckc.m1",
    };
}

/// Emits `{"id":...,"version":...}` through [`ObjectEmitter`], so the descriptor
/// shares the writer's field sorting and the policy hash is self-consistent with
/// the canonical bytes it certifies.
impl Canonical for CanonicalizationPolicy {
    fn emit_canonical(&self, out: &mut Vec<u8>) -> Result<(), CanonError> {
        let mut obj = ObjectEmitter::new();
        obj.member("id", |b| {
            emit_string(b, self.id);
            Ok(())
        })?;
        obj.member("version", |b| {
            emit_string(b, self.version);
            Ok(())
        })?;
        obj.finish(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::Id;

    // NIST SHA-256 vector for "abc": pins the digest and the lowercase-hex form.
    #[test]
    fn hash_bytes_matches_nist_abc_vector() {
        assert_eq!(
            hash_bytes(b"abc").as_str(),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    // NIST SHA-256 vector for the empty input: the raw-byte edge case.
    #[test]
    fn hash_bytes_matches_nist_empty_vector() {
        assert_eq!(
            hash_bytes(b"").as_str(),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn content_hash_is_sha256_of_canonical_bytes() {
        let id = Id::new("schema.ir_bundle").unwrap();
        let bytes = canonical_payload_bytes(&id).unwrap();
        assert_eq!(content_hash(&id).unwrap(), hash_bytes(&bytes));
    }

    #[test]
    fn content_hash_is_deterministic_and_value_sensitive() {
        let a = Id::new("a").unwrap();
        let b = Id::new("b").unwrap();
        assert_eq!(content_hash(&a).unwrap(), content_hash(&a).unwrap());
        assert_ne!(content_hash(&a).unwrap(), content_hash(&b).unwrap());
    }

    // Pins the descriptor's byte-sorted two-field shape so the policy hash is
    // stable and the bytes it certifies stay reviewable.
    #[test]
    fn policy_descriptor_canonical_bytes() {
        assert_eq!(
            canonical_payload_bytes(&CanonicalizationPolicy::M1).unwrap(),
            br#"{"id":"ckc.canonical_json","version":"ckc.m1"}"#
        );
    }

    #[test]
    fn canonicalization_policy_hash_is_stable_content_hash() {
        let h = canonicalization_policy_hash();
        assert_eq!(h, canonicalization_policy_hash());
        assert_eq!(h, content_hash(&CanonicalizationPolicy::M1).unwrap());
    }
}
