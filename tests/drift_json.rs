use drift_rust::{
    canonical_rust_test_id, discover_binary_tests, run_drift_case, RunOptions, TestError,
};
use serde::Deserialize;
use serde_json::error::Category;
use serde_json::{Map, Value};
use std::sync::Once;

static DISCOVER_ONCE: Once = Once::new();

fn ensure_inventory() {
    DISCOVER_ONCE.call_once(|| {
        let _ = discover_binary_tests("drift_json", "");
    });
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn category_name(cat: Category) -> &'static str {
    match cat {
        Category::Io => "io",
        Category::Syntax => "syntax",
        Category::Data => "data",
        Category::Eof => "eof",
    }
}

fn run_case(
    contract: &str,
    fn_name: &str,
    case_id: &str,
    body: impl Fn(&mut drift_rust::DriftContext) -> Result<(), TestError>,
) {
    ensure_inventory();
    let tid = canonical_rust_test_id("drift_json", module_path!(), fn_name);
    let outcome = run_drift_case(
        RunOptions {
            contract: contract.to_string(),
            test: tid,
            cases: vec![case_id.to_string()],
            level: "unit".to_string(),
            strict_when: true,
            ..Default::default()
        },
        body,
    )
    .unwrap_or_else(|e| panic!("DRIFT case {}#{} failed: {}", contract, case_id, e));
    assert_eq!(outcome, "passed");
}

#[test]
fn serialize_escaped_object() {
    run_case(
        "json-roundtrip",
        "serialize_escaped_object",
        "serialize-escaped-object",
        |ctx| {
            let op = ctx.when_str("serialize-escaped-object", "op")?;
            let input_kind = ctx.when_str("serialize-escaped-object", "input")?;
            if op != "serialize-object" || input_kind != "escaped-object" {
                return Err(TestError::harness(format!(
                    "unexpected fixture ({op}, {input_kind})"
                )));
            }
            let mut map = Map::new();
            map.insert("active".to_string(), Value::Bool(true));
            map.insert("count".to_string(), Value::from(-42_i64));
            map.insert("note".to_string(), Value::String("line1\n\"quoted\"".to_string()));
            map.insert("unset".to_string(), Value::Null);
            let original = Value::Object(map);

            let serialized = serde_json::to_string(&original)
                .map_err(|e| TestError::assertion(format!("serialize failed: {}", e)))?;
            let reparsed: Value = serde_json::from_str(&serialized)
                .map_err(|e| TestError::assertion(format!("reparse failed: {}", e)))?;
            let key_count = reparsed.as_object().map_or(0, Map::len);
            let roundtrip_equal = reparsed == original;

            let obs = format!(
                "{{\"ok\":true,\"json\":{},\"key_count\":{},\"roundtrip_equal\":{}}}",
                json_escape(&serialized),
                key_count,
                roundtrip_equal
            );
            ctx.check_projection("serialize-escaped-object", &obs)
        },
    );
}

#[test]
fn parse_nested_object() {
    run_case(
        "json-roundtrip",
        "parse_nested_object",
        "parse-nested-object",
        |ctx| {
            let _op = ctx.when_str("parse-nested-object", "op")?;
            let input = ctx.when_str("parse-nested-object", "input")?;
            let parsed: Value = serde_json::from_str(&input)
                .map_err(|e| TestError::assertion(format!("parse failed: {}", e)))?;
            let obj = parsed
                .as_object()
                .ok_or_else(|| TestError::assertion("expected JSON object"))?;
            let name = obj.get("name").and_then(Value::as_str).unwrap_or_default();
            let total_cents = obj.get("total_cents").and_then(Value::as_i64).unwrap_or(-1);
            let tag = obj
                .get("tags")
                .and_then(Value::as_array)
                .and_then(|arr| arr.get(1))
                .and_then(Value::as_str)
                .unwrap_or_default();

            let obs = format!(
                "{{\"ok\":true,\"key_count\":{},\"name\":{},\"total_cents\":{},\"tag\":{}}}",
                obj.len(),
                json_escape(name),
                total_cents,
                json_escape(tag)
            );
            ctx.check_projection("parse-nested-object", &obs)
        },
    );
}

