#!/bin/sh
# ape-build reproducible gate (SPEC §10.6; M3.ape-build). Builds the vendored APE fork and proves it
# runs under SWI-Prolog, and that the CKC-rewired AceRules engine loads clean + runs courteous
# semantics through APE. Repo-relative (run from any cwd); fail-closed: exit 0 iff every check passes.
#
#   sh clinicalcnl/clinical/ape_build_smoke.sh
#
# Rebuilds ape.exe (+ prolog/parser/*.plp) on each run; both are gitignored, so the tree stays clean.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)   # clinicalcnl/
cd "$ROOT"

fail() { echo "ape-build smoke FAIL: $1" >&2; exit 1; }

echo "[1/5] SWI-Prolog 9.x present (functional env gate)"
swipl --version | grep -q 'SWI-Prolog version 9' || fail "SWI-Prolog 9.x not found"

echo "[2/5] make install -> ape.exe (full vendored Clex baked in)"
make install >/dev/null 2>&1 || fail "make install"
test -x ape.exe || fail "ape.exe not built"

echo "[3/5] get_ape_results loads; acetext_to_drs -> well-formed should()-DRS (clinical frame)"
swipl -q -f get_ape_results.pl -g "
  ( ace_to_drs:acetext_to_drs('It is recommended that a patient takes a drug.', _S, _T, Drs, _M),
    Drs = drs(Refs, Conds), is_list(Refs), is_list(Conds), Conds = [should(drs(_,_))|_]
  -> halt(0) ; halt(1) )" -t 'halt(1)' || fail "get_ape_results / acetext_to_drs shape"

echo "[4/5] ape.exe -solo drs -> should()-DRS (full-vocab clinical sentence)"
./ape.exe -text 'It is recommended that a patient takes a drug.' -solo drs \
  | grep -q '^drs(\[\],\[should(' || fail "ape.exe -solo drs"

echo "[5/5] AceRules engine loads clean; court nixon -> courteous override"
LOADERR=$(swipl -q -g "consult('vendor/acerules/engine/acerules_processor.pl'), halt" -t 'halt(1)' 2>&1 1>/dev/null || true)
if printf '%s' "$LOADERR" | grep -qE 'ERROR:|Warning:'; then fail "AceRules engine load not clean: $LOADERR"; fi
swipl -q -g "
  use_module(library(readutil)),
  consult('vendor/acerules/engine/acerules_processor.pl'),
  read_file_to_codes('vendor/acerules/engine/testcases/court/input/nixon', Codes, []),
  ( acerules_processor:generate_output(Codes, court, [guess=on], _R, _A, _T, [AT|_]),
    sub_atom(AT, _, _, _, 'It is false that Nixon is a')
  -> halt(0) ; halt(1) )" -t 'halt(1)' || fail "AceRules court nixon override"

echo "ape-build smoke: PASS"
