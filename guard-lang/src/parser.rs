use super::{Span, Location, ParseError, RangeType};
use super::exprs::*;

use nom::{Slice, InputTake};

use nom::multi::{
    many0,
    many1,
    fold_many1,
    separated_list,
    separated_nonempty_list
};

use nom::branch::alt;
use nom::bytes::complete::{
    tag,
    take_till,
    is_not,
    take_while,
    take_while1
};
use nom::character::complete::{
    char,
    anychar,
    multispace0,
    multispace1,
    space0,
    space1,
    digit1,
    one_of,
    alpha1,
    newline,
};
use nom::combinator::{
    map,
    value,
    map_res,
    opt,
    all_consuming,
    cut,
    peek
};
use nom::error::{context, ErrorKind};
use nom::number::complete::double;
use nom::sequence::{
    delimited,
	preceded,
	separated_pair,
	tuple,
	pair,
	terminated
};

type IResult<I, O> = nom::IResult<I, O, ParseError>;

impl<'a> nom::error::ParseError<Span<'a>> for ParseError {
    fn from_error_kind(input: Span<'a>, kind: nom::error::ErrorKind) -> Self {
        ParseError::new(
            Location::new(input.location_line(), input.get_column()),
            format!("File {}, error {}", input.extra, kind.description())
        )
    }

    fn append(_input: Span<'a>, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn add_context(_input: Span<'a>, ctx: &'static str, other: Self) -> Self {
        let message = format!("{} {}", other.get_message(), ctx);
        other.message(message)
    }
}

fn comment2(input: Span) -> IResult<Span, Span> {
    delimited(char('#'), take_till(|c| c == '\n'), char('\n'))(input)
}
//
// This function extracts either white-space-CRLF or a comment
// and discards them
//
// (LWSP / comment)
//
// Expected error codes: (remember alt returns the error from the last one)
//    nom::error::ErrorKind::Char => if the comment does not start with '#'
//
fn white_space_or_comment(input: Span) -> IResult<Span, ()> {
    value((), alt((
        multispace1,
        comment2
    )))(input)
}

//
// This provides extract for 1*(LWSP / commment). It does not indicate
// failure when this isn't the case. Consumers of this combinator must use
// cut or handle it as a failure if that is the right outcome
//
fn one_or_more_ws_or_comment(input: Span) -> IResult<Span, ()> {
    value((), many1(white_space_or_comment))(input)
}

//
// This provides extract for *(LWSP / comment), same as above but this one never
// errors out
//
fn zero_or_more_ws_or_comment(input: Span) -> IResult<Span, ()> {
    value((), many0(white_space_or_comment))(input)
}


///
/// Parser grammar
///
///
///
fn strip_comments_space<F, O>(parser: F) -> impl Fn(Span) -> IResult<Span, O>
    where F: Fn(Span) -> IResult<Span, O>
{
    move |input| {
        let (input, _comments) = zero_or_more_ws_or_comment(input)?;
        let (input, result) = parser(input)?;
        let (input, _comments) = zero_or_more_ws_or_comment(input)?;
        Ok((input, result))
    }
}

fn parser_return_void<F, O>(parser: F) -> impl Fn(Span) -> IResult<Span, ()>
    where F: Fn(Span) -> IResult<Span, O>
{
    move |input| {
        let (input, _result) = strip_comments_space(|i| parser(i))(input)?;
        Ok((input, ()))
    }
}

fn keyword<'a>(name: &str, input: Span<'a>) -> IResult<Span<'a>, ()> {
    let (input, _keyword) = tag(name)(input)?;
    Ok((input, ()))
}

fn parse_name(input: Span) -> IResult<Span, String> {
    map(
        tuple((
            alpha1,
            take_while(|ch: char| ch.is_alphanumeric() || ch == '_')
        )),
        |mapped: (Span, Span)| {
            let mut first = mapped.0.fragment().to_string();
            first.push_str(*mapped.1.fragment());
            first
        }
    )(input)
}

fn var_name(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, name) = strip_comments_space(parse_name)(input)?;
    Ok((input, Expr::String(Box::new(StringExpr::new(name, location)))))
}

fn parse_int_value(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let negative = map_res(preceded(tag("-"), digit1), |s: Span| {
        s.fragment().parse::<i64>().map(|i| -1 * i)
    });
    let positive = map_res(digit1, |s: Span| {
        s.fragment().parse::<i64>()
    });
    let (input, result) = alt((positive, negative))(input)?;
    Ok((input, Expr::Int(Box::new(IntExpr::new(result, location)))))
}

fn parse_string_inner(ch: char) -> impl Fn(Span) -> IResult<Span, String> {
    move |input: Span| {
        let mut completed = String::new();
        let (input, _begin) = char(ch)(input)?;
        let mut span = input;
        loop {
            let (remainder, upto) = take_while(|c| c != ch)(span)?;
            let frag = *upto.fragment();
            if frag.ends_with('\\') {
                completed.push_str(&frag[0..frag.len()-1]);
                completed.push(ch);
                span = remainder.slice(1..);
                continue;
            }
            completed.push_str(frag);
            let (remainder, _end) = cut(char(ch))(remainder)?;
            return Ok((remainder, completed))
        }
    }
}

