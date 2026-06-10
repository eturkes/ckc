# CKC roadmap

Flat ordered checklist consumed by the /session-prompt command. SPEC.md is the
design authority; its §2 is the build plan. Completed lines gain `[x]` plus a
trailing `NN% NNNK/200K` annotation from `.agent/compaction.sh`. An empty tail
means: author the next units from the current SPEC milestone.

- [x] boilerplate: minimal repository skeleton per SPEC §3 — .gitignore already sufficient; root
  Cargo workspace (resolver 3, edition 2024) seeded with an empty crates/ckc-core member
  (user-approved deviation: every cargo command rejects a memberless virtual workspace, so the
  sole stub member replaced "no member crates yet"); corpus/{fixtures,lexicon,gold}/ and
  registry/ directories. Reading: SPEC §3. Gate: `cargo test --workspace` runs clean and the
  tree commits clean. 25% 51K/200K
- [ ] plan-v1: author the V1 build units into this roadmap from SPEC §8.7 (dependency order;
  memory sizing anchors), run as a planning
  workflow per this command. Reading: SPEC §2, §4, §8. Gate: forward units authored below this
  line.
