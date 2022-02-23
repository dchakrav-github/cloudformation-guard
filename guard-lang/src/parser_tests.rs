use super::*;
use nom::Err;

#[test]
fn test_parser_name() {
    let success = [
        "lambda",
        "s3_buckets",
        "ec2_security_groups",
        "ec2_security_rule type"
    ];

    success.iter()
        .map(|s|
            (Span::new_extra(*s, ""),
             s.split(" ").next().map_or("", std::convert::identity))
        )
        .for_each(|(span, compare)| {
            let result = parse_name(span);
            assert_eq!(result.is_ok(), true);
            assert_eq!(result.unwrap().1, compare)
        });

    let failures = [
        "",
        "1reject",
    ];

    failures.iter()
        .map(|s|
            (Span::new_extra(*s, ""),
             s.split(" ").next().map_or("", std::convert::identity))
        )
        .for_each(|(span, compare)| {
            let result = parse_name(span);
            assert_eq!(result.is_err(), true);
        });
}

#[test]
fn test_parse_bool() {
    let success_true = [
        "true",
        "True",
        "TRUE",
        "T",
    ];

    success_true.iter().map(|s| Span::new_extra(*s, ""))
        .for_each(|span| {
            let result = parse_bool(span);
            assert_eq!(result.is_ok(), true);
            assert_eq!(result.unwrap().1,
                       Expr::Bool(Box::new(BoolExpr::new(true, Location::new(1, 1)))));
        });

    let false_pass = [
        "false",
        "False",
        "FALSE",
        "F",
    ];

    false_pass.iter().map(|s| Span::new_extra(*s, ""))
        .for_each(|span| {
            let result = parse_bool(span);
            assert_eq!(result.is_ok(), true);
            assert_eq!(result.unwrap().1,
                       Expr::Bool(Box::new(BoolExpr::new(false, Location::new(1, 1)))));
        });

    let failures = [
        "",
        "1",
        "0"
    ];

    failures.iter().map(|s| Span::new_extra(*s, ""))
        .for_each(|span| {
            let result = parse_bool(span);
            assert_eq!(result.is_err(), true);
        });
}

#[test]
fn test_parse_string() {
    let success = [
        "literal",
        "1literal",
        "this is",
        "escap\\\"ed",
    ];

    success.iter().map(|s| (format!("\"{}\"", s), *s))
        .map(|(to_parse, content)| {
            (to_parse,
            if content.contains("\\") {
                content.replace("\\", "")
            }
            else {
                content.to_string()
            })
        })
        .for_each(
            |(to_parse, expected)| {
                let span = Span::new_extra(&to_parse, "");
                let result = parse_string(span);
                let location = Location::new(1, 1);
                assert_eq!(result.is_ok(), true);
                match result {
                    Ok((_, Expr::String(s))) => {
                        assert_eq!(s.value(), &expected);
                        assert_eq!(s.location(), &location);
                    }
                    _ => unreachable!()
                }
            }
        );

    let success = [
        "\"Hello World\"",
        "Added embedded \"string\"",
    ];
    success.iter().map(|s| (format!("\'{}\'", s), *s))
        .for_each(
            |(to_parse, expected)| {
                let span = Span::new_extra(&to_parse, "");
                let result = parse_string(span);
                let location = Location::new(1, 1);
                assert_eq!(result.is_ok(), true);
                match result {
                    Ok((_, Expr::String(s))) => {
                        assert_eq!(s.value(), expected);
                        assert_eq!(s.location(), &location);
                    }
                    _ => unreachable!()
                }
            }
        );

    let failures = [
        "",
        "No Quotes"
    ];

    failures.iter().for_each(
        |e| {
            let span = Span::new_extra(*e, "");
            let result = parse_string(span);
            assert_eq!(result.is_err(), true);
            match result {
                Err(nom::Err::Error(pe)) => {
                    let location = pe.get_location();
                    assert_eq!(location.row(), 1);
                    assert_eq!(location.column(), 1);
                },
                _ => unreachable!()
            }
        })

}

#[test]
fn test_parse_regex() {

    let success = [
        "/.*PROD.*/",
        "/arn:[\\w+=\\/,.@-]+:[\\w+=\\/,.@-]+:[\\w+=\\/,.@-]*:[0-9]*:[\\w+=,.@-]+(\\/[\\w+=,.@-]+)*/",
        "/notescape/d",
    ];

    success.iter().map(|s| {
        let mut to_match = s.replace("\\/", "/");
        let mut to_chars = to_match.chars();
        to_chars.next();
        to_chars.next_back();
        (*s, to_chars.as_str().to_string())
    }).for_each( |(to_parse, expected)| {
        let span = Span::new_extra(to_parse, "");
        let result = parse_regex(span);
        assert_eq!(result.is_ok(), true);
        match result {
            Ok((_span, Expr::Regex(regex))) => {
                if regex.value().contains("not") {
                    assert_eq!(regex.value(), "notescape");
                }
                else {
                    assert_eq!(regex.value(), expected);
                }
            },
            _ => unreachable!()
        }
    });

    let failures = [
        "",
        "/open",
        "close/",
    ];

    failures.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_regex(span);
        assert_eq!(result.is_err(), true);
        match result {
            Err(nom::Err::Error(pe)) => {
                assert_eq!(pe.get_location().row(), 1);
                assert_eq!(pe.get_location().column() == 1 || pe.get_location().column() == 6, true);
            },

            _ => unreachable!()
        }
    });

}

