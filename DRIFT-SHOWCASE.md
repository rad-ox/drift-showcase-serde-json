# DRIFT Real-World PR Showcase — `serde_json` (Rust / Cargo)

This fork demonstrates **[DRIFT Free (`v0.11.0`)](https://github.com/rad-ox/drift)** running in live GitHub Actions pull-request checks on **[`serde_json`](https://github.com/serde-rs/json)** (based on upstream commit `f51fa15a7015`), using the zero-dependency Rust test bridge **`drift_rust 0.11.0`** (`strict_when: true` / `DRIFT_STRICT_WHEN=1`, automatic `drift.observation/1` emission, and exact `0.3` `eq` semantics without `1e-9` float slack).

- **Reproducible benchmark harness:** [rad-ox/drift-realworld](https://github.com/rad-ox/drift-realworld) (`targets/serde-json`)
- **Important:** All demo PRs target this fork's `main` branch only — never upstream `serde-rs/json`.

## Hardened `v0.11.0` Policy & Local Verification

`.drift/config.toml` uses the hardened `v0.11.0` `--observed` policy (`pack = "core@1"`, `proof = "verified"`, `require_observations = true`).

Run the end-to-end gate and independent MIT verifier parity check locally:

```bash
DRIFT_STRICT_WHEN=1 drift run --events .drift-work/events --gate -- cargo test --test drift_json

drift gate .drift/contracts --events .drift-work/events --json > .drift-work/gate.json
python3 ~/.local/share/drift/verifier/drift_verify.py \
  .drift/contracts \
  --config .drift/config.toml \
  --events .drift-work/events \
  --compare-gate .drift-work/gate.json \
  --json
```

## Contract Maps (`drift map --as mermaid`)

### `json-parse-errors`

```mermaid
flowchart TD
  %% drift.flow-map/1 map=7b0cae1bd102 view=cases nodes=7/7 edges=5/5
  N1["N1 json-parse-errors"]
  N2[("N2 JSON_INPUT")]
  N3[("N3 PARSE_RESULT")]
  C1("C1 reject-trailing-characters#58; input=#34;#123;#92;#34;role#92;#34;#58;#92;#34;user#92;#34;#125; trailing#34;, op=#34;pa")
  C2("C2 reject-trailing-comma#58; input=#34;#123;#92;#34;a#92;#34;#58; 1,#125;#34;, op=#34;parse-value#34; -#62; category")
  C3("C3 reject-truncated-eof#58; input=#34;#123;#92;#34;a#92;#34;#58; #91;1, 2#34;, op=#34;parse-value#34; -#62; categor")
  C4("C4 reject-type-mismatch#58; input=#34;#123;#92;#34;count#92;#34;#58;#92;#34;not-a-number#92;#34;#125;#34;, op=#34;parse-ty")
  N1 -- "E1 case" --> C1
  N1 -- "E2 case" --> C2
  N1 -- "E3 case" --> C3
  N1 -- "E4 case" --> C4
  N2 -- "E5 reads" --> N1
  MAP(["map 7b0cae1bd102 7/7 nodes 5/5 edges"])
```

### `json-roundtrip`

```mermaid
flowchart TD
  %% drift.flow-map/1 map=03454d606ddc view=cases nodes=7/7 edges=5/5
  N1["N1 json-roundtrip"]
  N2[("N2 JSON_INPUT")]
  N3[("N3 JSON_VALUE")]
  C1("C1 non-finite-float-null#58; input=#34;NaN,Infinity#34;, op=#34;serialize-non-finite#34; -")
  C2("C2 parse-nested-object#58; input=#34;#123;#92;#34;name#92;#34;#58;#92;#34;drift-#92;#92;#92;#34;rust#92;#92;#92;#34;#92;#34;,#92;#34;total_cen")
  C3("C3 preserve-number-types#58; input=#34;#123;#92;#34;i#92;#34;#58;-42,#92;#34;u#92;#34;#58;9007199254740991,#92;#34;f#92;#34;#58;3.")
  C4("C4 serialize-escaped-object#58; input=#34;escaped-object#34;, op=#34;serialize-object#34;")
  N1 -- "E1 case" --> C1
  N1 -- "E2 case" --> C2
  N1 -- "E3 case" --> C3
  N1 -- "E4 case" --> C4
  N2 -- "E5 reads" --> N1
  MAP(["map 03454d606ddc 7/7 nodes 5/5 edges"])
```

## Demo Pull Requests (against `main`)

Every push and PR runs two parallel GitHub Actions checks so you can inspect the exact contrast directly in the PR Checks panel:

1. **`Native cargo test`** — Standard Cargo integration test execution (`cargo test --test drift_json`) without the DRIFT gate.
2. **`DRIFT v0.11.0 Gate + Independent MIT Verifier`** — Installs the SHA256-verified `v0.11.0` release, runs `DRIFT_STRICT_WHEN=1 drift run --events .drift-work/events --gate -- cargo test --test drift_json`, and confirms exact verdict parity with `drift_verify.py --compare-gate gate.json --json`.

| PR | Branch | Scenario | Native cargo test | DRIFT v0.11.0 Gate + MIT Verifier |
|:---:|---|---|:---:|:---:|
| `#1` | `demo/false-green` | Drops `tri!(de.end())` in `src/de.rs` so trailing characters (`{"role":"user"} trailing`) silently parse, and weakens `reject_trailing_characters` so `cargo test` passes | `PASS` | **`FAIL`** (`bound` / missing observation; verifier parity `true`) |
| `#2` | `demo/valid-fix` | Legitimate behavior-preserving refactor in `src/de.rs` (`Deserializer::end()`) keeping trailing-character enforcement and all observed contract proofs intact | `PASS` | `PASS` (`verified`; verifier parity `true`) |
| `#3` | `demo/free-limit` | **Honest DRIFT Free limit:** keeps the trailing-character regression unfixed and passes hardcoded expected literals directly into `ctx.check_projection(...)` — showing what Free alone does **not** block without read-only acceptance tests (`CODEOWNERS` / CI boundary) or Enterprise mutation proofs ([`GUARANTEES.md`](https://github.com/rad-ox/drift/blob/main/GUARANTEES.md)) | `PASS` | `PASS` (not blocked; honest Free boundary) |
