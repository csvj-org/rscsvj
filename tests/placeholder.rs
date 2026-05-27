use rscsvj::{parse, stringify, Table};

#[test]
fn parse_placeholder_returns_not_implemented() {
    let err = parse("\n").expect_err("parse should be unimplemented");
    assert!(err.to_string().contains("not yet implemented"));
}

#[test]
fn stringify_placeholder_returns_not_implemented() {
    let err = stringify(&Table::default()).expect_err("stringify should be unimplemented");
    assert!(err.to_string().contains("not yet implemented"));
}
