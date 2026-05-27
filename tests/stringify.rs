use rscsvj::{parse, stringify, Table, Value};

fn s(text: &str) -> Value {
    Value::String(text.to_string())
}

#[test]
fn simple_writer() {
    let table = Table {
        header: vec!["h1".into(), "h2".into(), "h3".into()],
        rows: vec![vec![
            Value::Number(2.0),
            Value::Number(4.0),
            Value::Number(5.0),
        ]],
    };
    let out = stringify(&table).unwrap();
    assert_eq!(out, "\"h1\",\"h2\",\"h3\"\n2.0,4.0,5.0\n");
}

#[test]
fn header_only() {
    let table = Table {
        header: vec!["h1".into()],
        rows: vec![],
    };
    let out = stringify(&table).unwrap();
    assert_eq!(out, "\"h1\"\n");
}

#[test]
fn empty_header_and_no_rows() {
    let table = Table {
        header: vec![],
        rows: vec![],
    };
    let out = stringify(&table).unwrap();
    assert_eq!(out, "\n");
}

#[test]
fn mixed_types_writer() {
    let table = Table {
        header: vec!["h1".into(), "h2".into(), "h3".into()],
        rows: vec![vec![s("test"), Value::Null, Value::Number(42.0)]],
    };
    let out = stringify(&table).unwrap();
    assert_eq!(out, "\"h1\",\"h2\",\"h3\"\n\"test\",null,42.0\n");
}

#[test]
fn duplicate_header_rejected() {
    let table = Table {
        header: vec!["a".into(), "b".into(), "a".into()],
        rows: vec![],
    };
    assert!(stringify(&table).is_err());
}

#[test]
fn duplicate_empty_header_rejected() {
    let table = Table {
        header: vec!["".into(), "".into()],
        rows: vec![],
    };
    assert!(stringify(&table).is_err());
}

#[test]
fn row_length_mismatch_rejected() {
    let table = Table {
        header: vec!["h1".into(), "h2".into(), "h3".into()],
        rows: vec![vec![Value::Number(1.0), Value::Number(2.0)]],
    };
    assert!(stringify(&table).is_err());
}

#[test]
fn nan_rejected() {
    let table = Table {
        header: vec!["h1".into()],
        rows: vec![vec![Value::Number(f64::NAN)]],
    };
    assert!(stringify(&table).is_err());
}

#[test]
fn infinity_rejected() {
    let table = Table {
        header: vec!["h1".into()],
        rows: vec![vec![Value::Number(f64::INFINITY)]],
    };
    assert!(stringify(&table).is_err());
}

#[test]
fn utf8_round_trip() {
    let table = Table {
        header: vec!["h1".into(), "h2".into(), "h3".into()],
        rows: vec![vec![s("héllo"), s("日本語"), s("🚀")]],
    };
    let out = stringify(&table).unwrap();
    let parsed = parse(&out).unwrap();
    assert_eq!(parsed, table);
}

#[test]
fn string_escapes_round_trip() {
    let table = Table {
        header: vec!["h1".into(), "h2".into(), "h3".into()],
        rows: vec![vec![s("line1\nline2"), s("quote\"end"), s("back\\slash")]],
    };
    let out = stringify(&table).unwrap();
    let parsed = parse(&out).unwrap();
    assert_eq!(parsed, table);
}

#[test]
fn booleans_round_trip() {
    let table = Table {
        header: vec!["h1".into(), "h2".into()],
        rows: vec![vec![Value::Bool(true), Value::Bool(false)]],
    };
    let out = stringify(&table).unwrap();
    let parsed = parse(&out).unwrap();
    assert_eq!(parsed, table);
}

#[test]
fn multiple_rows_round_trip() {
    let table = Table {
        header: vec!["h1".into(), "h2".into()],
        rows: vec![
            vec![Value::Number(1.0), Value::Number(2.0)],
            vec![Value::Number(3.0), Value::Number(4.0)],
            vec![Value::Number(5.0), Value::Number(6.0)],
        ],
    };
    let out = stringify(&table).unwrap();
    let parsed = parse(&out).unwrap();
    assert_eq!(parsed, table);
}
