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
	digit0
};
use nom::combinator::{
    map,
	value,
	map_res,
	opt,
	all_consuming,
	cut,
	peek,
	recognize
};
use nom::error::{
    context,
    ErrorKind
};
use nom::number::complete::{
    double,
    recognize_float
};
use nom::sequence::{
    delimited,
	preceded,
	separated_pair,
	tuple,
	pair,
	terminated
};
use crate::Expr;

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

//
// Common helpers
//
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


//
// Parser for the grammar
//
fn strip_comments_space<'a, F, O>(parser: F) -> impl Fn(Span<'a>) -> IResult<Span<'a>, O>
    where F: Fn(Span<'a>) -> IResult<Span<'a>, O>
{
    move |input: Span| {
        let (input, _comments) = zero_or_more_ws_or_comment(input)?;
        let (input, result) = parser(input)?;
        let (input, _comments) = zero_or_more_ws_or_comment(input)?;
        Ok((input, result))
    }
}

fn keyword<'a>(name: &str, input: Span<'a>) -> IResult<Span<'a>, ()> {
    let (input, _keyword) = tag(name)(input)?;
    Ok((input, ()))
}

//
// Language grammar common to value literals and expressions
//

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
    let (input, name) = parse_name(input)?;
    Ok((input, Expr::String(Box::new(StringExpr::new(name, location)))))
}

//
// Value parsing functions
//


//
// INT  ::= (+|-)? (0..9)+
//
fn parse_int_value(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, part) = recognize(
        tuple((
            opt(alt((char('+'), char('-')))),
            digit1
        ))
    )(input)?;
    let result = double(part)?.1 as i64;
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

//
// STRING   ::= '"' (ESC|.)* '"' |
//              '\'' (ESC|.)* '\''
// ESC      ::= '\\' ('|")
//
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

//
// REGEX        ::= '/' (ESC|,)* '/'
// ESC          ::= '\\' /
//
fn parse_regex(input: Span) -> IResult<Span, Expr> {
    delimited(char('/'), parse_regex_inner, char('/'))(input)
}

//
// BOOL         ::= True|TRUE|true|T|False|F|false|FALSE
//

fn parse_bool(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let true_parser = value(true, alt((tag("true"), tag("True"), tag("TRUE"), tag("T"))));
    let false_parser = value(false, alt((tag("false"), tag("False"), tag("FALSE"), tag("F"))));
    let (input, res) = alt((true_parser, false_parser))(input)?;
    Ok((input, Expr::Bool(Box::new(BoolExpr::new(res, location)))))
}

//
//
//

fn parse_float(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let i = input.clone();
    let _ = recognize(
        tuple((
            opt(alt((char('+'), char('-')))),
            alt((
                map(tuple((digit1, char('.'))), |_| ()),
                map(tuple((char('.'), digit1)), |_| ()),
                map(tuple((
                    digit1,
                    alt((char('e'), char('E'))),
                    opt(alt((char('+'), char('-')))),
                    cut(digit1)
                )), |_| ())
            ))
        ))
    )(i)?;
    let (input, value) = double(input)?;
    Ok((input, Expr::Float(Box::new(FloatExpr::new(value, location)))))
}

fn parse_char(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, ch) = anychar(input)?;
    Ok((input, Expr::Char(Box::new(CharExpr::new(ch, location)))))
}

