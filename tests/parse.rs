use rscsvj::{parse, Value};

fn s(text: &str) -> Value {
    Value::String(text.to_string())
}

#[test]
fn simple_strings() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n\"Row1\", \"Row2\", \"Row3\"\n";
    let table = parse(csvj).unwrap();
    assert_eq!(table.header, vec!["Header1", "Header2", "Header3"]);
    assert_eq!(table.rows.len(), 1);
    assert_eq!(table.rows[0], vec![s("Row1"), s("Row2"), s("Row3")]);
}

#[test]
#[allow(clippy::approx_constant)]
fn mixed_types() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, 3.14, false\nnull, true, \"trailing\"\n";
    let table = parse(csvj).unwrap();
    assert_eq!(
        table.rows[0],
        vec![Value::Number(42.0), Value::Number(3.14), Value::Bool(false)]
    );
    assert_eq!(
        table.rows[1],
        vec![Value::Null, Value::Bool(true), s("trailing")]
    );
}

#[test]
fn crlf_line_endings() {
    let csvj = "\"h1\",\"h2\",\"h3\"\r\n2,4,5\r\n";
    let table = parse(csvj).unwrap();
    assert_eq!(table.header, vec!["h1", "h2", "h3"]);
    assert_eq!(
        table.rows[0],
        vec![Value::Number(2.0), Value::Number(4.0), Value::Number(5.0)]
    );
}

#[test]
fn null_in_row() {
    let csvj = "\"h1\",\"h2\",\"h3\"\n\"test\",null,42\n";
    let table = parse(csvj).unwrap();
    assert_eq!(
        table.rows[0],
        vec![s("test"), Value::Null, Value::Number(42.0)]
    );
}

#[test]
fn empty_header_line() {
    let table = parse("\n").unwrap();
    assert!(table.header.is_empty());
    assert!(table.rows.is_empty());
}

#[test]
fn empty_header_line_crlf() {
    let table = parse("\r\n").unwrap();
    assert!(table.header.is_empty());
}

#[test]
fn trailing_null_value() {
    let csvj = "\"a\", \"b\", \"c\"\n\"x\", \"y\", null\n";
    let table = parse(csvj).unwrap();
    assert_eq!(table.rows[0], vec![s("x"), s("y"), Value::Null]);
}

#[test]
fn trailing_empty_string() {
    let csvj = "\"a\", \"b\", \"c\"\n\"x\", \"y\", \"\"\n";
    let table = parse(csvj).unwrap();
    assert_eq!(table.rows[0], vec![s("x"), s("y"), s("")]);
}

#[test]
fn utf8_values() {
    let csvj = "\"h1\", \"h2\", \"h3\"\n\"héllo\", \"日本語\", \"🚀\"\n";
    let table = parse(csvj).unwrap();
    assert_eq!(table.rows[0], vec![s("héllo"), s("日本語"), s("🚀")]);
}

#[test]
fn json_string_escapes() {
    let csvj = "\"h\"\n\"line1\\nline2\\there\\\"end\\\\\"\n";
    let table = parse(csvj).unwrap();
    assert_eq!(table.rows[0], vec![s("line1\nline2\there\"end\\")]);
}

#[test]
fn unicode_escapes_with_surrogate_pair() {
    // 😀 = U+1F600 GRINNING FACE
    let csvj = "\"h1\", \"h2\"\n\"\\u00e9\", \"\\uD83D\\uDE00\"\n";
    let table = parse(csvj).unwrap();
    assert_eq!(table.rows[0], vec![s("é"), s("😀")]);
}

#[test]
fn number_forms() {
    let csvj = "\"h1\", \"h2\", \"h3\", \"h4\", \"h5\"\n-1, 0, 1.5, 1e10, -2.5e-3\n";
    let table = parse(csvj).unwrap();
    assert_eq!(
        table.rows[0],
        vec![
            Value::Number(-1.0),
            Value::Number(0.0),
            Value::Number(1.5),
            Value::Number(1e10),
            Value::Number(-2.5e-3),
        ]
    );
}

#[test]
fn booleans_and_null() {
    let csvj = "\"h1\", \"h2\", \"h3\", \"h4\"\ntrue, false, null, \"string\"\n";
    let table = parse(csvj).unwrap();
    assert_eq!(
        table.rows[0],
        vec![
            Value::Bool(true),
            Value::Bool(false),
            Value::Null,
            s("string"),
        ]
    );
}

#[test]
fn multiple_rows() {
    let csvj = "\"h1\", \"h2\"\n1, 2\n3, 4\n5, 6\n";
    let table = parse(csvj).unwrap();
    assert_eq!(table.rows.len(), 3);
    assert_eq!(table.rows[0], vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(table.rows[2], vec![Value::Number(5.0), Value::Number(6.0)]);
}

#[test]
fn long_value_4kib() {
    let long = "a".repeat(4096);
    let csvj = format!("\"h1\"\n\"{}\"\n", long);
    let table = parse(&csvj).unwrap();
    assert_eq!(table.rows[0], vec![s(&long)]);
}

#[test]
fn empty_input_rejected() {
    assert!(parse("").is_err());
}

#[test]
fn missing_trailing_newline_rejected() {
    assert!(parse("\"h1\"").is_err());
    assert!(parse("\"h1\"\n\"row1\"").is_err());
}

#[test]
fn ragged_row_short_rejected() {
    let csvj = "\"a\", \"b\", \"c\"\n\"x\", \"y\"\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn ragged_row_long_rejected() {
    let csvj = "\"a\", \"b\"\n\"x\", \"y\", \"z\"\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn duplicate_header_rejected() {
    let csvj = "\"a\", \"b\", \"a\"\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn duplicate_empty_header_rejected() {
    let csvj = "\"\", \"\"\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn non_string_header_rejected() {
    let csvj = "\"Header1\", 1, \"Header2\", \"Header3\"\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn invalid_value_token_rejected() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, $, false\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn array_value_rejected() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, [], false\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn object_value_rejected() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, {}, false\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn leading_zeros_rejected() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, 0123, false\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn bare_dot_number_rejected() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, .5, false\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn trailing_dot_number_rejected() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, 1., false\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn nan_rejected() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, NaN, false\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn infinity_rejected() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, Infinity, false\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn uppercase_true_rejected() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, True, false\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn uppercase_null_rejected() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, Null, false\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn single_quoted_string_rejected() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, 'hello', false\n";
    assert!(parse(csvj).is_err());
}

#[test]
fn unescaped_control_char_rejected() {
    let csvj = "\"Header1\", \"Header2\", \"Header3\"\n42, \"hello\x01world\", false\n";
    assert!(parse(csvj).is_err());
}
