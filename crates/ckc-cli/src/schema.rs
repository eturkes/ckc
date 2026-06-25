//! ClinicalIR JSON-Schema export (SPEC §9 `schemas/`).
//!
//! [`clinical_ir_schema`] emits the committed `schemas/clinical_ir.schema.json`:
//! a JSON-Schema (draft 2020-12, an engine-agnostic standard) that mirrors the
//! §4.3 ClinicalIR canonical encoding (`ckc-core` `ir.rs`) member for member, so
//! the M2 local-model runtime can constrain `route.single_ir` output to exactly
//! the shape the deterministic tail reads back. The mirror:
//!
//! - sorted-name object members (`additionalProperties:false` mirrors the
//!   canonical reader rejecting unknown fields); optional members
//!   (`certainty`, the interval bounds) are present in `properties` but absent
//!   from `required`, the schema image of canonical optional-omit;
//! - §4.3 sets render as `array` + `uniqueItems` (`population`/`condition`/
//!   `atoms`, the id sets), ordered components as bare `array` (`bindings`/
//!   `statements`/`exceptions`); the canonical byte-sort *within* a set is not
//!   expressible in JSON-Schema and is re-imposed by the deterministic
//!   re-emission downstream, so the schema constrains structure and vocabulary,
//!   not element order;
//! - [`ContextAtom`](ckc_core::ContextAtom) is the §4.3 `{tag,value}` tagged
//!   union → a `oneOf` of `{tag:{const},value}` branches;
//! - interval bounds are §4.3 string-quoted integers → a `string` with the
//!   canonical-decimal pattern, never a bare JSON number;
//! - the derived [`Action`](ckc_core::Action) `key` is carried as a plain id
//!   (the `kind:target` derivation is re-checked by bundle validation, not the
//!   schema).
//!
//! Controlled-vocabulary id fields take lexicon-derived `enum`s from the loaded
//! [`Lexicon`] (`corpus/lexicon/ja_core.yaml`): `system`, `code`, the action
//! `kind`/`target`, the context-atom concept value, and the interval `var`.
//! Generated ids (`*_id`, the derived `key`) and grounded reference ids
//! (`alternatives`/`region_ids`/`source_segment_ids`) stay free `Id` strings —
//! constrained by the id grammar and downstream grounding, not a vocabulary.
//!
//! The bytes are deterministic (sorted-key, compact) — built on `ckc-core`'s
//! canonical [`ObjectEmitter`]/[`emit_string`], so no runtime JSON dependency —
//! and `hash_bytes` over them is the `schema_hash` the manifests and the
//! registry `SchemaEntry` record.

use ckc_core::{
    BindingStatus, CanonError, Certainty, Direction, ObjectEmitter, Strength, emit_string,
};

use crate::normalize::Lexicon;

/// Draft 2020-12 meta-schema URI; validators auto-detect the dialect from it.
const DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";
/// `Id` grammar (`ckc-core` `id.rs`): a lowercase-led identifier-ascii token.
const ID_PATTERN: &str = "^[a-z][a-z0-9_.:-]*$";
/// §4.3 string-quoted integer: canonical decimal, no leading zero, no `-0`.
const INT_PATTERN: &str = "^(0|-?[1-9][0-9]*)$";