fn range_value<'a, P, O>(parse: P) -> impl Fn(Span<'a>) -> IResult<Span<'a>, (O, O)>
    where P: Fn(Span<'a>) -> IResult<Span<'a>, O>
{
    move |input: Span| {
        let parser = |i| parse(i);
        delimited(
            zero_or_more_ws_or_comment,
            //separated_pair(|i| parse(i), char(','), |i| parse(i)),
            separated_pair(parser, strip_comments_space(char(',')), parser),
            zero_or_more_ws_or_comment,
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
                    range_value(alt((parse_float, parse_int_value))),
                    range_value(alt((parse_float, parse_int_value))),
        )))(input)?;
    let (input, end) = cut(one_of(")]"))(input)?;
    let mut inclusive: u8 = if start == '[' { super::types::LOWER_INCLUSIVE } else { 0u8 };
    inclusive |= if end == ']' { super::types::UPPER_INCLUSIVE } else { 0u8 };
    let value = match (&start_value, &end_value) {
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

        (Expr::Float(_), Expr::Int(_)) |
        (Expr::Int(_), Expr::Float(_)) => {
            let start = match start_value {
                Expr::Int(s) => s.value() as f64,
                Expr::Float(s) => s.value(),
                _ => unreachable!()
            };
            let end = match end_value {
                Expr::Int(s) => s.value() as f64,
                Expr::Float(s) => s.value(),
                _ => unreachable!()
            };
            if start > end {
                return Err(nom::Err::Failure(ParseError::new(
                    location,
                    format!("Range specified is incorrect Start = {}, end = {}",
                            start, end)
                )))
            }
            Expr::RangeFloat(
                Box::new(RangeFloatExpr::new(RangeType {
                    lower: start,
                    upper: end,
                    inclusive
                }, location))
            )
        }


        (_, _) => unreachable!()
    };
    Ok((input, value))
}

fn parse_null(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, _null) = alt((tag("null"), tag("NULL")))(input)?;
    Ok((input, Expr::Null(Box::new(location))))
}

//
// Adding the parser to return scalar values
//
fn parse_scalar_value(input: Span) -> IResult<Span, Expr> {
    alt((
        parse_string,
        parse_regex,
        //
        // is before parse_float as float can also handle 10 as 10.0
        //
        parse_float,
        parse_int_value,
        parse_bool,
    ))(input)
}

fn parse_value_separator(input: Span) -> IResult<Span, ()> {
    empty_value(char(','))(input)
}

fn parse_map_key(input: Span) -> IResult<Span, Expr> {
    strip_comments_space(alt((
        var_name,
        parse_string,
    )))(input)
}

fn parse_start_bracket(input: Span) -> IResult<Span, ()> {
    empty_value(char('{'))(input)
}

fn parse_end_bracket(input: Span) -> IResult<Span, ()> {
    empty_value(char('}'))(input)
}

fn parse_map_key_value_sep(input: Span) -> IResult<Span, ()> {
    empty_value(char(':'))(input)
}

fn parse_map(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, _start_bracket) = parse_start_bracket(input)?;
    let mut map = indexmap::IndexMap::new();
    let mut span = input;
    //
    // empty map
    //
    if let Ok((left, _)) = parse_end_bracket(span) {
        return Ok((left, Expr::Map(Box::new(MapExpr::new(map, location)))));
    }
    loop {
        let (left, (key, value)) = separated_pair(
            parse_map_key,
            parse_map_key_value_sep,
            parse_value)(span)?;

        span = left;
        if let Expr::String(key) = key {
            map.insert(*key, value);
        }

        match parse_value_separator(span) {
            Ok((left, _)) => {
                span = left;
                if let Ok((left, _)) = parse_end_bracket(span) {
                    return Ok((left, Expr::Map(Box::new(MapExpr::new(map, location)))));
                }
            },

            Err(nom::Err::Error(_)) => {
                let (left, _end_bracket) = cut(parse_end_bracket)(span)?;
                return Ok((left, Expr::Map(Box::new(MapExpr::new(map, location)))));
            },

            Err(rest) => return Err(rest)
        }
    }
}

