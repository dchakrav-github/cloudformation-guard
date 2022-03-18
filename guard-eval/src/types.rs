use std::fmt::{Formatter, Debug};
use guard_lang::{Location, RangeType, LangError, Expr};
use yaml_rust::ScanError;
use std::path::PathBuf;

///
/// Errors
///
/// Language related errors when parsing the grammar for the language
///
#[derive(Debug)]
pub enum EvaluationError<'expr> {
    /// Indicate handling incorrect language level errors including location and
    /// associated context message
    GuardFileParseError(guard_lang::LangError),

    /// Error when parsing data files JSON or YAML
    ///
    DataParseError(yaml_rust::ScanError),

    /// Unexpected Expression handling error
    ///
    UnexpectedExpr(String, &'expr Expr),

    /// Unexpected Expression handling error
    ///
    ComputationError(String),

    /// Any io error that occurs when reading or opening Files
    IoError(std::io::Error),
}

impl<'expr> std::error::Error for EvaluationError<'expr> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EvaluationError::GuardFileParseError(err) => Some(err),
            EvaluationError::DataParseError(err) => Some(err),
            EvaluationError::IoError(io_error) => Some(io_error),
            EvaluationError::UnexpectedExpr(..) => None,
            EvaluationError::ComputationError(_) => None,
        }
    }
}

impl<'expr> std::fmt::Display for EvaluationError<'expr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EvaluationError::GuardFileParseError(p)  => std::fmt::Display::fmt(p, f),
            EvaluationError::DataParseError(p) => std::fmt::Display::fmt(p, f),
            EvaluationError::IoError(p)  => std::fmt::Display::fmt(p, f),
            EvaluationError::UnexpectedExpr(msg, expr) => {
                write!(f, "Error {} Location {}, Expr {:?}", msg, expr.get_location(), *expr)
            },
            EvaluationError::ComputationError(msg) => {
                write!(f, "Error {}", msg)
            }
        }
    }
}

impl From<ScanError> for EvaluationError<'_> {
    fn from(err: ScanError) -> Self {
        EvaluationError::DataParseError(err)
    }
}

impl From<LangError> for EvaluationError<'_> {
    fn from(err: LangError) -> Self {
        EvaluationError::GuardFileParseError(err)
    }
}

impl From<std::io::Error> for EvaluationError<'_> {
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
    RangeInt(RangeType<i64>, Location),
    RangeFloat(RangeType<f64>, Location),
    Char(char, Location),
    List(Vec<Value>, Location),
    Map(indexmap::IndexMap<String, Value>, Location),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Status {
    PASS,
    FAIL,
    SKIP
}

#[derive(Debug, Clone)]
pub enum ValueType<'value> {
    DataValue(&'value Value),
    LiteralValue(&'value Expr),
    ComputedValue(Value),
}


pub trait EvalReporter<'value> : Debug {
    fn report_missing_value(
        &mut self,
        until: ValueType<'value>,
        data_file_name: &'value str,
        expr: &'value Expr) -> Result<(), EvaluationError<'value>>;

    fn report_mismatch_value_traversal(
        &mut self,
        until: ValueType<'value>,
        data_file_name: &'value str,
        expr: &'value Expr) -> Result<(), EvaluationError<'value>>;

    fn report_evaluation(
        &mut self,
        value: ValueType<'value>,
        data_file_name: &'value str,
        expr: &'value Expr,
        status: Status) -> Result<(), EvaluationError<'value>>;
}

#[derive(Debug)]
pub struct DataFile {
    pub file: PathBuf,
    pub root: Value,
}

pub type DataFiles = Vec<DataFile>;
