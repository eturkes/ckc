//! ClinicalIR JSON-Schema export (SPEC §9 `schemas/`).
//!
//! [`clinical_ir_schema`] emits the `schemas/clinical_ir.schema.json` bytes
//! (committed and hashed by `schemas-export.1b`): a JSON-Schema (draft 2020-12,
//! an engine-agnostic standard) that mirrors the §4.3 ClinicalIR canonical
//! encoding (`ckc-core` `ir.rs`) member for member, so the M2 local-model
//! runtime can constrain `route.single_ir` output to the shape and vocabulary
//! the deterministic tail reads back. The mirror:
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
//! - interval bounds are §4.3 string-quoted `i64`s → a `string` with the
//!   canonical-decimal `pattern` (never a bare JSON number); the pattern fixes
//!   the lexical form, the `i64` magnitude bound re-imposed downstream by the
//!   deterministic reader (`read_i64`), like the set sort and key derivation;
//! - the derived [`Action`](ckc_core::Action) `key` is carried as a plain id
//!   (the `kind:target` derivation is re-checked by bundle validation, not the
//!   schema).
//!
//! Controlled-vocabulary id fields take lexicon-derived `enum`s from the loaded
//! [`Lexicon`] (`corpus/lexicon/ja_core.yaml`): `system`, the binding `code` and
//! its competing `alternatives`, the action `kind`/`target`, the context-atom
//! concept value, and the interval `var` — `code` and `alternatives` are one
//! concept vocabulary, which `IrBundle::validate` checks alike.
//! Generated ids (`*_id`, the derived `key`) and grounded reference ids
//! (`region_ids`/`source_segment_ids`) stay free `Id` strings — constrained by
//! the id grammar and downstream grounding, not a vocabulary.
//!
//! The bytes are deterministic (sorted-key, compact) — built on `ckc-core`'s
//! canonical [`ObjectEmitter`]/[`emit_string`], so no runtime JSON dependency —
//! and `hash_bytes` over them is the `schema_hash` that `schemas-export.1b`
//! pins for the manifests and the registry `SchemaEntry`.

use ckc_core::{
    BindingStatus, CanonError, Certainty, Direction, ObjectEmitter, Strength, emit_string,
};

use crate::normalize::Lexicon;

