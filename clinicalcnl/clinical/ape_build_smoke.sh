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

echo "[2/5] make install -> ape.exe (full vendored Clex baked in, 0 err/warn)"
BUILDLOG=$(make install 2>&1) || { printf '%s\n' "$BUILDLOG" >&2; fail "make install (nonzero exit)"; }
if printf '%s' "$BUILDLOG" | grep -qE 'ERROR:|Warning:'; then printf '%s\n' "$BUILDLOG" >&2; fail "make install emitted errors/warnings"; fi
test -x ape.exe || fail "ape.exe not built"

echo "[3/5] acetext_to_drs -> clean should()-DRS carrying full-vocab clinical terms (patient/drug/take)"
swipl -q -f get_ape_results.pl -g "
  ( ace_to_drs:acetext_to_drs('It is recommended that a patient takes a drug.', _S, _T, Drs, Msgs),
    Msgs == [],
    Drs = drs([], [should(drs(Refs, Conds))]), is_list(Refs), is_list(Conds),
    memberchk(object(_,patient,_,_,_,_)-_, Conds),
    memberchk(object(_,drug,_,_,_,_)-_, Conds),
    memberchk(predicate(_,take,_,_)-_, Conds)
  -> halt(0) ; halt(1) )" -t 'halt(1)' || fail "acetext_to_drs: clean full-vocab should()-DRS"

echo "[4/5] ape.exe -solo drs -> should()-DRS (baked binary, full-vocab clinical sentence)"
DRSOUT=$(./ape.exe -text 'It is recommended that a patient takes a drug.' -solo drs) \
  || fail "ape.exe -solo drs (nonzero exit)"
printf '%s' "$DRSOUT" | grep -q '^drs(\[\],\[should(drs(' || fail "ape.exe -solo drs: not a should()-DRS"

echo "[5/5] AceRules engine loads clean; court nixon -> courteous override"
if ! LOADERR=$(swipl -q -g "consult('vendor/acerules/engine/acerules_processor.pl'), halt" -t 'halt(1)' 2>&1 1>/dev/null); then
  fail "AceRules engine load exited nonzero: $LOADERR"
fi
if printf '%s' "$LOADERR" | grep -qE 'ERROR:|Warning:'; then fail "AceRules engine load not clean: $LOADERR"; fi
swipl -q -g "
  use_module(library(readutil)),
  consult('vendor/acerules/engine/acerules_processor.pl'),
  read_file_to_codes('vendor/acerules/engine/testcases/court/input/nixon', Codes, []),
  ( acerules_processor:generate_output(Codes, court, [guess=on], _R, _A, _T, [AT|_]),
    sub_atom(AT, _, _, _, 'It is false that Nixon is a pacifist.')
  -> halt(0) ; halt(1) )" -t 'halt(1)' 2>/dev/null || fail "AceRules court nixon override"

echo "ape-build smoke: PASS"