#[test]
fn non_finite_float_null() {
    run_case(
        "json-roundtrip",
        "non_finite_float_null",
        "non-finite-float-null",
        |ctx| {
            let _op = ctx.when_str("non-finite-float-null", "op")?;
            let raw_input = ctx.when_str("non-finite-float-null", "input")?;
            let vals: Vec<f64> = raw_input
                .split(',')
                .map(|tok| match tok.trim() {
                    "NaN" => f64::NAN,
                    "Infinity" => f64::INFINITY,
                    "-Infinity" => f64::NEG_INFINITY,
                    other => other.parse::<f64>().unwrap_or(f64::NAN),
                })
                .collect();
            let serialized = serde_json::to_string(&vals)
                .map_err(|e| TestError::assertion(format!("serialize failed: {}", e)))?;

            let obs = format!("{{\"ok\":true,\"json\":{}}}", json_escape(&serialized));
            ctx.check_projection("non-finite-float-null", &obs)
        },
    );
}

#[test]
fn preserve_number_types() {
    run_case(
        "json-roundtrip",
        "preserve_number_types",
        "preserve-number-types",
        |ctx| {
            let _op = ctx.when_str("preserve-number-types", "op")?;
            let input = ctx.when_str("preserve-number-types", "input")?;
            let parsed: Value = serde_json::from_str(&input)
                .map_err(|e| TestError::assertion(format!("parse failed: {}", e)))?;
            let obj = parsed
                .as_object()
                .ok_or_else(|| TestError::assertion("expected JSON object"))?;
            let is_i64 = obj.get("i").and_then(Value::as_i64) == Some(-42);
            let is_u64 = obj.get("u").and_then(Value::as_u64) == Some(9_007_199_254_740_991);
            let is_f64 = obj
                .get("f")
                .and_then(Value::as_f64)
                .is_some_and(|v| (v - 3.5).abs() < f64::EPSILON);

            let obs = format!(
                "{{\"ok\":true,\"key_count\":{},\"is_i64\":{},\"is_u64\":{},\"is_f64\":{}}}",
                obj.len(),
                is_i64,
                is_u64,
                is_f64
            );
            ctx.check_projection("preserve-number-types", &obs)
        },
    );
}

fn observe_value_parse(input: &str) -> String {
    match serde_json::from_str::<Value>(input) {
        Ok(_) => "{\"ok\":true,\"category\":\"none\",\"error_line\":0,\"error_col\":0}".to_string(),
        Err(err) => format!(
            "{{\"ok\":false,\"category\":{},\"error_line\":{},\"error_col\":{}}}",
            json_escape(category_name(err.classify())),
            err.line(),
            err.column()
        ),
    }
}

#[test]
fn reject_trailing_characters() {
    run_case(
        "json-parse-errors",
        "reject_trailing_characters",
        "reject-trailing-characters",
        |ctx| {
            let _op = ctx.when_str("reject-trailing-characters", "op")?;
            let input = ctx.when_str("reject-trailing-characters", "input")?;
            let _ = serde_json::from_str::<Value>(&input);
            Ok(())
        },
    );
}

#[test]
fn reject_trailing_comma() {
    run_case(
        "json-parse-errors",
        "reject_trailing_comma",
        "reject-trailing-comma",
        |ctx| {
            let _op = ctx.when_str("reject-trailing-comma", "op")?;
            let input = ctx.when_str("reject-trailing-comma", "input")?;
            let obs = observe_value_parse(&input);
            ctx.check_projection("reject-trailing-comma", &obs)
        },
    );
}

#[test]
fn reject_truncated_eof() {
    run_case(
        "json-parse-errors",
        "reject_truncated_eof",
        "reject-truncated-eof",
        |ctx| {
            let _op = ctx.when_str("reject-truncated-eof", "op")?;
            let input = ctx.when_str("reject-truncated-eof", "input")?;
            let obs = observe_value_parse(&input);
            ctx.check_projection("reject-truncated-eof", &obs)
        },
    );
}

#[derive(Deserialize)]
struct TypedCounter {
    #[allow(dead_code)]
    count: u64,
}

#[test]
fn reject_type_mismatch() {
    run_case(
        "json-parse-errors",
        "reject_type_mismatch",
        "reject-type-mismatch",
        |ctx| {
            let _op = ctx.when_str("reject-type-mismatch", "op")?;
            let input = ctx.when_str("reject-type-mismatch", "input")?;
            let obs = match serde_json::from_str::<TypedCounter>(&input) {
                Ok(_) => "{\"ok\":true,\"category\":\"none\",\"error_line\":0}".to_string(),
                Err(err) => format!(
                    "{{\"ok\":false,\"category\":{},\"error_line\":{}}}",
                    json_escape(category_name(err.classify())),
                    err.line()
                ),
            };
            ctx.check_projection("reject-type-mismatch", &obs)
        },
    );
}