/// Emit the ClinicalIR JSON-Schema as deterministic canonical bytes, injecting
/// `enum`s from `lexicon` for the controlled-vocabulary id fields.
pub fn clinical_ir_schema(lexicon: &Lexicon) -> Vec<u8> {
    let concept_codes = lexicon
        .concepts
        .iter()
        .map(|c| c.concept_id.as_str().to_owned())
        .collect::<Vec<_>>();
    let action_kinds = lexicon
        .actions
        .iter()
        .map(|a| a.action_id.as_str().to_owned())
        .collect::<Vec<_>>();
    let systems = vec![lexicon.system.as_str().to_owned()];
    let interval_vars = lexicon
        .concepts
        .iter()
        .filter_map(|c| c.interval.as_ref().map(|i| i.var.as_str().to_owned()))
        .collect::<Vec<_>>();

    let defs = obj(vec![
        (
            "Action",
            record(
                &["key", "kind", "target"],
                vec![
                    ("key", reference("Id")),
                    ("kind", reference("ActionKind")),
                    ("target", reference("ConceptCode")),
                ],
            ),
        ),
        ("ActionKind", string_enum(action_kinds)),
        (
            "BindingStatus",
            string_enum(spellings(BindingStatus::ALL, BindingStatus::as_str)),
        ),
        (
            "Certainty",
            string_enum(spellings(Certainty::ALL, Certainty::as_str)),
        ),
        (
            "ClinicalStatement",
            record(
                &[
                    "action",
                    "condition",
                    "exceptions",
                    "modality",
                    "population",
                    "source_segment_ids",
                    "statement_id",
                    "strength",
                ],
                vec![
                    ("action", reference("Action")),
                    ("certainty", reference("Certainty")),
                    ("condition", array_of(reference("ContextAtom"), true)),
                    ("exceptions", array_of(reference("ExceptionClause"), false)),
                    ("modality", reference("Direction")),
                    ("population", array_of(reference("ContextAtom"), true)),
                    ("source_segment_ids", array_of(reference("Id"), true)),
                    ("statement_id", reference("Id")),
                    ("strength", reference("Strength")),
                ],
            ),
        ),
        ("ConceptCode", string_enum(concept_codes)),
        (
            "ContextAtom",
            obj(vec![(
                "oneOf",
                Json::Arr(vec![
                    atom_branch("concept", reference("ConceptCode")),
                    atom_branch("concept_negated", reference("ConceptCode")),
                    atom_branch("interval", reference("QuantityInterval")),
                ]),
            )]),
        ),
        (
            "Direction",
            string_enum(spellings(Direction::ALL, Direction::as_str)),
        ),
        (
            "ExceptionClause",
            record(
                &["atoms", "exception_id", "region_ids"],
                vec![
                    ("atoms", array_of(reference("ContextAtom"), true)),
                    ("exception_id", reference("Id")),
                    ("region_ids", array_of(reference("Id"), true)),
                ],
            ),
        ),
        ("Id", string_pattern(ID_PATTERN)),
        ("IntervalBound", string_pattern(INT_PATTERN)),
        ("IntervalVar", string_enum(interval_vars)),
        (
            "QuantityInterval",
            record(
                &["var"],
                vec![
                    ("ge", reference("IntervalBound")),
                    ("gt", reference("IntervalBound")),
                    ("le", reference("IntervalBound")),
                    ("lt", reference("IntervalBound")),
                    ("var", reference("IntervalVar")),
                ],
            ),
        ),
        (
            "Strength",
            string_enum(spellings(Strength::ALL, Strength::as_str)),
        ),
        (
            "TerminologyBinding",
            record(
                &[
                    "alternatives",
                    "binding_id",
                    "code",
                    "region_ids",
                    "status",
                    "system",
                ],
                vec![
                    ("alternatives", array_of(reference("Id"), true)),
                    ("binding_id", reference("Id")),
                    ("code", reference("ConceptCode")),
                    ("region_ids", array_of(reference("Id"), true)),
                    ("status", reference("BindingStatus")),
                    ("system", reference("TerminologySystem")),
                ],
            ),
        ),
        ("TerminologySystem", string_enum(systems)),
    ]);

    let root = obj(vec![
        ("$defs", defs),
        ("$schema", s(DRAFT)),
        ("additionalProperties", Json::Bool(false)),
        (
            "properties",
            obj(vec![
                ("bindings", array_of(reference("TerminologyBinding"), false)),
                (
                    "statements",
                    array_of(reference("ClinicalStatement"), false),
                ),
            ]),
        ),
        ("required", Json::Arr(vec![s("bindings"), s("statements")])),
        ("title", s("ClinicalIR")),
        ("type", s("object")),
    ]);

    let mut out = Vec::new();
    root.emit(&mut out);
    out
}

