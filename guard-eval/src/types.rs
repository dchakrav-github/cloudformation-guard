use std::fmt::Formatter;
use guard_lang::{Location, RangeType, LangError};
use yaml_rust::ScanError;

///
/// Errors
///
/// Language related errors when parsing the grammar for the language
///
#[derive(Debug)]
pub enum EvaluationError {
    /// Indicate handling incorrect language level errors including location and
    /// associated context message
    GuardFileParseError(guard_lang::LangError),

    /// Error when parsing data files JSON or YAML
    ///
    DataParseError(yaml_rust::ScanError),

    /// Any io error that occurs when reading or opening Files
    IoError(std::io::Error),
}

impl std::error::Error for EvaluationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EvaluationError::GuardFileParseError(err) => Some(err),
            EvaluationError::DataParseError(err) => Some(err),
            EvaluationError::IoError(io_error) => Some(io_error)
        }
    }
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::GuardFileParseError(p)  => p.fmt(f),
            EvaluationError::DataParseError(p) => p.fmt(f),
            EvaluationError::IoError(p)  => p.fmt(f)
        }
    }
}

impl From<ScanError> for EvaluationError {
    fn from(err: ScanError) -> Self {
        EvaluationError::DataParseError(err)
    }
}

impl From<LangError> for EvaluationError {
    fn from(err: LangError) -> Self {
        EvaluationError::GuardFileParseError(err)
    }
}

impl From<std::io::Error> for EvaluationError {
    fn from(err: std::io::Error) -> Self {
        EvaluationError::IoError(err)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    BadValue(String, Location),
    Null(Location),
    String(String, Location),
    Bool(bool, Location),
    Int(i64, Location),
    Float(f64, Location),
    Char(char, Location),
    List(Vec<Value>, Location),
    Map(indexmap::IndexMap<(String, Location), Value>, Location),
}