fn parse_string(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, res) = alt((
        parse_string_inner('\''),
        parse_string_inner('\"')
    ))(input)?;
    Ok((input, Expr::String(Box::new(StringExpr::new(res, location)))))
}

fn parse_regex_inner(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let mut regex = String::new();
    let parser = is_not("/");
    let mut span = input;
    loop {
        let (remainder, content) = parser(span)?;
        let fragment = *content.fragment();
        //
        // if the last one has an escape, then we need to continue
        //
        if fragment.len() > 0 && fragment.ends_with("\\") {
            regex.push_str(&fragment[0..fragment.len()-1]);
            regex.push('/');
            span = remainder.take_split(1).0;
            continue;
        }
        regex.push_str(fragment);
        return Ok((remainder, Expr::Regex(Box::new(RegexExpr::new(regex, location)))))
    }
}

fn parse_regex(input: Span) -> IResult<Span, Expr> {
    delimited(char('/'), parse_regex_inner, char('/'))(input)
}

fn parse_bool(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let true_parser = value(true, alt((tag("true"), tag("True"), tag("TRUE"), tag("T"))));
    let false_parser = value(false, alt((tag("false"), tag("False"), tag("FALSE"), tag("F"))));
    let (input, res) = alt((true_parser, false_parser))(input)?;
    Ok((input, Expr::Bool(Box::new(BoolExpr::new(res, location)))))
}

fn parse_float(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, value) = double(input)?;
    Ok((input, Expr::Float(Box::new(FloatExpr::new(value, location)))))
}

fn parse_char(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, ch) = anychar(input)?;
    Ok((input, Expr::Char(Box::new(CharExpr::new(ch, location)))))
}

fn range_value<P, O>(parse: P) -> impl Fn(Span) -> IResult<Span, (O, O)>
    where P: Fn(Span) -> IResult<Span, O>
{
    move |input: Span| {
        let parser = |i| parse(i);
        delimited(
            multispace0,
            //separated_pair(|i| parse(i), char(','), |i| parse(i)),
            separated_pair(parser, char(','), parser),
            multispace0,
        )(input)
    }
}

fn parse_range(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, _range) = char('r')(input)?;
    let (input, start) = one_of("([")(input)?;
    let (input, (start_value, end_value)) =
        context(
            "expecting range of integers or floats. E,g, r[10, 20] or r(10.2, 12.5]",
            alt((
            range_value(strip_comments_space(parse_int_value)),
            range_value(strip_comments_space(parse_float))
        )))(input)?;
    let (input, end) = cut(one_of(")]"))(input)?;
    let mut inclusive: u8 = if start == '[' { super::types::LOWER_INCLUSIVE } else { 0u8 };
    inclusive |= if end == ']' { super::types::UPPER_INCLUSIVE } else { 0u8 };
    let value = match (start_value, end_value) {
        (Expr::Int(start), Expr::Int(end)) => {
            if start.value() > end.value() {
                return Err(nom::Err::Failure(ParseError::new(
                    location,
                    format!("Range specified is incorrect Start = {}, end = {}",
                            start.value(), end.value())
                )))
            }
            Expr::RangeInt(
                Box::new(RangeIntExpr::new(RangeType {
                    lower: start.value(),
                    upper: end.value(),
                    inclusive
                }, location))
            )
        },

        (Expr::Float(start), Expr::Float(end)) => {
            if start.value() > end.value() {
                return Err(nom::Err::Failure(ParseError::new(
                    location,
                    format!("Range specified is incorrect Start = {}, end = {}",
                            start.value(), end.value())
                )))
            }
            Expr::RangeFloat(
                Box::new(RangeFloatExpr::new(RangeType {
                    lower: start.value(),
                    upper: end.value(),
                    inclusive
                }, location))
            )
        },

        (_, _) => unreachable!()
    };
    Ok((input, value))
}

//
////
//// Adding the parser to return scalar values
////
//fn parse_scalar_value(input: Span) -> IResult<Span, Value> {
//    //
//    // IMP: order does matter
//    // parse_float is before parse_int. the later can parse only the whole part of the float
//    // to match.
//    alt((
//        parse_string,
//        parse_float,
//        parse_int_value,
//        parse_bool,
//        parse_regex,
//    ))(input)
//}
//

//fn parse_let(input: Span) -> IResult<Span, Expr> {
//    let location = Location::new(
//        input.location_line() as usize,
//        column: input.get_utf8_column());
//    map(
//        tuple(
//            (
//                parser_return_void(tag("let")),
//                parse_name,
//                parser_return_void(tag("=")),
//            ),
//            ),
//        |_, name, _, expr| {
//            Expr::Let(LetExpr::new( name, expr, location))
//        })(input)
//}
#[cfg(test)]
#[path = "parser_tests.rs"]
mod parser_tests;