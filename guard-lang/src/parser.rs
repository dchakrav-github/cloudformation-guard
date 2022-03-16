use super::{Span, Location, ParseError, RangeType};
use super::exprs::*;

use nom::{Slice, InputTake, FindSubstring, InputLength};

use nom::multi::{many0, many1};

use nom::branch::alt;
use nom::bytes::complete::{
    tag,
    take_till,
    is_not,
    take_while,
    take,
};
use nom::character::complete::{char, anychar, multispace1, digit1, one_of, alpha1, newline, space0};
use nom::combinator::{map, value, opt, cut, recognize, peek, };
use nom::error::{
    context,
};
use nom::number::complete::{
    double,
};
use nom::sequence::{delimited, preceded, separated_pair, tuple, terminated};
use crate::Expr;
use std::collections::VecDeque;

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
}

impl<'a> nom::error::ContextError<Span<'a>> for ParseError {

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
// This provides extract for *(LWSP / comment), same as above but this one never
// errors out
//
fn zero_or_more_ws_or_comment(input: Span) -> IResult<Span, ()> {
    value((), many0(white_space_or_comment))(input)
}

fn one_or_more_ws_or_comment(input: Span) -> IResult<Span, ()> {
    value((), many1(white_space_or_comment))(input)
}


//
// Parser for the grammar
//
fn strip_comments_space<'a, F, O>(mut parser: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O>
    where F: FnMut(Span<'a>) -> IResult<Span<'a>, O>
{
    move |input: Span| {
        let (input, _comments) = zero_or_more_ws_or_comment(input)?;
        let (input, result) = parser(input)?;
        let (input, _comments) = zero_or_more_ws_or_comment(input)?;
        Ok((input, result))
    }
}

fn strip_comments_trailing_space<'a, F, O>(mut parser: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O>
    where F: FnMut(Span<'a>) -> IResult<Span<'a>, O>
{
    move |input: Span| {
        let (input, _comments) = zero_or_more_ws_or_comment(input)?;
        let (input, result) = parser(input)?;
        let (input, _comments) = one_or_more_ws_or_comment(input)?;
        Ok((input, result))
    }
}

fn strip_comments_space1<'a, F, O>(mut parser: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O>
    where F: FnMut(Span<'a>) -> IResult<Span<'a>, O>
{
    move |input: Span| {
        let (input, _comments) = one_or_more_ws_or_comment(input)?;
        let (input, result) = parser(input)?;
        let (input, _comments) = one_or_more_ws_or_comment(input)?;
        Ok((input, result))
    }
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
    let true_parser = value(true, alt((tag("true"), tag("True"), tag("TRUE"),)));
    let false_parser = value(false, alt((tag("false"), tag("False"), tag("FALSE"),)));
    let (input, res) = alt((true_parser, false_parser))(input)?;
    Ok((input, Expr::Bool(Box::new(BoolExpr::new(res, location)))))
}

//
// FLOAT        ::= (+|-) ( INT EXP | DOTTED (EXP)? )
// DOTTED       ::= digit1 '.' digit1 |
//                  '.' digit1 |
//                  digit '.'
// EXPT         ::= (e|E) (+|-) digit1
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

//
// CHAR         ::= "'" char "'"
//
fn parse_char(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, ch) = anychar(input)?;
    Ok((input, Expr::Char(Box::new(CharExpr::new(ch, location)))))
}

fn range_value<'a, P, O>(mut parse: P) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, (O, O)>
    where P: FnMut(Span<'a>) -> IResult<Span<'a>, O>
{
    move |input: Span| {
        let (input, _) = zero_or_more_ws_or_comment(input)?;
        let (input, lower) = parse(input)?;
        let (input, _) = strip_comments_space(char(','))(input)?;
        let (input, higher) = parse(input)?;
        Ok((input, (lower, higher)))
    }
}

//
// RANGE        ::= 'r' ('(' | '[') (COM | SPC)* (INT | FLOAT) (COM | SPC)* ( ')' | ']' )
//
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

//
// MAP          ::= '{' (COM|SPACE)* KEY (COM|SPACE)* SEPARATOR (COM|SPACE)* VALUE '}'
// SEPARATOR    ::= ':'
// KEY          ::= PROPERTY_NAME | STRING
//
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
            let_query_or_value)(span)?;

        span = left;
        if let Expr::String(key) = key {
            map.insert((*key).value, value);
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
        value((), |i| parser(i))(input)
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
        let (left, value) = cut(let_query_or_value)(span)?;
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

//
// VALUE        ::= SCALAR | MAP | ARRAY | NULL
//
pub(crate) fn parse_value(input: Span) -> IResult<Span, Expr> {
    strip_comments_space(alt((
        parse_scalar_value,
        parse_map,
        parse_array,
        parse_null,
        parse_range,
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

fn parse_query_simple_segment(input: Span) -> IResult<Span, Expr> {
    strip_comments_space(
        alt((
            var_name,
            parse_variable_reference,
            parse_all_reference,
            parse_string,
            parse_int_value
        ))
    )(input)
}

fn var_block_name(input: Span) -> IResult<Span, (String, bool)> {
    tuple((
        parse_name,
        alt((
            map(strip_comments_space(char('|')), |_| true),
            map(peek(parse_end_braces), |_| false)
        ))))(input)
}

fn parse_var_block(input: Span) -> IResult<Span, (Expr, Option<Expr>)> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, var_block) = opt(var_block_name)(input)?;
    match var_block {
        None =>
            map(
                parse_block_inner_expr,
                |b| (Expr::Filter(Box::new(b)), None)
            )(input),

        Some((variable, false)) => Ok((
            input,
            (Expr::Variable(Box::new(StringExpr::new(variable, location))), None)
        )),

        Some((variable, true)) => {
            let (input, blk) = map(
                parse_block_inner_expr,
                |b| Expr::Filter(Box::new(b))
            )(input)?;
            Ok((input,
                (Expr::Variable(Box::new(StringExpr::new(variable, location))),
                 Some(blk))))
        }
    }
}

fn parse_query_filter_segment<'a, F>(mut filter: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, (Expr, Option<Expr>)>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, (Expr, Option<Expr>)>
{
    move |input: Span| {
        let (input, _start_braces) = parse_start_braces(input)?;
        let (input, expr) = strip_comments_space(alt((
            map(alt((parse_string, parse_int_value, parse_all_reference, parse_variable_reference)), |e| (e, None)),
            |i| filter(i),
        )))(input)?;
        let (input, _end_braces) = cut(parse_end_braces)(input)?;
        Ok((input, expr))
    }
}

fn query_segment<'a, P, F>(parser: P, filter: F, input: Span<'a>, segements: &mut Vec<Expr>) -> IResult<Span<'a>, ()>
where
    P: FnMut(Span<'a>) -> IResult<Span<'a>, Expr>,
    F: FnMut(Span<'a>) -> IResult<Span<'a>, (Expr, Option<Expr>)>,
{
    let (input, start) = strip_comments_space(parser)(input)?;
    let (input, filter) = opt(parse_query_filter_segment(filter))(input)?;
    segements.push(start);
    if let Some(filter) = filter {
        segements.push(filter.0);
        if let Some(expr) = filter.1 {
            segements.push(expr);
        }
    }
    Ok((input, ()))
}

fn parse_query<'a, F>(mut filter: F, input: Span<'a>) -> IResult<Span<'a>, Expr>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, (Expr, Option<Expr>)>
{
    let location = Location::new(input.location_line(), input.get_column());
    let mut segments = Vec::with_capacity(4);
    let (input, _) = query_segment(
        alt((var_name, parse_variable_reference, parse_string)),
        |i| filter(i),
        input,
        &mut segments
    )?;
    let mut next = input;
    loop {
        match query_segment(
            preceded(
                char('.'),
                parse_query_simple_segment,
            ),
            |i| filter(i),
            next,
            &mut segments) {
            Ok((input, _)) => {
                next = input;
            },

            Err(nom::Err::Error(_)) => {
                return Ok((next, Expr::Select(Box::new(QueryExpr::new(segments, location)))))
            },

            Err(e) => return Err(e)
        }
    }
}

fn parse_select(input: Span) -> IResult<Span, Expr> {
    parse_query(parse_var_block, input)
}

fn parse_assignment_query(input: Span) -> IResult<Span, Expr> {
    parse_query(
        map(parse_block_inner_expr, |e| (Expr::Filter(Box::new(e)), None)),
        input
    )
}

fn let_query_or_value(input: Span) -> IResult<Span, Expr> {
    strip_comments_space(
        alt((
            parse_value,
            keys_or_indices_unary_expr(parse_assignment_query),
            parse_assignment_query,
        ))
    )(input)
}

fn parse_let_expr(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, (_let, variable)) = strip_comments_space(tuple((
        terminated(tag("let"), one_or_more_ws_or_comment),
        strip_comments_space(parse_name)
    )))(input)?;
    let (input, _assign_sign) = cut(strip_comments_space(char('=')))(input)?;
    let (input, assignment) = let_query_or_value(input)?;
    let (input, or_assignment) =
        opt(preceded(
            strip_comments_space(or_operator), let_query_or_value))(input)?;
    match or_assignment {
        Some(value) => {
            Ok((input, Expr::Let(Box::new(LetExpr::new(variable, Expr::BinaryOperation(Box::new(
                BinaryExpr::new(BinaryOperator::Or, assignment, value, location.clone()))), location)))))
        },
        None =>{
            Ok((input, Expr::Let(Box::new(LetExpr::new(variable, assignment, location)))))
        }
    }
}

fn query_or_value(input: Span) -> IResult<Span, Expr> {
    strip_comments_space(
        alt((
            parse_value,
            parse_select,
        ))
    )(input)
}

fn binary_cmp_operator(input: Span) -> IResult<Span, BinaryOperator> {
    alt( (
        strip_comments_space(
            alt((
                value(BinaryOperator::Equals, tag("==")),
                value(BinaryOperator::NotEquals, tag("!=")),
                // Order matter, we first check specific ">=" before ">" to prevent partial matches
                value(BinaryOperator::GreaterThanEquals, tag(">=")),
                value(BinaryOperator::Greater, preceded(tag(">"), peek(nom::combinator::not(char('>'))))),
                value(BinaryOperator::LesserThanEquals, tag("<=")),
                value(BinaryOperator::Lesser, preceded(tag("<"), peek(nom::combinator::not(char('<')))))
            ))),
        strip_comments_trailing_space(
            value(BinaryOperator::In, alt((tag("in"), tag("IN"))))
        ),
    ))(input)
}

fn parse_binary_bool_expr(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, (lhs, operator, rhs)) = tuple((
        query_or_value,
        binary_cmp_operator,
        cut(query_or_value)
    ))(input)?;
    let (input, message_doc) = strip_comments_space(opt(alt((here_doc, message_doc))))(input)?;
    Ok((input, Expr::BinaryOperation(Box::new(BinaryExpr::new_with_msg(operator, lhs, rhs, location, message_doc)))))
}

fn not(input: Span) -> IResult<Span, UnaryOperator> {
    value(UnaryOperator::Not,
          alt((
              strip_comments_trailing_space(alt((tag("not"), tag("NOT")))),
               strip_comments_space(tag("!")))
          )
    )(input)
}

fn here_doc(input: Span) -> IResult<Span, String> {
    let (input, (_here_start, identity, _newline)) = tuple(
        (tag("<<"), parse_name, newline)
    )(input)?;
    let (input, message) = match input.find_substring(&identity) {
        None => return Err(nom::Err::Failure(ParseError::new(
            Location::new(input.location_line(), input.get_column()),
            format!("Can not find HEREDOC ending for {}", identity),
        ))),
        Some(v) => {
            let split = input.take_split(v);
            let (input, _identity_part) = take(identity.len())(split.0)?;
            (input, (*split.1.fragment()).to_string())
        }
    };
    let (input, _space) = cut(one_or_more_ws_or_comment)(input)?;
    Ok((input, message))
}

fn message_doc(input: Span) -> IResult<Span, String> {
    let(input, _start_tag) = tag("<<")(input)?;
    let (input, message) = match input.find_substring(">>") {
        None => return Err(nom::Err::Failure(ParseError::new(
            Location::new(input.location_line(), input.get_column()),
            format!("Can not find ending for message doc {}", ">>"),
        ))),
        Some(v) => {
            let split = input.take_split(v);
            (split.0, (*split.1.fragment()).to_string())
        }
    };
    let (input, _end_tag) = cut(tag(">>"))(input)?;
    Ok((input, message))
}

fn unary_operator<'a, P, O3>(
    mut parser: P, op: UnaryOperator, not_op: UnaryOperator) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, UnaryOperator>
where
    P: FnMut(Span<'a>) -> IResult<Span<'a>, O3>
{
    move |input: Span| {
        let (input, not) = opt(not)(input)?;
        let (input, value) = value(op, |i| parser(i))(input)?;
        Ok((input, not.map_or(value, |_| not_op)))
    }
}

fn unary_cmp_operator(input: Span) -> IResult<Span, UnaryOperator> {
    strip_comments_space(alt((
        unary_operator(
        alt((tag("EMPTY"), tag("empty"))),
        UnaryOperator::Empty, UnaryOperator::NotEmpty),

        unary_operator(
            alt((tag("EXISTS"), tag("exists"))),
            UnaryOperator::Exists, UnaryOperator::NotExists),

        unary_operator(
            alt((tag("is_string"), tag("IS_STRING"))),
            UnaryOperator::IsString, UnaryOperator::IsNotString),

        unary_operator(
            alt((tag("is_list"), tag("IS_LIST"))),
            UnaryOperator::IsList, UnaryOperator::IsNotList),

        unary_operator(
            alt((tag("is_int"), tag("IS_INT"))),
            UnaryOperator::IsInt, UnaryOperator::IsNotInt),

        unary_operator(
            alt((tag("is_float"), tag("IS_FLOAT"))),
            UnaryOperator::IsFloat, UnaryOperator::IsNotFloat),

        unary_operator(
            alt((tag("is_float"), tag("IS_FLOAT"))),
            UnaryOperator::IsFloat, UnaryOperator::IsNotFloat),

        unary_operator(
            alt((tag("is_bool"), tag("IS_BOOL"))),
            UnaryOperator::IsBool, UnaryOperator::IsNotBool),

        unary_operator(
            alt((tag("is_regex"), tag("IS_REGEX"))),
            UnaryOperator::IsRegex, UnaryOperator::IsNotRegex),

    )))(input)
}

fn keys_or_indices(input: Span) -> IResult<Span, UnaryOperator> {
    alt((
        value(UnaryOperator::Keys, strip_comments_trailing_space(alt((tag("keys"), tag("KEYS"))))),
        value(UnaryOperator::Indices, strip_comments_trailing_space(alt((tag("indices"), tag("INDICES"))))),
    ))(input)
}

fn keys_or_indices_unary_expr<P>(mut parser: P) -> impl FnMut(Span) -> IResult<Span, Expr>
where
    P: FnMut(Span) -> IResult<Span, Expr>
{
    move |input: Span| {
        let location = Location::new(input.location_line(), input.get_column());
        let (input, keys_or_indices) = keys_or_indices(input)?;
        let (input, expr) = parser(input)?;
        Ok((input, Expr::UnaryOperation(Box::new(UnaryExpr::new(keys_or_indices, expr, location)))))
    }
}

fn unary_post_bool_expr(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, (expr, operator)) = tuple((
        query_or_value,
        unary_cmp_operator,
    ))(input)?;
    let (input, message_doc) = strip_comments_space(opt(alt((here_doc, message_doc))))(input)?;
    Ok((
        input,
        Expr::UnaryOperation(Box::new(UnaryExpr::new_with_msg(operator, expr, location, message_doc)))))
}

fn unary_expr(input: Span) -> IResult<Span, Expr> {
    alt((keys_or_indices_unary_expr(query_or_value), unary_post_bool_expr))(input)
}

fn anyone(input: Span) -> IResult<Span, UnaryOperator> {
    alt((
        value(UnaryOperator::AnyOne, strip_comments_trailing_space(alt((tag("anyone"), tag("some"), tag("atleast-one"))))),
        value(UnaryOperator::Any, strip_comments_trailing_space(tag("any")))
    ))(input)
}

fn parse_unary_bool_expr(input: Span) -> IResult<Span, Expr> {
    unary_expr(input)
}

fn parse_block_inner_expr(input: Span) -> IResult<Span, BlockExpr> {
    let location = Location::new(input.location_line(), input.get_column());
    let mut assignments = Vec::with_capacity(5);
    let mut ands_exprs = VecDeque::with_capacity(5);
    let (input, _) = alt((
        map(parse_let_expr, |e| {
            assignments.push(e)
        }),
        map(parse_and_conjunction, |e| ands_exprs.push_back(e)),
    ))(input)?;
    let mut next = input;
    loop {
        let result = alt((
            map(parse_let_expr, |e| { assignments.push(e); }),
            map(parse_and_conjunction, |e| { ands_exprs.push_back(e); }),
        ))(next);

        match result {
            Err(nom::Err::Error(pe)) => {
                if ands_exprs.is_empty() {
                    return Err(nom::Err::Failure(ParseError::new(pe.get_location().clone(),
                        "There are no conjunctions specified".to_string()
                    )))
                }
                return Ok((next, BlockExpr {
                    assignments, location, clause: reduce_ands_ors(ands_exprs, BinaryOperator::And)
                }))
            },

            Ok((input, _)) => {
                next = input;
            }

            Err(e) => return Err(e)
        }

    }
}

fn reduce_ands_ors(mut exprs: VecDeque<Expr>, op: BinaryOperator) -> Expr {
    if exprs.len() == 1 {
        return exprs.pop_front().unwrap()
    }
    let lhs = exprs.pop_front().unwrap();
    let location = Location::new(lhs.get_location().row() as u32, lhs.get_location().column());
    Expr::BinaryOperation(Box::new(BinaryExpr::new(
        op,
        lhs,
        reduce_ands_ors(exprs, op),
        location,
    )))
}

fn or_operator(input: Span) -> IResult<Span, BinaryOperator> {
    value(BinaryOperator::Or,
          alt((strip_comments_trailing_space(alt((tag("or"), tag("OR")))),
               strip_comments_space(tag("||"))))
    )(input)
}

fn and_operator(input: Span) -> IResult<Span, BinaryOperator> {
    value(BinaryOperator::And,
          alt((strip_comments_trailing_space(alt((tag("and"), tag("AND")))),
               strip_comments_space(tag("&&"))))
    )(input)
}

fn parse_rule_parameter(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, expr) = preceded(
        zero_or_more_ws_or_comment, query_or_value)(input)?;
    let (input, alt_value) = opt(preceded(or_operator, query_or_value))(input)?;
    match alt_value {
        Some(alternative) => Ok((input, Expr::BinaryOperation(
            Box::new(BinaryExpr::new(
                BinaryOperator::Or,
                expr,
                alternative,
                location
            ))
        ))),
        None => Ok((input, expr))
    }
}

fn parse_rule_clause_expr(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, name) = preceded(zero_or_more_ws_or_comment,parse_name)(input)?;
    let (input, parameters) = opt(
        preceded(zero_or_more_ws_or_comment,
                 parse_parameters(parse_rule_parameter)))(input)?;
    let (input, message) = opt(preceded(zero_or_more_ws_or_comment, message_doc))(input)?;
    match message {
        Some(message) => {
            Ok((input, Expr::RuleClause(Box::new(
                RuleClauseExpr::new(name, parameters, location, Some(message))))))
        },
        None => {
            //
            // If there is no message_doc, then is has to be followed up with or_operator, inline and
            // comment or newline or is enclosed inside group (), {}, or was the end of when condtio
            //
            if input.input_len() > 0 {
                let _ = peek(alt((
                    // part of OR / AND operations
                    value((), or_operator),
                    value((), and_operator),

                    // Has an implicit AND specified
                    value((), comment2),
                    value((), preceded(space0, newline)),

                    // Was a part of Group operations either inside ()
                    value((), parse_end_parenthesis),
                    //value((), parse_end_bracket),

                    // Was end of a when condition and being of the block
                    value((), parse_start_bracket),
                )))(input)?;
            }
            Ok((input, Expr::RuleClause(Box::new(
                RuleClauseExpr::new(name, parameters, location, None)))))
        }
    }
}

fn group_operations(input: Span) -> IResult<Span, Expr> {
    let (input, _parens) = strip_comments_space(char('('))(input)?;
    let (input, expr) =  parse_and_conjunction(input)?;
    let (input, _end_parens) = cut(strip_comments_space(char(')')))(input)?;
    Ok((input, expr))
}

fn parse_unary_binary_or_block_expr2<'a, P>(mut parser: P) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Expr>
where
    P: FnMut(Span<'a>) -> IResult<Span<'a>, Expr>
{
    move |input: Span| {
        let location = Location::new(input.location_line(), input.get_column());
        let (input, any) = opt(anyone)(input)?;
        let (input, expr) = parser(input)?;
        let expr = match any {
            Some(op) => {
                Expr::UnaryOperation(Box::new(UnaryExpr::new(op, expr, location)))
            },
            None => expr,
        };
        Ok((input, expr))
    }
}

fn parse_unary_binary_or_block_expr(input: Span) -> IResult<Span, Expr> {
    parse_unary_binary_or_block_expr2(
        alt((
            strip_comments_space( alt((
                parse_when_block_expr,
                parse_query_block_expr,
                parse_binary_bool_expr,
                parse_unary_bool_expr,
            ))),
            parse_rule_clause_expr,
        )))(input)

}

fn parse_disjunction_expr2<'a, P>(mut parser: P) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Expr>
where
    P: FnMut(Span<'a>) -> IResult<Span<'a>, Expr>
{
    move |input: Span| {
        let (input, expr) = parser(input)?;
        let mut ors = VecDeque::with_capacity(4);
        ors.push_back(expr);
        let mut next = input;
        loop {
            match or_operator(next) {
                Err(nom::Err::Error(_)) => return Ok((next, reduce_ands_ors(ors, BinaryOperator::Or))),
                Ok((span, _)) => {
                    let (span, expr) = cut(|i| parser(i))(span)?;
                    ors.push_back(expr);
                    next = span;
                },
                Err(e) => return Err(e)
            }
        }
    }
}

fn parse_disjunction_expr(input: Span) -> IResult<Span, Expr> {
    parse_disjunction_expr2(
        alt((
            group_operations,
            parse_unary_binary_or_block_expr
    )))(input)
}

fn parse_and_conjunction2<'a, P>(mut parser: P) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Expr>
where
    P: FnMut(Span<'a>) -> IResult<Span<'a>, Expr>
{
    move |input: Span| {
        let location = Location::new(input.location_line(), input.get_column());
        let (input, not) = opt(not)(input)?;
        let (input, expr) = parser(input)?;
        let mut ands = VecDeque::with_capacity(4);
        ands.push_back(expr);
        let mut next = input;
        loop {
            //
            // We reached here because we either hit an inline and_operator or
            // disjunction was done without or_operator joining clauses, which leads to
            // an implicit AND. We therefore only optional consume the inline and_operator
            // else we hit an implicit AND with \r or \n
            //
            match preceded(opt(and_operator), |i| parser(i))(next) {
                Err(nom::Err::Error(_)) => {
                    let and_expr = reduce_ands_ors(ands, BinaryOperator::And);
                    return Ok((next, match not {
                        Some(op) => Expr::UnaryOperation(Box::new(
                                UnaryExpr::new(op, and_expr, location)
                            )),
                        None => and_expr
                    }))
                }
                Ok((span, expr)) => {
                    ands.push_back(expr);
                    next = span;
                },
                Err(e) => return Err(e)
            }
        }
    }
}

fn parse_and_conjunction(input: Span) -> IResult<Span, Expr> {
    parse_and_conjunction2(parse_disjunction_expr)(input)
}

fn parse_query_block_expr(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, query) = parse_select(input)?;
    let query = match query { Expr::Select(q) => *q, _ => unreachable!() };
    let (input, block) = parse_block(input)?;
    let (input, message_doc) = strip_comments_space(opt(alt((here_doc, message_doc))))(input)?;
    Ok((input, Expr::Block(Box::new(BlockClauseExpr::new_with_msg(query, block, location, message_doc)))))
}

fn parse_block(input: Span) -> IResult<Span, BlockExpr> {
    let (input, _open) = parse_start_bracket(input)?;
    let (input, block) = cut(parse_block_inner_expr)(input)?;
    let (input, _end) = cut(parse_end_bracket)(input)?;
    Ok((input, block))
}

fn parse_when_cond_block_syntax(input: Span) -> IResult<Span, Expr> {
    let (input, _open) = parse_start_braces(input)?;
    let (input, expr) = cut(
        map(parse_block_inner_expr, |m| Expr::Filter(Box::new(m))))
        (input)?;
    let (input, _end) = cut(parse_end_braces)(input)?;
    Ok((input, expr))
}


fn parse_when_cond_syntax(input: Span) -> IResult<Span, Expr> {
    strip_comments_space(parse_and_conjunction2(parse_disjunction_expr2(parse_when_cond_expr)))(input)
}

fn parse_when_cond_expr(input: Span) -> IResult<Span, Expr> {
    parse_unary_binary_or_block_expr2(
        alt((
            strip_comments_space( alt((
                group_operations,
                parse_binary_bool_expr,
                parse_unary_bool_expr,
            ))),
            parse_rule_clause_expr,
        )))(input)
}

fn parse_when_block_expr(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, _when_keyword) = strip_comments_trailing_space(alt((
        tag("when"),
        tag("WHEN"),
    )))(input)?;
    let (input, expr) = alt((
        parse_when_cond_block_syntax,
        parse_when_cond_syntax))(input)?;
    let (input, block) = parse_block(input)?;
    Ok((input, Expr::When(Box::new(WhenExpr::new(expr, block, location)))))
}

fn parse_start_parenthesis(input: Span) -> IResult<Span, ()> {
    empty_value(char('('))(input)
}

fn parse_end_parenthesis(input: Span) -> IResult<Span, ()> {
    empty_value(char(')'))(input)
}


fn parse_parameters<'a, P>(mut parser: P) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Vec<Expr>>
where
    P: FnMut(Span<'a>) -> IResult<Span<'a>, Expr>
{
    move |input: Span| {
        let (input, _start_parens) = parse_start_parenthesis(input)?;
        let mut names = Vec::new();
        let mut span = input;
        loop {
            let (left, name) = cut(|i| parser(i))(span)?;
            names.push(name);
            span = left;
            match parse_value_separator(span) {
                Ok((left, _)) => {
                    if let Ok((left, _)) = parse_end_parenthesis(left) {
                        return Ok((left, names))
                    }
                    span = left;
                },

                Err(nom::Err::Error(_)) => {
                    let (left, _end) = cut(parse_end_parenthesis)(span)?;
                    return Ok((left, names))
                },

                Err(rest) => return Err(rest)
            }
        }
    }
}

fn parse_rule_parameter_names(input: Span) -> IResult<Span, Vec<Expr>> {
    parse_parameters(preceded(zero_or_more_ws_or_comment,var_name))(input)
}

fn parse_rule_expr(input: Span) -> IResult<Span, Expr> {
    let location = Location::new(input.location_line(), input.get_column());
    let (input, _rule_keyword) = strip_comments_trailing_space(tag("rule"))(input)?;
    let (input, name) = strip_comments_space(parse_name)(input)?;
    let (input, parameters) = opt(parse_rule_parameter_names)(input)?;

    match parse_when_block_expr(input) {
        Ok((input, Expr::When(when))) => {
            let when = *when;
            Ok((input, Expr::Rule(Box::new(RuleExpr {
                block: when.block,
                location,
                name,
                parameters,
                when: Some(when.when)
            }))))
        },

        Err(nom::Err::Error(_)) => {
            let (input, block) = cut(parse_block)(input)?;
            Ok((input, Expr::Rule(Box::new(RuleExpr {
                block,
                location,
                name,
                parameters,
                when: None
            }))))

        },

        Err(e) => Err(e),

        _ => unreachable!()
    }
}

pub(crate) fn parse_rules_file<'a>(input: Span<'a>, name: &str) -> IResult<Span<'a>, Expr> {
    let mut assignments = Vec::new();
    let mut rules = Vec::new();
    let mut span = input;
    loop {
        match strip_comments_space(alt((
            parse_let_expr,
            parse_rule_expr
        )))(span) {
            Ok((left, Expr::Let(let_expr))) => {
                assignments.push(*let_expr);
                span = left;
            },

            Ok((left,Expr::Rule(rule))) => {
                rules.push(*rule);
                span = left;
            },

            Err(nom::Err::Error(_)) => {
                if span.input_len() > 0 {
                    return Err(nom::Err::Failure(
                        ParseError::new(
                            Location::new(span.location_line(), span.get_column()),
                            "EOF not detected".to_string())))
                }
                return Ok((span, Expr::File(Box::new(FileExpr::new(name.to_string(), assignments, rules)))))
            }

            Err(e) => return Err(e),

            _ => unreachable!()

        }
    }
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