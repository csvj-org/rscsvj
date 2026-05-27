//! Reader and writer for [CSVJ](https://csvj.org) files.
//!
//! The public surface is two functions:
//!
//! - [`parse`] — parse CSVJ bytes into a [`Table`].
//! - [`stringify`] — serialize a [`Table`] back to CSVJ bytes.
//!
//! Strict §1 enforcement from day one: empty input, missing trailing
//! newline, ragged rows, and duplicate header names are rejected. Value
//! tokens follow RFC 8259 strictly (no leading zeros, no bare `.5`, no
//! trailing `1.`, no `NaN` / `Infinity`, no uppercase keywords, no
//! single-quoted strings, no unescaped control characters in strings).

use std::error::Error as StdError;
use std::fmt;

use serde_json::Value as JsonValue;

/// A parsed CSVJ table: a header row plus zero or more data rows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Table {
    pub header: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}

/// A CSVJ value. CSVJ inherits JSON's value model restricted to scalars:
/// strings, numbers, booleans, and `null` (no nested arrays or objects).
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

/// Error returned by [`parse`] when the input is not valid CSVJ.
#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for ParseError {}

/// Error returned by [`stringify`] when the table cannot be serialized.
#[derive(Debug)]
pub struct WriteError {
    message: String,
}

impl WriteError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for WriteError {}

/// Parse CSVJ bytes into a [`Table`].
pub fn parse(input: &str) -> Result<Table, ParseError> {
    if input.is_empty() {
        return Err(ParseError::new("empty input"));
    }
    if !input.ends_with('\n') {
        return Err(ParseError::new("file does not end with newline"));
    }

    let body = &input[..input.len() - 1];
    let mut raw_lines: Vec<&str> = body.split('\n').collect();
    for line in raw_lines.iter_mut() {
        if let Some(stripped) = line.strip_suffix('\r') {
            *line = stripped;
        }
    }

    let header_values = parse_line(raw_lines[0], "header")?;
    let mut header: Vec<String> = Vec::with_capacity(header_values.len());
    for v in header_values {
        match v {
            Value::String(s) => header.push(s),
            _ => return Err(ParseError::new("non-string item at csvj header")),
        }
    }

    let mut seen = std::collections::HashSet::with_capacity(header.len());
    for name in &header {
        if !seen.insert(name.as_str()) {
            return Err(ParseError::new(format!(
                "duplicate header name \"{}\"",
                name
            )));
        }
    }

    let header_len = header.len();
    let mut rows = Vec::with_capacity(raw_lines.len().saturating_sub(1));
    for (idx, line) in raw_lines.iter().enumerate().skip(1) {
        let label = format!("row {}", idx);
        let row = parse_line(line, &label)?;
        if row.len() != header_len {
            return Err(ParseError::new(format!(
                "row {} has {} values, header has {}",
                idx,
                row.len(),
                header_len
            )));
        }
        rows.push(row);
    }

    Ok(Table { header, rows })
}

fn parse_line(body: &str, label: &str) -> Result<Vec<Value>, ParseError> {
    let wrapped = format!("[{}]", body);
    let parsed: JsonValue = serde_json::from_str(&wrapped)
        .map_err(|e| ParseError::new(format!("{} parse error: {}", label, e)))?;

    let arr = match parsed {
        JsonValue::Array(a) => a,
        _ => {
            return Err(ParseError::new(format!(
                "{} parse error: not a JSON array",
                label
            )))
        }
    };

    let mut out = Vec::with_capacity(arr.len());
    for (i, v) in arr.into_iter().enumerate() {
        let converted = match v {
            JsonValue::Null => Value::Null,
            JsonValue::Bool(b) => Value::Bool(b),
            JsonValue::String(s) => Value::String(s),
            JsonValue::Number(n) => match n.as_f64() {
                Some(f) => Value::Number(f),
                None => {
                    return Err(ParseError::new(format!(
                        "{} parse error at item {}",
                        label, i
                    )))
                }
            },
            JsonValue::Array(_) | JsonValue::Object(_) => {
                return Err(ParseError::new(format!(
                    "{} parse error at item {}",
                    label, i
                )))
            }
        };
        out.push(converted);
    }
    Ok(out)
}

/// Serialize a [`Table`] back to CSVJ bytes.
///
/// Output is terminated by a single `\n`. Every row must have exactly
/// `table.header.len()` values; duplicate header names and non-finite
/// numbers are rejected so the writer can never produce a file the
/// strict reader would refuse.
pub fn stringify(table: &Table) -> Result<String, WriteError> {
    let mut seen = std::collections::HashSet::with_capacity(table.header.len());
    for name in &table.header {
        if !seen.insert(name.as_str()) {
            return Err(WriteError::new(format!(
                "duplicate header name \"{}\"",
                name
            )));
        }
    }

    let header_row: Vec<Value> = table
        .header
        .iter()
        .map(|s| Value::String(s.clone()))
        .collect();
    let header_len = table.header.len();

    let mut out = String::new();
    out.push_str(&serialize_row(&header_row, "header")?);
    out.push('\n');

    for (i, row) in table.rows.iter().enumerate() {
        if row.len() != header_len {
            return Err(WriteError::new(format!(
                "row {} has {} values, expected {}",
                i,
                row.len(),
                header_len
            )));
        }
        out.push_str(&serialize_row(row, &format!("row {}", i))?);
        out.push('\n');
    }
    Ok(out)
}

fn serialize_row(row: &[Value], label: &str) -> Result<String, WriteError> {
    let arr: Vec<JsonValue> = row
        .iter()
        .enumerate()
        .map(|(i, v)| value_to_json(v, label, i))
        .collect::<Result<_, _>>()?;
    let json = serde_json::to_string(&JsonValue::Array(arr))
        .map_err(|e| WriteError::new(format!("{}: json encode failed: {}", label, e)))?;
    Ok(json[1..json.len() - 1].to_string())
}

fn value_to_json(v: &Value, label: &str, idx: usize) -> Result<JsonValue, WriteError> {
    match v {
        Value::Null => Ok(JsonValue::Null),
        Value::Bool(b) => Ok(JsonValue::Bool(*b)),
        Value::String(s) => Ok(JsonValue::String(s.clone())),
        Value::Number(f) => {
            if !f.is_finite() {
                return Err(WriteError::new(format!(
                    "{}: item {} is not CSVJ type-safe: non-finite number",
                    label, idx
                )));
            }
            let n = serde_json::Number::from_f64(*f).ok_or_else(|| {
                WriteError::new(format!(
                    "{}: item {} cannot be represented as JSON number",
                    label, idx
                ))
            })?;
            Ok(JsonValue::Number(n))
        }
    }
}
