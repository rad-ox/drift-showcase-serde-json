# DRIFT Real-World PR Showcase — `serde_json` (Rust / Cargo)

This fork demonstrates **[DRIFT Free (`v0.10.0`)](https://github.com/rad-ox/drift)** running in live GitHub Actions pull-request checks on **[`serde_json`](https://github.com/serde-rs/json)** (pinned at commit `f51fa15a7015`), upgrading the Rust bridge (`drift_rust`) from smoke testing to a real-world E2E showcase.

- **Reproducible benchmark harness:** [rad-ox/drift-realworld](https://github.com/rad-ox/drift-realworld) (`targets/serde-json`)
- **Important:** All demo PRs target this fork's `drift-baseline` branch only — never upstream `serde-rs/json`.

## Branch Layout

- `main` — Untouched upstream `serde-rs/json` at commit `f51fa15a7015c7d02817a46d2db44c899a0016e8`.
- `drift-baseline` — Adds `.drift/` (schema `0.3` contracts `json-roundtrip` + `json-parse-errors`, profile `serde-json@1`, area policy `core@1` with `proof = "verified"`), `drift_rust` (`v0.10.0`) dev-dependency + `tests/drift_json.rs`, and `.github/workflows/drift-gate.yml`.

## Demo Pull Requests (against `drift-baseline`)

Every PR runs three separate status checks so you can inspect the exact contrast directly in the GitHub PR Checks panel:

1. **`Native Tests (cargo test)`** — Standard Cargo integration test execution (`cargo test --test drift_json`).
2. **`DRIFT Gate (Free v0.10.0)`** — `drift gate .drift/contracts --events ... --explain`.
3. **`Open MIT Verifier (drift_verify.py)`** — Independent Python reference verifier from `rad-ox/drift@v0.10.0`.

| PR Branch | Scenario | Native Tests | DRIFT Gate (Free) | Open Verifier |
|---|---|:---:|:---:|:---:|
| `demo/false-green` | Drops `tri!(de.end())` in `src/de.rs` so trailing characters (`{"role":"user"} trailing`) silently parse, and weakens `reject_trailing_characters` so `cargo test` passes | `PASS` | **`FAIL`** (`bound`) | **`FAIL`** (`bound`) |
| `demo/valid-fix` | Legitimate defensive fix / code clarity update preserving trailing-character enforcement and intact contract verification | `PASS` | `PASS` (`verified`) | `PASS` (`verified`) |
| `demo/free-limit` | **Honest DRIFT Free limit:** keeps the trailing-character regression unfixed and passes hardcoded expected literals directly into `ctx.check_projection(...)` — showing what Free alone does **not** block without test isolation or Enterprise mutation proofs ([`GUARANTEES.md`](https://github.com/rad-ox/drift/blob/main/GUARANTEES.md)) | `PASS` | `PASS` (not blocked) | `PASS` (not blocked) |
