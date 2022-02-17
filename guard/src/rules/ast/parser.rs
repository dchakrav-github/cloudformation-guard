use crate::rules::parser::{Span, zero_or_more_ws_or_comment, IResult};
use crate::rules::ast::exprs::{Expr, LetExpr, Location, StringExpr, IntExpr, BoolExpr};
use nom::sequence::delimited;
use nom::combinator::value;
use nom::bytes::complete::*;
use nom::combinator::*;
use nom::sequence::*;
use nom::branch::*;
use nom::character::complete::*;
use nom::Slice;
use yaml_rust::yaml::Yaml::Boolean;

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

fn keyword<'a>(name: &str, input: Span<'a>) -> IResult<'a, Span<'a>, ()> {
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

fn parse_bool(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let true_parser = value(true, alt((tag("true"), tag("True"))));
    let false_parser = value(false, alt((tag("false"), tag("False"))));
    let (input, res) = alt((true_parser, false_parser))(input)?;
    Ok((input, Expr::Bool(Box::new(BoolExpr::new(res, location)))))
}

//fn parse_float(input: Span) -> IResult<Span, Value> {
//    let whole = digit1(input.clone())?;
//    let fraction = opt(preceded(char('.'), digit1))(whole.0)?;
//    let exponent = opt(tuple((one_of("eE"), one_of("+-"), digit1)))(fraction.0)?;
//    if (fraction.1).is_some() || (exponent.1).is_some() {
//        let r = double(input)?;
//        return Ok((r.0, Value::Float(r.1)));
//    }
//    Err(nom::Err::Error(ParserError {
//        context: format!("Could not parse floating number"),
//        kind: nom::error::ErrorKind::Float,
//        span: input
//    }))
//}
//
//fn parse_regex_inner(input: Span) -> IResult<Span, Value> {
//    let mut regex = String::new();
//    let parser = is_not("/");
//    let mut span = input;
//    loop {
//        let (remainder, content) = parser(span)?;
//        let fragment = *content.fragment();
//        //
//        // if the last one has an escape, then we need to continue
//        //
//        if fragment.len() > 0 && fragment.ends_with("\\") {
//            regex.push_str(&fragment[0..fragment.len()-1]);
//            regex.push('/');
//            span = remainder.take_split(1).0;
//            continue;
//        }
//        regex.push_str(fragment);
//        return Ok((remainder, Value::Regex(regex)));
//    }
//}
//
//fn parse_regex(input: Span) -> IResult<Span, Value> {
//    delimited(char('/'), parse_regex_inner, char('/'))(input)
//}
//
//fn parse_char(input: Span) -> IResult<Span, Value> {
//    map(anychar, Value::Char)(input)
//}
//
//fn range_value(input: Span) -> IResult<Span, Value> {
//    delimited(
//        space0,
//        alt((parse_float, parse_int_value, parse_char)),
//        space0,
//    )(input)
//}
//
//fn parse_range(input: Span) -> IResult<Span, Value> {
//    let parsed = preceded(
//        char('r'),
//        tuple((
//            one_of("(["),
//            separated_pair(range_value, char(','), range_value),
//            one_of(")]"),
//        )),
//    )(input)?;
//    let (open, (start, end), close) = parsed.1;
//    let mut inclusive: u8 = if open == '[' { super::types::LOWER_INCLUSIVE } else { 0u8 };
//    inclusive |= if close == ']' { super::types::UPPER_INCLUSIVE } else { 0u8 };
//    let val = match (start, end) {
//        (Value::Int(s), Value::Int(e)) => Value::RangeInt(RangeType {
//            upper: e,
//            lower: s,
//            inclusive,
//        }),
//
//        (Value::Float(s), Value::Float(e)) => Value::RangeFloat(RangeType {
//            upper: e,
//            lower: s,
//            inclusive,
//        }),
//
//        (Value::Char(s), Value::Char(e)) => Value::RangeChar(RangeType {
//            upper: e,
//            lower: s,
//            inclusive,
//        }),
//
//        _ => return Err(nom::Err::Failure(ParserError {
//            span: parsed.0,
//            kind: nom::error::ErrorKind::IsNot,
//            context: format!("Could not parse range")
//        }))
//    };
//    Ok((parsed.0, val))
//}
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