/// Draft 2020-12 meta-schema URI; validators auto-detect the dialect from it.
const DRAFT: &str = "https://json-schema.org/draft/2020-12/schema";
/// `Id` grammar (`ckc-core` `id.rs`): a lowercase-led identifier-ascii token.
const ID_PATTERN: &str = "^[a-z][a-z0-9_.:-]*$";
/// §4.3 string-quoted integer lexical form: canonical decimal, no leading zero,
/// no `-0`. Matches the `i64` bound spelling; the `i64` magnitude bound itself
/// (`read_i64`) is re-imposed downstream, not by this pattern.
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
                    ("alternatives", array_of(reference("ConceptCode"), true)),
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
    use ckc_core::{
        Action, ClinicalIr, ClinicalStatement, ContextAtom, ExceptionClause, Id, QuantityInterval,
        TerminologyBinding, canonical_payload_bytes, hash_bytes,
    };
    use jsonschema::Validator;
    use serde_json::Value;

    /// Pinned `schema_hash` = `hash_bytes` over the committed canonical bytes;
    /// update only on an intended schema change (regenerate with
    /// `CKC_BLESS=clinical_ir_schema`).
    const SCHEMA_HASH: &str =
        "sha256:0111668b2445286d22b069a8e51f6c517d4062b3b345f4d364f25dfb7970eaa0";

    /// The committed export: the bless test writes it, the drift guard reads it.
    const SCHEMA_PATH: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../schemas/clinical_ir.schema.json"
    );

    fn committed_lexicon() -> Lexicon {
        let bytes = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/lexicon/ja_core.yaml"
        ))
        .unwrap();
        load_lexicon(&bytes).unwrap()
    }

    fn id(value: &str) -> Id {
        Id::new(value).unwrap()
    }

    /// A valid ClinicalIR over committed-lexicon vocabulary, exercising every
    /// schema feature: both enums and free ids, all four fieldless enums, set
    /// and ordered arrays, present/absent optional `certainty`, and both
    /// concept and interval atom branches.
    fn valid_ir(certainty: Option<Certainty>) -> ClinicalIr {
        ClinicalIr {
            bindings: vec![TerminologyBinding {
                binding_id: id("b.1"),
                system: id("ckc.lex"),
                code: id("cond.sepsis"),
                status: BindingStatus::Exact,
                alternatives: vec![],
                region_ids: vec![id("r.1")],
            }],
            statements: vec![ClinicalStatement {
                statement_id: id("s.1"),
                population: vec![ContextAtom::Concept(id("pop.adult"))],
                condition: vec![ContextAtom::Concept(id("cond.sepsis"))],
                action: Action::new(id("act.administer"), id("drug.abx_a")),
                modality: Direction::For,
                strength: Strength::Strong,
                certainty,
                exceptions: vec![ExceptionClause {
                    exception_id: id("e.1"),
                    atoms: vec![ContextAtom::Interval(QuantityInterval {
                        var: id("q.age_years"),
                        ge: Some(18),
                        gt: None,
                        le: None,
                        lt: None,
                    })],
                    region_ids: vec![id("r.2")],
                }],
                source_segment_ids: vec![id("seg.1")],
            }],
        }
    }

    /// The compiled validator over the emitted schema for the committed lexicon.
    fn validator() -> Validator {
        let schema: Value = serde_json::from_slice(&clinical_ir_schema(&committed_lexicon()))
            .expect("emitted schema is valid JSON");
        jsonschema::validator_for(&schema).expect("emitted schema compiles")
    }

    /// A ClinicalIR's canonical bytes parsed back as a JSON instance to validate.
    fn instance(ir: &ClinicalIr) -> Value {
        serde_json::from_slice(&canonical_payload_bytes(ir).unwrap())
            .expect("canonical ClinicalIR is valid JSON")
    }

    /// The emitted schema parsed as a generic JSON value — the structural oracle
    /// for the parse-only tests below (the jsonschema validation oracle and the
    /// committed-file + hash pins are the `*schema*` tests that follow them).
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

        // The committed lexicon yields a non-empty vocab for every controlled
        // field, so no `$def` is the unsatisfiable empty `enum`.
        for def in [
            "ConceptCode",
            "ActionKind",
            "TerminologySystem",
            "IntervalVar",
        ] {
            assert!(
                !enum_of(def).is_empty(),
                "{def} enum is non-empty for the committed lexicon"
            );
        }
    }

    // Each controlled-vocab consuming field references the right `$def`: concept
    // codes (binding `code` + `alternatives`, action `target`, atom concept
    // value) → ConceptCode; action `kind` → ActionKind; binding `system` →
    // TerminologySystem; the derived `key` stays a free Id. Guards the
    // `code`/`alternatives` vocabulary parity that bundle validation enforces.
    #[test]
    fn consuming_fields_bind_expected_vocab() {
        let schema = parse();
        let defs = &schema["$defs"];
        let ref_of = |v: &Value| v["$ref"].as_str().map(str::to_owned);

        assert_eq!(
            ref_of(&defs["Action"]["properties"]["kind"]).as_deref(),
            Some("#/$defs/ActionKind")
        );
        assert_eq!(
            ref_of(&defs["Action"]["properties"]["target"]).as_deref(),
            Some("#/$defs/ConceptCode")
        );
        assert_eq!(
            ref_of(&defs["Action"]["properties"]["key"]).as_deref(),
            Some("#/$defs/Id")
        );
        assert_eq!(
            ref_of(&defs["TerminologyBinding"]["properties"]["code"]).as_deref(),
            Some("#/$defs/ConceptCode")
        );
        assert_eq!(
            ref_of(&defs["TerminologyBinding"]["properties"]["alternatives"]["items"]).as_deref(),
            Some("#/$defs/ConceptCode")
        );
        assert_eq!(
            ref_of(&defs["TerminologyBinding"]["properties"]["system"]).as_deref(),
            Some("#/$defs/TerminologySystem")
        );
        assert_eq!(
            ref_of(&defs["TerminologyBinding"]["properties"]["status"]).as_deref(),
            Some("#/$defs/BindingStatus")
        );

        let branches = defs["ContextAtom"]["oneOf"]
            .as_array()
            .expect("ContextAtom oneOf is an array");
        let value_ref = |tag: &str| {
            let branch = branches
                .iter()
                .find(|b| b["properties"]["tag"]["const"].as_str() == Some(tag))
                .expect("branch for tag");
            ref_of(&branch["properties"]["value"])
        };
        assert_eq!(value_ref("concept").as_deref(), Some("#/$defs/ConceptCode"));
        assert_eq!(
            value_ref("concept_negated").as_deref(),
            Some("#/$defs/ConceptCode")
        );
        assert_eq!(
            value_ref("interval").as_deref(),
            Some("#/$defs/QuantityInterval")
        );
    }

    // The committed schema file is exactly the emitter output.
    // `CKC_BLESS=clinical_ir_schema` regenerates it (the exact token stops an
    // ambient `CKC_BLESS` silently re-blessing real drift); the bare run is the
    // drift guard.
    #[test]
    fn committed_schema_matches_emitter() {
        let want = clinical_ir_schema(&committed_lexicon());
        if std::env::var("CKC_BLESS").as_deref() == Ok("clinical_ir_schema") {
            std::fs::create_dir_all(concat!(env!("CARGO_MANIFEST_DIR"), "/../../schemas")).unwrap();
            std::fs::write(SCHEMA_PATH, &want).unwrap();
        }
        let got = std::fs::read(SCHEMA_PATH)
            .expect("committed schema present (CKC_BLESS=clinical_ir_schema to regenerate)");
        assert_eq!(
            got, want,
            "committed schema drifted (CKC_BLESS=clinical_ir_schema to regenerate)"
        );
    }

    // The `schema_hash` is stable; a deliberate schema change re-pins it
    // alongside the committed file.
    #[test]
    fn schema_hash_is_pinned() {
        let bytes = clinical_ir_schema(&committed_lexicon());
        assert_eq!(hash_bytes(&bytes).as_str(), SCHEMA_HASH);
    }

    // The schema accepts a canonical ClinicalIR, with optional certainty both
    // present and omitted.
    #[test]
    fn schema_accepts_canonical_clinical_ir() {
        let validator = validator();
        assert!(validator.is_valid(&instance(&valid_ir(Some(Certainty::High)))));
        assert!(validator.is_valid(&instance(&valid_ir(None))));
    }

    // The schema rejects each malformed shape: a dropped required member, an
    // off-lexicon concept code (in `code` and in `alternatives`, the parity
    // bundle validation enforces), an off-lexicon action kind, a bare-number
    // interval bound, an unknown member, and a duplicated §4.3 set element. An
    // out-of-`i64` magnitude bound is deliberately absent — `INT_PATTERN` is
    // i64-lexical not i64-bounded, and `read_i64` is that downstream backstop.
    #[test]
    fn schema_rejects_malformed() {
        let validator = validator();
        let good = instance(&valid_ir(Some(Certainty::High)));
        assert!(validator.is_valid(&good), "baseline must validate");

        // a required member dropped
        let mut missing_action = good.clone();
        missing_action["statements"][0]
            .as_object_mut()
            .unwrap()
            .remove("action");
        assert!(
            !validator.is_valid(&missing_action),
            "missing required action"
        );

        // a controlled-vocabulary code outside the lexicon enum (still a valid id)
        let mut bad_code = good.clone();
        bad_code["bindings"][0]["code"] = Value::from("cond.not_in_lexicon");
        assert!(!validator.is_valid(&bad_code), "non-lexicon concept code");

        // a competing alternative outside the concept vocabulary (guards the
        // codex-fixed `alternatives` → ConceptCode parity with `code`)
        let mut bad_alternative = good.clone();
        bad_alternative["bindings"][0]["alternatives"] = Value::from(vec!["cond.not_in_lexicon"]);
        assert!(
            !validator.is_valid(&bad_alternative),
            "non-lexicon alternative"
        );

        // an action kind outside the lexicon enum
        let mut bad_kind = good.clone();
        bad_kind["statements"][0]["action"]["kind"] = Value::from("act.bogus");
        assert!(!validator.is_valid(&bad_kind), "non-lexicon action kind");

        // an interval bound as a bare number rather than a quoted decimal
        let mut numeric_bound = good.clone();
        numeric_bound["statements"][0]["exceptions"][0]["atoms"][0]["value"]["ge"] =
            Value::from(18);
        assert!(
            !validator.is_valid(&numeric_bound),
            "bare-number interval bound"
        );

        // a string interval bound that is not a canonical integer — proves the
        // INT_PATTERN `pattern` is enforced, not just the `string` type
        let mut noncanonical_bound = good.clone();
        noncanonical_bound["statements"][0]["exceptions"][0]["atoms"][0]["value"]["ge"] =
            Value::from("1.5");
        assert!(
            !validator.is_valid(&noncanonical_bound),
            "non-canonical interval bound"
        );

        // an unknown member under additionalProperties:false
        let mut extra = good.clone();
        extra["bindings"][0]
            .as_object_mut()
            .unwrap()
            .insert("extra".to_owned(), Value::from("x"));
        assert!(!validator.is_valid(&extra), "unknown member rejected");

        // a §4.3 set with a duplicate element
        let mut dup = good.clone();
        dup["statements"][0]["source_segment_ids"] = Value::from(vec!["seg.1", "seg.1"]);
        assert!(!validator.is_valid(&dup), "duplicate set element");
    }
}
