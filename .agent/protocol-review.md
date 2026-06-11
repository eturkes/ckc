# Review session protocol (routed from /session-prompt)

Every item of the current milestone is checked. Review sessions run
single-context at 1M: hold the milestone's range, its artifacts, and the
implicated spec sections together in your own context and analyze them
coherently yourself — the 1M window exists precisely so the codebase stays
whole instead of fragmenting across subagents.

Recover the range from the header: `git log --oneline <plan-hash>..HEAD`, and
`git show` the unit commits (the items' recorded hashes) to bound the scope;
read the touched artifacts in full.

Beyond trivial bug fixes, the review is a holistic analysis of codebase
cohesion and overall project direction, scrutinized along: bugs and incorrect
logic, specification non-conformance, CLAUDE.md/memory non-conformance,
inconsistencies, token-inefficiency, obsolescence. Specification improvements
are in scope: when the analysis exposes a better contract or design, edit
SPEC.md in the same session (contract-affecting amendments reach the user
first, per SPEC §1).

Close the milestone in one commit, scoped `review-m<n>:` — fill the last
item's pending `_`, carry the corrections (or state the review was clean), and
mark the header `— review _`; reviews record no usage. The next roadmap-mode
session — the plan session for the next milestone — fills the review hash.
