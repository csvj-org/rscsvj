# rscsvj

[![CI](https://github.com/csvj-org/rscsvj/actions/workflows/ci.yml/badge.svg)](https://github.com/csvj-org/rscsvj/actions/workflows/ci.yml)

Rust reader and writer for [CSVJ](https://csvj.org) files. MSRV 1.74.

## Overview

CSVJ is a tabular data format where each value is a JSON literal. The
spec is at <https://csvj.org>; the Go reference implementation lives at
[csvj-org/gocsvj](https://github.com/csvj-org/gocsvj), the JavaScript
reference at [csvj-org/jscsvj](https://github.com/csvj-org/jscsvj), the
PHP reference at [csvj-org/phpcsvj](https://github.com/csvj-org/phpcsvj),
and the language-agnostic conformance suite at
[csvj-org/conformance](https://github.com/csvj-org/conformance).

The reader will enforce every §1 rule (empty input rejected; trailing
newline required; ragged rows rejected; duplicate header names
rejected; only `String | Number | Bool | Null` permitted at value
position; JSON lexical rules per RFC 8259) and pass all 25 vectors of
`csvj-org/conformance@master`.

## Parse

```rust
use rscsvj::{parse, Value};

let table = parse("\"name\",\"age\"\n\"alice\",30\n\"bob\",null\n").unwrap();
assert_eq!(table.header, vec!["name", "age"]);
assert_eq!(table.rows[0], vec![Value::String("alice".into()), Value::Number(30.0)]);
assert_eq!(table.rows[1], vec![Value::String("bob".into()), Value::Null]);
```

The returned [`Table`] has a `header` field (`Vec<String>`, zero or more
column names) and a `rows` field (`Vec<Vec<Value>>`) where every row has
exactly `table.header.len()` values. Each value is `Value::String`,
`Value::Number`, `Value::Bool`, or `Value::Null`.

Parsing rejects every input the spec says must be rejected — see the
[conformance suite](https://github.com/csvj-org/conformance) for the
full list. Invalid input returns `Err(rscsvj::ParseError)`.

## Serialize

```rust
use rscsvj::{stringify, Table, Value};

let table = Table {
    header: vec!["name".into(), "age".into()],
    rows: vec![
        vec![Value::String("alice".into()), Value::Number(30.0)],
        vec![Value::String("bob".into()),   Value::Null],
    ],
};
let bytes = stringify(&table).unwrap();
assert_eq!(bytes, "\"name\",\"age\"\n\"alice\",30\n\"bob\",null\n");
```

The output is always spec-compliant CSVJ: terminated by `\n`, every row
has exactly `table.header.len()` values, and every value is encoded as
a JSON literal.

## Status

`parse` and `stringify` are currently placeholders that return
`Err("not yet implemented")`. The public surface (types and signatures)
is stable so consumers can pin against it before the reader/writer
implementation lands (PLAN §7b.2).

## Install

```sh
cargo add rscsvj
```

(Not yet published to crates.io — see
[PLAN.md §6d](https://github.com/csvj-org/website) for the publication
checklist.)

## License

Dual-licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.
