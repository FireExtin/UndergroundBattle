# Cloud playtest tooling regressions

Run this directory's default automated regression discovery:

```sh
python -m unittest discover -s tools/cloud-playtest
```

The cancellation tests use local async doubles. They make no network calls,
start no browsers, and issue no game actions. They require the runner's existing
Python/Playwright imports. They verify that cancellation remains cancellation,
records a failed checkpoint, and closes owned contexts, including when the
final page is already unavailable.

These are implementation/tooling regressions. They do not establish independent
gameplay acceptance. The 2026-10-03 task assigns public gameplay and acceptance
decisions to the parent dot; do not resume its stopped autonomous QA run without
an explicit new instruction.