fn empty_value<'a, P, O>(parser: P) -> impl Fn(Span<'a>) -> IResult<Span<'a>, ()>
where
    P: Fn(Span<'a>) -> IResult<Span<'a>, O>
{
    move |input: Span| {
        let (input, _) = zero_or_more_ws_or_comment(input)?;
        let (input, _ign) = parser(input)?;
        zero_or_more_ws_or_comment(input).map(|(i, _)| (i, ()))
    }
}

fn parse_start_braces(input: Span) -> IResult<Span, ()> {
    empty_value(char('['))(input)
}

fn parse_end_braces(input: Span) -> IResult<Span, ()> {
    empty_value(char(']'))(input)
}

fn parse_array(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, _start) = parse_start_braces(input)?;
    let mut collection = Vec::new();
    let mut span = input;
    //
    // empty map
    //
    if let Ok((left, _)) = parse_end_braces(span) {
        return Ok((left, Expr::Array(Box::new(ArrayExpr::new(collection, location)))))
    }
    loop {
        let (left, value) = parse_value(span)?;
        collection.push(value);
        span = left;
        match parse_value_separator(span) {
            Ok((left, _)) => {
                span = left;
                if let Ok((left, _)) = parse_end_braces(span) {
                    return Ok((left, Expr::Array(Box::new(ArrayExpr::new(collection, location)))))
                }
            },

            Err(nom::Err::Error(_)) => {
                let (left, _end) = cut(parse_end_braces)(span)?;
                return Ok((left, Expr::Array(Box::new(ArrayExpr::new(collection, location)))))
            },

            Err(rest) => return Err(rest)
        }
    }
}

fn parse_value(input: Span) -> IResult<Span, Expr> {
    strip_comments_space(alt((
        parse_scalar_value,
        parse_map,
        parse_array,
        parse_null
    )))(input)
}


fn parse_variable_reference(input: Span) -> IResult<Span, Expr> {
    map(
        preceded(
        char('%'),
        var_name),
        |s| match s {
            Expr::String(expr) => Expr::VariableReference(expr),
            _ => unreachable!()
        }
    )(input)
}

fn parse_all_reference(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, _) = tag("*")(input)?;
    Ok((input, Expr::String(Box::new(StringExpr::new("*".to_string(), location)))))
}

fn parse_variable(input: Span) -> IResult<Span, Expr> {
    map(
        var_name,
        |s| match s {
            Expr::String(expr) => Expr::Variable(expr),
            _ => unreachable!()
        }
    )(input)
}

fn parse_property_name(input: Span) -> IResult<Span, Expr> {
    strip_comments_space(var_name)(input)
}

fn parse_block_inner_expr(input: Span) -> IResult<Span, BlockExpr> {
    todo!()
}

fn parse_query_simple_segment(input: Span) -> IResult<Span, Expr> {
    strip_comments_space(
        alt((
            var_name,
            parse_variable_reference,
            parse_all_reference,
            parse_string,
        ))
    )(input)
}

fn parse_var_block(input: Span) -> IResult<Span, (Expr, Option<Expr>)> {
    let location = Location::new(input.location_line(), input.get_column());
    match parse_name(input) {
        Err(nom::Err::Error(_)) => {
            map(
                parse_block_inner_expr,
                |b| (Expr::Filter(Box::new(b)), None)
            )(input)
        },


        Ok((input, variable)) => {
            let (input, blk) = opt(
                strip_comments_space(preceded(char('|'), parse_block_inner_expr)))(input)?;
            match blk {
                Some(blk) =>
                    Ok((input,
                       (Expr::Variable(Box::new(StringExpr::new(variable, location))),
                        Some(Expr::Filter(Box::new(blk))))
                    )),
                None =>
                    Ok((input,
                        (Expr::Variable(Box::new(StringExpr::new(variable, location))), None)
                       )),
            }
        }

        Err(e) => return Err(e)
    }
}

fn parse_query_filter_segment(input: Span) -> IResult<Span, (Expr, Option<Expr>)> {
    let (input, _start_braces) = parse_start_braces(input)?;
    let (input, expr) = strip_comments_space(alt( (
        map(alt(( parse_string, parse_int_value, parse_all_reference, parse_variable_reference)), |e| (e, None)),
        parse_var_block
    )))(input)?;
    let (input, _end_braces) = cut(parse_end_braces)(input)?;
    Ok((input, expr))
}

fn parse_assignment_query(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, start_query_expr) =  strip_comments_space(
        alt((var_name, parse_variable_reference))
    )(input)?;
    todo!()
}

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