#[test]
fn test_parse_int() {
    let success = [
        "100",
        "200E",
        "0",
        "0123K-12"
    ];

    let expected = [
        100,
        200,
        0,
        123
    ];

    success.iter().zip(&expected).for_each(
        |(to_parse, expected)| {
            let span = Span::new_extra(*to_parse, "");
            let result = parse_int_value(span);
            assert_eq!(result.is_ok(), true);
            match result {
                Ok((span, Expr::Int(val))) => {
                    assert_eq!(val.value(), *expected);
                    if *expected == 200 {
                        assert_eq!(span.get_column(), 4);
                        assert_eq!(*span.fragment(), "E");
                    }

                    if *expected == 123 {
                        assert_eq!(span.get_column(), 5);
                        assert_eq!(*span.fragment(), "K-12");
                    }
                },
                _ => unreachable!()
            }
        }
    );

    let failures = [
        "",
        "a10",
        "error"
    ];

    failures.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_int_value(span);
        assert_eq!(result.is_err(), true);
        match result {
            Err(nom::Err::Error(pe)) => {
                assert_eq!(pe.get_location().row(), 1);
                assert_eq!(pe.get_location().column(), 1);
            },
            _ => unreachable!()
        }
    });

}

#[test]
fn test_parse_range() {
    let success = [
        "r(10, 20)",
        r#"r(10,
             20)"#,
        r###"r(# okay starting with 10
               10, # ending with 20 not inclusive
             20)"###,
        "r[100, 200]",
        "r(100, 200]"
    ];

    let expected = [
        RangeType { lower: 10, upper: 20, inclusive: 0 },
        RangeType { lower: 10, upper: 20, inclusive: 0 },
        RangeType { lower: 10, upper: 20, inclusive: 0 },
        RangeType { lower: 100, upper: 200, inclusive: crate::types::LOWER_INCLUSIVE | crate::types::UPPER_INCLUSIVE },
        RangeType { lower: 100, upper: 200, inclusive: crate::types::UPPER_INCLUSIVE },
    ];

    success.iter().zip(&expected)
        .for_each(
            |(to_parse, expected)| {
                let span = Span::new_extra(*to_parse, "");
                let range = parse_range(span);
                assert_eq!(range.is_ok(), true);
                match range.unwrap().1 {
                    Expr::RangeInt(range) => {
                        assert_eq!(range.value(), expected);
                    },
                    _ => unreachable!()
                }
            }
        );

    let success = [
        r#"r(10.2,
             20.5)"#,
        "r[100.0, 200.10]",
        "r(10.0, 20]",
    ];

    let expected = [
        RangeType { lower: 10.2, upper: 20.5, inclusive: 0 },
        RangeType { lower: 100.0, upper: 200.10, inclusive: crate::types::LOWER_INCLUSIVE | crate::types::UPPER_INCLUSIVE },
        RangeType { lower: 10.0, upper: 20.0, inclusive: crate::types::UPPER_INCLUSIVE },
    ];

    success.iter().zip(&expected)
        .for_each(
            |(to_parse, expected)| {
                let span = Span::new_extra(*to_parse, "");
                let range = parse_range(span);
                assert_eq!(range.is_ok(), true);
                match range.unwrap().1 {
                    Expr::RangeFloat(range) => {
                        assert_eq!(range.value(), expected);
                    },
                    _ => unreachable!()
                }
            }
        );

    let failures = [
        "",
        "r(10",
        "r[10, \"error\"]",
        "r(10, 20", // failure
    ];

    let locations = [
        Location::new(1, 1),
        Location::new(1, failures[1].len() + 1),
        Location::new(1, "r[10, ".len() + 1),
        Location::new(1, "r(10, 20".len() + 1),
    ];

    failures.iter().zip(&locations)
        .for_each(
            |(to_parse, loc)| {
                let span = Span::new_extra(*to_parse, "");
                let result = parse_range(span);
                println!("{:?}", result);
                assert_eq!(result.is_err(), true);
                result.map_err(|err| match err {
                    nom::Err::Error(pe) |
                    nom::Err::Failure(pe) => {
                        assert_eq!(pe.get_location(), loc);
                    },
                    _ => unreachable!()
                });
            }
        );
}

#[test]
fn test_parse_map() {

    let success = [
        r###"
        {
            size: 10,
            length: 20,
            units: {
                type: "centimeters",
            }
        }
        "###,
        "{ size: 10, }",
        "{ size: 10, length: 20 }",
        "{ ok: true, value: null }",
    ];

    success.iter()
        .for_each(
            |to_parse| {
                let span = Span::new_extra(*to_parse, "");
                let result = parse_map(span);
                assert_eq!(result.is_ok(), true);
            }
        )

}