/// A minimal JSON value, emitted as deterministic compact bytes: objects sort
/// their members by key (via [`ObjectEmitter`]), arrays keep their given order.
enum Json {
    Str(String),
    Bool(bool),
    Arr(Vec<Json>),
    Obj(Vec<(&'static str, Json)>),
}

impl Json {
    fn emit(&self, out: &mut Vec<u8>) {
        match self {
            Json::Str(value) => emit_string(out, value),
            Json::Bool(value) => out.extend_from_slice(if *value { b"true" } else { b"false" }),
            Json::Arr(items) => {
                out.push(b'[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(b',');
                    }
                    item.emit(out);
                }
                out.push(b']');
            }
            Json::Obj(members) => {
                let mut object = ObjectEmitter::new();
                for &(name, ref value) in members {
                    object
                        .member(name, |b| {
                            value.emit(b);
                            Ok::<(), CanonError>(())
                        })
                        .expect("schema member emits");
                }
                object.finish(out).expect("schema object has unique keys");
            }
        }
    }
}

fn s(value: &str) -> Json {
    Json::Str(value.to_owned())
}

fn obj(members: Vec<(&'static str, Json)>) -> Json {
    Json::Obj(members)
}

/// A `$ref` into `#/$defs/<def>`.
fn reference(def: &str) -> Json {
    obj(vec![("$ref", Json::Str(format!("#/$defs/{def}")))])
}

/// A `string` constrained by `pattern`.
fn string_pattern(pattern: &str) -> Json {
    obj(vec![("pattern", s(pattern)), ("type", s("string"))])
}

/// A `string` constrained to `values` (sorted + de-duplicated for canonical bytes).
fn string_enum(mut values: Vec<String>) -> Json {
    values.sort();
    values.dedup();
    obj(vec![
        (
            "enum",
            Json::Arr(values.into_iter().map(Json::Str).collect()),
        ),
        ("type", s("string")),
    ])
}

/// An `array` of `items`; `unique_items` marks a §4.3 set.
fn array_of(items: Json, unique_items: bool) -> Json {
    let mut members = vec![("items", items), ("type", s("array"))];
    if unique_items {
        members.push(("uniqueItems", Json::Bool(true)));
    }
    obj(members)
}

/// A closed `object` (`additionalProperties:false`) over `properties`, requiring
/// `required` (sorted for canonical bytes; optional members are omitted from it).
fn record(required: &[&'static str], properties: Vec<(&'static str, Json)>) -> Json {
    let mut req = required.to_vec();
    req.sort_unstable();
    obj(vec![
        ("additionalProperties", Json::Bool(false)),
        ("properties", Json::Obj(properties)),
        ("required", Json::Arr(req.into_iter().map(s).collect())),
        ("type", s("object")),
    ])
}

/// One tagged-union branch: `{tag:{const:<tag>}, value:<value>}`, closed.
fn atom_branch(tag: &'static str, value: Json) -> Json {
    obj(vec![
        ("additionalProperties", Json::Bool(false)),
        (
            "properties",
            obj(vec![
                ("tag", obj(vec![("const", s(tag))])),
                ("value", value),
            ]),
        ),
        ("required", Json::Arr(vec![s("tag"), s("value")])),
        ("type", s("object")),
    ])
}

/// The identifier-ascii spellings of a fieldless enum's values.
fn spellings<T: Copy>(all: &[T], as_str: fn(T) -> &'static str) -> Vec<String> {
    all.iter().map(|&v| as_str(v).to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalize::load_lexicon;
    use serde_json::Value;

    fn committed_lexicon() -> Lexicon {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/lexicon/ja_core.yaml"
        ))
        .unwrap();
        load_lexicon(&bytes).unwrap()
    }

    /// The emitted schema parsed as a generic JSON value — the structural oracle
    /// for these parse-only tests. The jsonschema validation oracle (good/
    /// malformed instances) and the committed-file + hash pins land in
    /// schemas-export.1b.
    fn parse() -> Value {
        serde_json::from_slice(&clinical_ir_schema(&committed_lexicon()))
            .expect("emitted schema is valid JSON")
    }

    // The emitted bytes are well-formed JSON carrying the ClinicalIR root shape.
    #[test]
    fn schema_bytes_parse_as_json() {
        let bytes = clinical_ir_schema(&committed_lexicon());
        let value: Value = serde_json::from_slice(&bytes).expect("emitted schema is valid JSON");
        assert_eq!(value["title"], Value::from("ClinicalIR"));
        assert_eq!(value["type"], Value::from("object"));
        assert_eq!(value["$schema"], Value::from(DRAFT));
        assert_eq!(value["additionalProperties"], Value::from(false));
    }

    // The §4.3 ContextAtom tagged union renders as a closed three-branch oneOf
    // whose tag consts are exactly concept / concept_negated / interval, in the
    // authored order (the emitter sorts object members, not array elements).
    #[test]
    fn context_atom_is_three_branch_oneof() {
        let schema = parse();
        let branches = schema["$defs"]["ContextAtom"]["oneOf"]
            .as_array()
            .expect("ContextAtom oneOf is an array");
        let tags = branches
            .iter()
            .map(|b| b["properties"]["tag"]["const"].as_str().expect("tag const"))
            .collect::<Vec<_>>();
        assert_eq!(tags, ["concept", "concept_negated", "interval"]);
        for branch in branches {
            assert_eq!(branch["type"], Value::from("object"));
            assert_eq!(branch["additionalProperties"], Value::from(false));
            assert_eq!(branch["required"], Value::from(vec!["tag", "value"]));
        }
    }

    // QuantityInterval requires only `var`; its four bounds reference
    // IntervalBound, whose pattern is the §4.3 string-quoted integer (a quoted
    // decimal, never a bare JSON number).
    #[test]
    fn quantity_interval_required_var_and_string_int_bounds() {
        let schema = parse();
        let interval = &schema["$defs"]["QuantityInterval"];
        assert_eq!(interval["required"], Value::from(vec!["var"]));
        for bound in ["ge", "gt", "le", "lt"] {
            assert_eq!(
                interval["properties"][bound]["$ref"],
                Value::from("#/$defs/IntervalBound"),
                "{bound} references IntervalBound"
            );
        }
        let bound_def = &schema["$defs"]["IntervalBound"];
        assert_eq!(bound_def["type"], Value::from("string"));
        assert_eq!(bound_def["pattern"], Value::from(INT_PATTERN));
    }

    // The derived Action.key is a required member (a free Id; the kind:target
    // derivation is re-checked by bundle validation, not by the schema).
    #[test]
    fn action_requires_key() {
        let schema = parse();
        let required = schema["$defs"]["Action"]["required"]
            .as_array()
            .expect("Action required is an array");
        assert!(
            required.iter().any(|v| v.as_str() == Some("key")),
            "Action.required must contain key"
        );
        assert_eq!(
            schema["$defs"]["Action"]["properties"]["key"]["$ref"],
            Value::from("#/$defs/Id")
        );
    }

    // The controlled-vocabulary enums are exactly the loaded lexicon vocab
    // (sorted + de-duplicated) for each of concept / action / system / var.
    #[test]
    fn controlled_vocab_enums_track_lexicon() {
        let lexicon = committed_lexicon();
        let schema = parse();

        let enum_of = |def: &str| {
            schema["$defs"][def]["enum"]
                .as_array()
                .expect("enum is an array")
                .iter()
                .map(|v| v.as_str().expect("enum member is a string").to_owned())
                .collect::<Vec<_>>()
        };
        let canon = |mut v: Vec<String>| {
            v.sort();
            v.dedup();
            v
        };

        assert_eq!(
            enum_of("ConceptCode"),
            canon(
                lexicon
                    .concepts
                    .iter()
                    .map(|c| c.concept_id.as_str().to_owned())
                    .collect()
            )
        );
        assert_eq!(
            enum_of("ActionKind"),
            canon(
                lexicon
                    .actions
                    .iter()
                    .map(|a| a.action_id.as_str().to_owned())
                    .collect()
            )
        );
        assert_eq!(
            enum_of("TerminologySystem"),
            vec![lexicon.system.as_str().to_owned()]
        );
        assert_eq!(
            enum_of("IntervalVar"),
            canon(
                lexicon
                    .concepts
                    .iter()
                    .filter_map(|c| c.interval.as_ref().map(|i| i.var.as_str().to_owned()))
                    .collect()
            )
        );
    }
}
