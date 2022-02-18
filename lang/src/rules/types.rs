use serde::{Serialize, Deserialize};
use nom_locate::LocatedSpan;

//
//    .X > 10
//    .X <= 20
//
//    .X in r(10, 20]
//    .X in r(10, 20)
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct RangeType<T: PartialOrd> {
    pub upper: T,
    pub lower: T,
    pub inclusive: u8,
}

pub type Span<'a> = LocatedSpan<&'a str, &'a str>;

pub type IResult<'a, I, O> = nom::IResult<I, O, ParserError<'a>>;

pub type Result<R> = std::result::Result<R,super::errors::Error>;


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

pub fn from_str2(in_str: &str) -> Span {
    Span::new_extra(in_str, "");
}

