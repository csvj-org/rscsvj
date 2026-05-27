//! Reader and writer for [CSVJ](https://csvj.org) files.
//!
//! The public surface is two functions:
//!
//! - [`parse`] — parse CSVJ bytes into a [`Table`].
//! - [`stringify`] — serialize a [`Table`] back to CSVJ bytes.
//!
//! Both are placeholders until the reader/writer implementation lands.

use std::error::Error as StdError;
use std::fmt;

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
///
/// Not yet implemented — returns [`ParseError`] with a "not yet implemented"
/// message. The signature is stable.
pub fn parse(_input: &str) -> Result<Table, ParseError> {
    Err(ParseError::new("rscsvj::parse is not yet implemented"))
}

/// Serialize a [`Table`] back to CSVJ bytes.
///
/// Not yet implemented — returns [`WriteError`] with a "not yet implemented"
/// message. The signature is stable.
pub fn stringify(_table: &Table) -> Result<String, WriteError> {
    Err(WriteError::new("rscsvj::stringify is not yet implemented"))
}
