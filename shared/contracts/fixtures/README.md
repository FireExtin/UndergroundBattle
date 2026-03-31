# shared/contracts/fixtures

Purpose: authoritative CardLogic DSL fixtures that gate cards into the main pool.

Each fixture is self-contained and must include:

- `cardId`
- `schemaVersion`
- `card`
- `input`
- `expectations`

TypeScript tooling and the Go authority both consume these exact JSON files.
