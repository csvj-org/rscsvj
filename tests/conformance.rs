//! Run the language-agnostic vectors from `csvj-org/conformance` against
//! `rscsvj::parse`. The conformance directory is located via the
//! `CSVJ_CONFORMANCE_DIR` environment variable; if unset, the suite is
//! skipped so `cargo test` works in any checkout.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use rscsvj::{parse, Table, Value};
use serde_json::Value as JsonValue;

fn conformance_dir() -> Option<PathBuf> {
    std::env::var_os("CSVJ_CONFORMANCE_DIR").map(PathBuf::from)
}

fn case_stems(dir: &Path) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {}", dir.display(), e))
        .filter_map(Result::ok)
        .filter_map(|e| {
            let path = e.path();
            if path.extension().and_then(|s| s.to_str()) != Some("csvj") {
                return None;
            }
            path.file_stem().and_then(|s| s.to_str()).map(String::from)
        })
        .collect();
    out.sort();
    out
}

fn table_to_json(table: &Table) -> JsonValue {
    let mut arrays: Vec<JsonValue> = Vec::with_capacity(1 + table.rows.len());
    arrays.push(JsonValue::Array(
        table
            .header
            .iter()
            .map(|s| JsonValue::String(s.clone()))
            .collect(),
    ));
    for row in &table.rows {
        arrays.push(JsonValue::Array(
            row.iter()
                .map(|v| match v {
                    Value::Null => JsonValue::Null,
                    Value::Bool(b) => JsonValue::Bool(*b),
                    Value::String(s) => JsonValue::String(s.clone()),
                    Value::Number(f) => serde_json::Number::from_f64(*f)
                        .map(JsonValue::Number)
                        .unwrap_or(JsonValue::Null),
                })
                .collect(),
        ));
    }
    JsonValue::Array(arrays)
}

/// Compare two JSON values structurally with number equality on f64 to
/// paper over `42` vs `42.0` differences between source dialects.
fn json_equivalent(a: &JsonValue, b: &JsonValue) -> bool {
    match (a, b) {
        (JsonValue::Number(x), JsonValue::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(xf), Some(yf)) => xf == yf,
            _ => x == y,
        },
        (JsonValue::Array(xs), JsonValue::Array(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys.iter()).all(|(a, b)| json_equivalent(a, b))
        }
        (JsonValue::Object(xs), JsonValue::Object(ys)) => {
            let kx: HashSet<_> = xs.keys().collect();
            let ky: HashSet<_> = ys.keys().collect();
            kx == ky && xs.iter().all(|(k, v)| json_equivalent(v, &ys[k]))
        }
        _ => a == b,
    }
}

#[test]
fn accept_vectors() {
    let Some(root) = conformance_dir() else {
        eprintln!("CSVJ_CONFORMANCE_DIR not set; skipping accept vectors");
        return;
    };
    let inputs = root.join("inputs");
    let expected = root.join("expected");
    let stems = case_stems(&inputs);
    assert!(
        !stems.is_empty(),
        "no accept vectors found in {}",
        inputs.display()
    );

    for stem in stems {
        let input_path = inputs.join(format!("{}.csvj", stem));
        let expected_path = expected.join(format!("{}.json", stem));

        let bytes = fs::read_to_string(&input_path)
            .unwrap_or_else(|e| panic!("read {}: {}", input_path.display(), e));
        let expected_text = fs::read_to_string(&expected_path)
            .unwrap_or_else(|e| panic!("read {}: {}", expected_path.display(), e));
        let expected_json: JsonValue = serde_json::from_str(&expected_text)
            .unwrap_or_else(|e| panic!("parse expected for {}: {}", stem, e));

        let table = parse(&bytes).unwrap_or_else(|e| panic!("parse {}: {}", stem, e));
        let actual_json = table_to_json(&table);

        assert!(
            json_equivalent(&actual_json, &expected_json),
            "vector {} mismatch:\n  actual:   {}\n  expected: {}",
            stem,
            actual_json,
            expected_json,
        );
    }
}

#[test]
fn must_reject_vectors() {
    let Some(root) = conformance_dir() else {
        eprintln!("CSVJ_CONFORMANCE_DIR not set; skipping must-reject vectors");
        return;
    };
    let reject_dir = root.join("must-reject");
    let stems = case_stems(&reject_dir);
    assert!(
        !stems.is_empty(),
        "no must-reject vectors found in {}",
        reject_dir.display()
    );

    for stem in stems {
        let input_path = reject_dir.join(format!("{}.csvj", stem));
        let bytes = fs::read_to_string(&input_path)
            .unwrap_or_else(|e| panic!("read {}: {}", input_path.display(), e));
        assert!(
            parse(&bytes).is_err(),
            "must-reject vector {} was accepted",
            stem
        );
    }
}
