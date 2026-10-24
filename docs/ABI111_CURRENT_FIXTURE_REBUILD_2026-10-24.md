# ABI111 current-kernel test inputs

This is author verification for the first imported twenty-card batch. It is not
an independent review, a browser playtest, or proof that all admitted cards have
been accepted on the public site.

The published runtime source is `28304934554e9a94a5890abf4281785f9c3ec893`:
`rust-v0.2.111-import20-rulings-complete`, pool
`limited-v2.102-import20-rulings-complete`, 272 total definitions, 233 supported
and 39 gated. Test-only checkpoint `ffd23b6df59edb7a01c0a82889f480c8d988f292`
corrects obsolete catalog assertions and introduces the current classic input
assembler. It does not change the runtime or any card rule.

The production WASM SHA-256 is
`4f205f04db4d8cb4293cb085afd37bd3e32ca3103ad3af7fbb4d8cbcc3e430aa`.

## Why the inputs were rebuilt

The first concentrated Native run passed 2495 tests and failed three assertions
that still described an earlier card-admission set. The source catalog was
checked before updating them: WM037's complete catalog helper now counts 233;
CQ28 is a newly admitted space spell; the old blanket “everything after 213 is
gated” assertion is replaced by equality with the exact twenty admitted IDs.
Those three directed reruns passed. No production code or timeout was relaxed.

The first concentrated Web and Sites runs also retained positive test inputs
bound to older source/WASM identities. A fixture-source mismatch is a failed
verification, not permission to replace its hash or modify its opaque Room.

## Fresh-input procedure

The selected existing Native tests were executed from current source with new
export destinations. The first export manifest records 49 groups, 846 selected
test executions, and zero failed or empty groups. These overlap the Native
suite; they are not 846 additional unique tests. The raw exports and original
logs are retained under the task's `serial-intake-20261024/abi111-fresh-native`
evidence directory. Existing historical fixtures remain unchanged.

The following tools select and package the newly generated evidence:

* `tools/fixture-tools/current-classic.mjs` selects the classic Native traces;
  `current93-classic-inputs.json` documents the exporter environment variables.
  Its historical filename is retained as a recipe, not a compatible kernel.
* The existing `jc061-current.py`, `jc087-current.py`, `jc094-current.py`,
  `wm056-current.py`, `wm023-current.py`, `xq25-current.py`, `wm038-current.py`,
  `jz35-current.py`, `jc052-current.py`, `current-four-faction-choices.py`,
  `current-win-flow.py`, and `current-bq030.mjs` consume the new raw exports.
* `seven_card_ui_fixtures` creates current `Game`/`RoomEnvelope` layouts from
  scratch. `current_jc052_worker_fixtures` accepts the freshly exported current
  JC052 input and checks unchanged opaque Room bytes and exact Native results.
* `tools/fixture-tools/current-ui-selectors.mjs` checks current catalog/source
  identities and packages those new selector outputs in `tests/current111`.
* The card-specific oracle packers compare every selected Native command/view
  to the actual current WASM before writing the new `Native111.fixture.json.gz`
  files. Historical raw-format names are schema labels; they do not load an old
  kernel. Complete catalog comparisons and source-binding checks are retained.
* Fresh third-batch and JZ56 platform witnesses use the same current-source
  envelope. `current-sentinel.mjs` compares the new Native command/view and
  duplicate receipts before packaging the Sentinel witness.
* `build-current-local-worker.mjs` creates an isolated, temporary clock wrapper
  around the actual current production module. The Sentinel test now builds
  this default harness itself; it needs no previously exported environment or
  isolated historical candidate. Its temporary output is never deployed.

The historical `current_ui_worker_fixtures` layout adapter was inspected and was
**not run**. No historical state was relabeled, no live Room was imported, and
no opaque seed/state/journal was round-tripped to fabricate parity.

Generated uncompressed selector intermediates and all original failure logs are
retained as task evidence. Only necessary compressed regression inputs and
their consumers belong in the repository checkpoint. Build outputs, local D1
databases, dependencies, credentials and browser seat data are excluded.

## Verification boundaries

UI component tests and local Worker/D1 tests are separate from real browser
acceptance. A fresh normal HTTP match already reached a terminal state through
1216 legal commands at turn 21 under the actual ABI111 engine; this is a script
test, not a browser match and not coverage of every new card. Public browser
evidence and later concentrated rerun totals are recorded separately with
their exact candidate identities. The initial red results remain part of the
evidence; corrected directed passes do not rewrite them.

The completed reruns are recorded in
`BATCH20_ABI111_POSTPUBLICATION_VERIFICATION_2026-10-24.md`. In particular,
JZ56 remains `supported:false`: Native implementation and public admission are
different boundaries. A mistaken test-only admission assertion was corrected
back to that existing boundary; no production flag changed. Historical test
filenames/titles such as `Native93` remain in some logs, while their loaded
fixture, whole-catalog comparisons and executable WASM bind the current ABI111.
