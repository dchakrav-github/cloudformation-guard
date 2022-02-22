use nom_locate::LocatedSpan;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

/// Present the Location of a given expression inside a Guard file
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Location {
    row: usize,
    column: usize
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line = {}, Column = {}", self.row, self.column)
    }
}

/// Represent a Range type for Ordered types like int, floats and chars
/// Ranges can be repesented as inclusive or exclusive of the range.
/// Inclusive is represented simialar to the mathematical model using '['
/// Exclusive uses '('.
///
/// e.g. r(0.100] represents a range of numbers from 0 (excluded) to 100
/// (included)
///
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct RangeType<T: PartialOrd> {
    pub upper: T,
    pub lower: T,
    pub inclusive: u8,
}

/// Span represent a span of content of a Guard File when processing it
/// It tracks the location, line and colume number in the bytes stream
/// for the file and used to create the [Location] instances
pub(crate) type Span<'a> = LocatedSpan<&'a str, &'a str>;

///
/// Errors
///
/// Language related errors when parsing the grammar for the language
///
#[derive(Debug)]
pub enum LangError {
    /// Indicate handling incorrect language level errors including location and
    /// associated context message
    ParseError(ParseError),

    /// Any io error that occurs when reading or opening Files
    IoError(std::io::Error),
}

impl std::error::Error for LangError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LangError::ParseError(_) => None,
            LangError::IoError(io_error) => Some(io_error)
        }
    }
}

impl std::fmt::Display for LangError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LangError::ParseError(p)  => p.fmt(f),
            LangError::IoError(p)  => p.fmt(f)
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    /// Represents any Parse Error that happens when handling Guard Language
    ///
    message: String,
    location: Location,
}

impl ParseError {
    pub fn new(location: Location, message: String) -> Self {
        ParseError { location, message }
    }

    pub(crate) fn message(mut self, msg: String) -> Self {
        self.message = msg;
        self
    }

    pub(crate) fn location(mut self, location: Location) -> Self {
        self.location = location;
        self
    }

    #[inline]
    pub(crate) fn get_message(&self) -> &str { &self.message }

    #[inline]
    pub(crate) fn get_location(&self) -> &Location { &self.location }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error parsing at {}. Reason {}", self.location, self.message)
    }
}

pub type Result<R> = std::result::Result<R, LangError>;


pub trait WithinRange<RHS: PartialOrd = Self> {
    fn is_within(&self, range: &RangeType<RHS>) -> bool;
}

impl WithinRange for i64 {
    fn is_within(&self, range: &RangeType<i64>) -> bool {
        is_within(range, self)
    }
}

impl WithinRange for f64 {
    fn is_within(&self, range: &RangeType<f64>) -> bool {
        is_within(range, self)
    }
}

impl WithinRange for char {
    fn is_within(&self, range: &RangeType<char>) -> bool {
        is_within(range, self)
    }
}

//impl WithinRange for

fn is_within<T: PartialOrd>(range: &RangeType<T>, other: &T) -> bool {
    let lower = if (range.inclusive & LOWER_INCLUSIVE) > 0 {
        range.lower.le(other)
    } else {
        range.lower.lt(other)
    };
    let upper = if (range.inclusive & UPPER_INCLUSIVE) > 0 {
        range.upper.ge(other)
    } else {
        range.upper.gt(other)
    };
    lower && upper
}

pub const LOWER_INCLUSIVE: u8 = 0x01;
pub const UPPER_INCLUSIVE: u8 = 0x01 << 1;

impl Location {
    #[inline]
    pub fn new(row: u32, column: usize) -> Location {
        Location { row: row as usize, column }
    }
    #[inline]
    pub fn row(&self) -> usize { self.row }
    #[inline]
    pub fn column(&self) -> usize { self.column }
}

pub fn from_str2(in_str: &str) -> Span {
    Span::new_extra(in_str, "")
}

