use super::*;
use crate::visitor::Visitor;

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
        .for_each(|(span, _compare)| {
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
    ];

    success_true.iter().map(|s| Span::new_extra(*s, ""))
        .for_each(|span| {
            let result = parse_bool(span);
            assert_eq!(result.is_ok(), true);
            let expr = match result.unwrap().1 {
                Expr::Bool(b) => *b,
                _ => unreachable!()
            };
            assert_eq!(expr.value, true);
            assert_eq!(expr.location, Location::new(1, 1));
        });

    let false_pass = [
        "false",
        "False",
        "FALSE",
    ];

    false_pass.iter().map(|s| Span::new_extra(*s, ""))
        .for_each(|span| {
            let result = parse_bool(span);
            assert_eq!(result.is_ok(), true);
            let expr = match result.unwrap().1 {
                Expr::Bool(b) => *b,
                _ => unreachable!()
            };
            assert_eq!(expr.value, false);
            assert_eq!(expr.location, Location::new(1, 1));
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
        let to_match = s.replace("\\/", "/");
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
        "0123K-12",
        "+10",
        "-10",
        "+12E+10", // technically float but will be parsed as integer, order matters
    ];

    let expected = [
        100,
        200,
        0,
        123,
        10,
        -10,
        12
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

                    if *expected == 12 {
                        assert_eq!(span.get_column(), 4);
                        assert_eq!(*span.fragment(), "E+10");
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
fn test_parse_float_value() {
    let success = [
        "10.9",
        ".9",
        "1.",
        "1e10",
        "1e-10",
        "1.2E10"
    ];

    let expected : Vec<f64> = success.iter().map(|s| s.parse::<f64>().unwrap()).collect();

    success.iter().zip(&expected)
        .for_each(
            |(to_parse, expected)| {
                let span = Span::new_extra(*to_parse, "");
                let result = parse_float(span);
                assert_eq!(result.is_ok(), true);
                match result.unwrap().1 {
                    Expr::Float(val) => {
                        assert_eq!(val.value(), *expected);
                    },
                    _ => unreachable!()
                }
            }
        );

    let failures = [
        "10",
        "10K",
        "10 ",
        "",
        "error"
    ];

    let locations = [
        Location::new(1, 3),
        Location::new(1, 3),
        Location::new(1, 3),
        Location::new(1, 1),
        Location::new(1, 1)
    ];

    failures.iter().zip(&locations).for_each(
        |(to_parse, location)| {
            let span = Span::new_extra(*to_parse, "");
            let result = parse_float(span);
            assert_eq!(result.is_err(), true);
            match result.unwrap_err() {
                nom::Err::Error(pe) |
                nom::Err::Failure(pe) => {
                    assert_eq!(pe.get_location(), location);
                },
                _ => unreachable!()
            }
        }
    )

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
                assert_eq!(result.is_err(), true);
                let _ =result.map_err(|err| match err {
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
        r###"
        {
           size: # this is a comment
              10,
        }
        "###,
        "{}",
        r#"
        { size: 10,
          type: "goodwill",
          amount: 10.5
        }
        "#,
    ];

    success.iter()
        .for_each(
            |to_parse| {
                let span = Span::new_extra(*to_parse, "");
                let result = parse_map(span);
                assert_eq!(result.is_ok(), true);
            }
        );

    let failures = [
        r###"{ size 10 }"###, // no value separator
        "{ size: 20, { error: true } }", // no map key
        r#"{ "touch": true, dimenstions: [ 10, 20, 30, ]"#,
        "{",
    ];

    let expected_failures = [
        Location::new(1, "{ size ".len() + 1),
        Location::new(1, "{ size: 20, ".len() + 1),
        Location::new(1, r#"{ "touch": true, dimenstions: [ 10, 20, 30, ]"#.len() + 1),
        Location::new(1, "{".len() + 1),
    ];

    failures.iter().zip(&expected_failures)
        .for_each(
            |(to_parse, failure)| {
                let span = Span::new_extra(*to_parse, "");
                let result = parse_map(span);
                assert_eq!(result.is_err(), true);
                let _ = result.map_err(|err| {
                    match err {
                        nom::Err::Error(pe) |
                        nom::Err::Failure(pe) => {
                            assert_eq!(pe.get_location(), failure);
                        },
                        _ => unreachable!()
                    }
                    ()
                });
            }
        )

}

#[test]
fn test_parse_collection() {
    let success = [
        "[10, 20]",
        "[10, 20, 30, { mixed: true },]",
        "[[10, 20], 30, [40, 50]]"
    ];

    success.iter()
        .for_each(
            |to_parse| {
                let span = Span::new_extra(*to_parse, "");
                let result = parse_array(span);
                assert_eq!(result.is_ok(), true);
            }
        );

    let failures = [
        "",
        "[",
        "[10, 40,",
        "10, 40]",
    ];

    let locations = [
        Location::new(1, 1),
        Location::new(1, 2),
        Location::new(1, "[10, 40,".len() + 1),
        Location::new(1, 1)
    ];

    failures.iter().zip(&locations)
        .for_each(
            |(to_parse, location)| {
                let span = Span::new_extra(*to_parse, "");
                let result = parse_array(span);
                assert_eq!(result.is_err(), true);
                match result.unwrap_err() {
                    nom::Err::Failure(pe) |
                    nom::Err::Error(pe) => {
                        assert_eq!(pe.get_location(), location);
                    },
                    _ => unreachable!()
                }
            }
        );
}

#[test]
fn test_parse_variable_reference() {
    let success = [
        "%var",
        "%var2",
        "%var_this"
    ];

    let expected: Vec<String> = success.iter().map(|s| s.replace("%", "")).collect();

    success.iter().zip(&expected).for_each(|(to_parse, expected)| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_variable_reference(span);
        assert_eq!(result.is_ok(), true);
        match result.unwrap().1 {
            Expr::VariableReference(sr) => {
                assert_eq!(sr.value(), expected);
            }
            _ => unreachable!()
        }
    });

    let failures = [
        "",
        "var",
        "10%var",
    ];

    failures.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_variable_reference(span);
        assert_eq!(result.is_err(), true);
    })
}

#[test]
fn test_parse_query_filter_segment() {
    let success = [
        "[name]",
        "[ bucket_name ]",
        r###"[ # comment here
            bucket_name
        ]"###,
        "[ %names ]",
        "[ 'lookup' ]",
        "[1]",
        r###"[ # select the set of names
           %names]"###
        // TODO need block expr to complete
    ];

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_query_filter_segment(parse_var_block)(span);
        assert_eq!(result.is_ok(), true);
        match result.unwrap().1 {
            (Expr::Variable(expr), None) => {
                let StringExpr { value, .. } = *expr;
                assert_eq!(value == "name" || value == "bucket_name", true);
            },

            (Expr::VariableReference(expr), None) => {
                let StringExpr { value, .. } = *expr;
                assert_eq!(value, "names");
            },

            (Expr::String(expr), None) => {
                let StringExpr{ value, .. } = *expr;
                assert_eq!(value, "lookup");
            },

            (Expr::Int(expr), None) => {
                let IntExpr{ value, .. } = *expr;
                assert_eq!(value, 1);
            },

            _ => unreachable!()
        }
    });

    let failures = [
        "",
        //"[]",
        //"[{}]"
    ];

    let locations = [
        Location::new(1, 1),
    ];

    failures.iter().zip(&locations).for_each(|(to_parse, loc)| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_query_filter_segment(parse_var_block)(span);
        assert_eq!(result.is_err(), true);
        if let nom::Err::Error(e) = result.unwrap_err() {
            assert_eq!(e.get_location(), loc);
        }

    });
}

#[test]
fn test_parse_query() {
    let success = [
        "Resources.*.Properties.Tags",
        "Resources[name].Properties.Tags",
        "Resources",
        "Resources[%buckets].Properties.Tags",
    ];

    success.iter() .for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_select(span);
        assert_eq!(result.is_ok(), true);
        match result.unwrap().1 {
            Expr::Select(query) => {
                let QueryExpr { parts, .. } = *query;
                for (idx, expr) in parts.iter().enumerate() {
                    match idx {
                        0 => {
                            if let Expr::String(s) = expr {
                                assert_eq!(s.value, "Resources");
                            }
                        },
                        1 => {
                            match expr {
                                Expr::String(s) => {
                                    let value = s.value();
                                    assert_eq!(value, "*");
                                },
                                Expr::Variable(var) => {
                                        let value = var.value();
                                        assert_eq!(value, "name");
                                },
                                Expr::VariableReference(var_ref) => {
                                    assert_eq!(var_ref.value(), "buckets");
                                },
                                _ => unreachable!()
                            }
                        },
                        2 => {
                            if let Expr::String(s) = expr {
                                assert_eq!(s.value(), "Properties")
                            }
                        },
                        3 => {
                            if let Expr::String(s) = expr {
                                assert_eq!(s.value(), "Tags")
                            }
                        },
                        _ => unreachable!()
                    }
                }
            },
            _ => unreachable!()
        }
    })
}

#[test]
fn test_unary_operator() {
    let operators = [
        "EXISTS",
        "EMPTY",
        "IS_BOOL",
        "IS_STRING",
        "IS_INT",
        "IS_FLOAT",
        "IS_REGEX",
        "IS_LIST",
        "IS_MAP",
        "NOT"
    ];

    let expected = [
        (UnaryOperator::Exists, UnaryOperator::NotExists),
        (UnaryOperator::Empty, UnaryOperator::NotEmpty),
        (UnaryOperator::IsBool, UnaryOperator::IsNotBool),
        (UnaryOperator::IsString, UnaryOperator::IsNotString),
        (UnaryOperator::IsInt, UnaryOperator::IsNotInt),
        (UnaryOperator::IsFloat, UnaryOperator::IsNotFloat),
        (UnaryOperator::IsRegex, UnaryOperator::IsNotRegex),
        (UnaryOperator::IsList, UnaryOperator::IsNotList),
        (UnaryOperator::IsMap, UnaryOperator::IsNotMap),
        (UnaryOperator::Not, UnaryOperator::Not)
    ];

    let operators: Vec<(String, String, String, String)> = operators.iter()
        .map(|s| (s.to_string(), s.to_lowercase()))
        .zip(&["!", "NOT", "not"])
        .map(
            |((upper, lower), not)| {
                let (not_upper, not_lower) = (format!("{} {}", not, upper), format!("{} {}", not, lower));
                (upper, lower, not_upper, not_lower)
            }
        )
        .collect();

    operators.iter().zip(&expected).for_each(|(operators, expected)| {
        let span = Span::new_extra(&operators.0, "");
        let result = unary_cmp_operator(span);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().1, expected.0);

        let span = Span::new_extra(&operators.1, "");
        let result = unary_cmp_operator(span);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().1, expected.0);

        let span = Span::new_extra(&operators.2, "");
        let result = unary_cmp_operator(span);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().1, expected.1);

        let span = Span::new_extra(&operators.3, "");
        let result = unary_cmp_operator(span);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().1, expected.1);
    });

}

#[test]
fn test_parse_unary_expr() {
    let success = [
        "Resources[*].Properties.Tags EXISTS << Ensure Tags EXISTs >>",
        r#"Resources.*.Properties.Tags !EXISTS <<YAML
        message: Tags Do not exist
        Guide: https://guides.bestpractices.aws
        YAML
        "#,
        "Resources[name].Properties.Tags EMPTY",
        "Resources[%buckets].Properties.Tags NOT EMPTY",
        "Resources.%buckets.Properties.Tags NOT EMPTY",
        r#"Resources.
            %buckets.
            Properties.
            Tags NOT EMPTY"#,
        r###"Resources
            # extract the list of buckets we used
            .%buckets
            .Properties
            # checking if Tags EXISTS and is not EMPTY
            .Tags !EMPTY"###,
        r###"keys %sqs_queues"###,
        r###"keys Resources[ Type == 'AWS::SQS::Queue' ]"###,
    ];

    for (_idx, expr) in success.iter().enumerate() {
        let span = Span::new_extra(expr, "");
        let result = parse_unary_bool_expr(span);
        assert_eq!(result.is_ok(), true, "{} {:?}", expr, result);
        let unary = result.unwrap().1;
        struct UnaryVisitor{}
        impl<'expr> Visitor<'expr> for UnaryVisitor {
            type Value = ();
            type Error = String;

            fn visit_unary_operation(self, _expr: &'expr Expr, value: &'expr UnaryExpr) -> Result<Self::Value, Self::Error> {
                if value.operator == UnaryOperator::Not {
                    return value.expr.accept(UnaryVisitor{});
                }
                assert_eq!(value.operator == UnaryOperator::Exists ||
                           value.operator == UnaryOperator::NotExists ||
                           value.operator == UnaryOperator::Keys ||
                           value.operator == UnaryOperator::Empty ||
                           value.operator == UnaryOperator::NotEmpty, true);
                struct ExpectQuery{}
                impl<'expr> Visitor<'expr> for ExpectQuery {
                    type Value = ();
                    type Error = String;

                    fn visit_select(self, _expr: &'expr Expr, value: &'expr QueryExpr) -> Result<Self::Value, Self::Error> {
                        assert_eq!(value.parts.len() == 4 || value.parts.len() == 1 || value.parts.len() == 2, true);
                        struct ExpectedPart{}
                        impl<'expr> Visitor<'expr> for ExpectedPart {
                            type Value = ();
                            type Error = String;

                            fn visit_select(self, _expr: &'expr Expr, value: &'expr QueryExpr) -> Result<Self::Value, Self::Error> {
                                assert_eq!(value.parts.len(), 1);
                                for each in &value.parts {
                                    each.accept(ExpectedPart{})?;
                                }
                                Ok(())
                            }

                            fn visit_binary_operation(self, _expr: &'expr Expr, value: &'expr BinaryExpr) -> Result<Self::Value, Self::Error> {
                                assert_eq!(value.operator, BinaryOperator::Equals);
                                value.lhs.accept(ExpectedPart{})?;
                                value.rhs.accept(ExpectedPart{})?;
                                Ok(())
                            }



                            fn visit_string(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
                                assert_eq!(
                                    value.value == "Resources" ||
                                    value.value == "*" ||
                                    value.value == "Properties" ||
                                    value.value == "Type" ||
                                    value.value == "AWS::SQS::Queue" ||
                                    value.value == "Tags",
                                    true,
                                    "Unexpected {}", value.value
                                );
                                Ok(())
                            }

                            fn visit_filter(self, _expr: &'expr Expr, value: &'expr BlockExpr) -> Result<Self::Value, Self::Error> {
                                assert_eq!(value.assignments.is_empty(), true);
                                value.clause.accept(ExpectedPart{})?;
                                Ok(())
                            }

                            fn visit_variable(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
                                assert_eq!(
                                    value.value, "name"
                                );
                                Ok(())
                            }

                            fn visit_variable_reference(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
                                assert_eq!(
                                    value.value == "buckets" ||
                                    value.value == "sqs_queues",
                                    true,
                                    "{}", value.value
                                );
                                Ok(())
                            }



                            fn visit_any(self, expr: &'expr Expr) -> Result<Self::Value, Self::Error> {
                                Err(format!("Unexpected Expr {:?}", expr))
                            }
                        }
                        for each in &value.parts {
                            each.accept(ExpectedPart{})?;
                        }
                        Ok(())
                    }


                    fn visit_any(self, expr: &'expr Expr) -> Result<Self::Value, Self::Error> {
                        Err(format!("Unexpected Expr {:?}", expr))
                    }
                }
                value.expr.accept(ExpectQuery{})?;
                Ok(())
            }


            fn visit_any(self, expr: &'expr Expr) -> Result<Self::Value, Self::Error> {
                Err(format!("Unexpected expr {:?}", expr))
            }
        }
        let result = unary.accept(UnaryVisitor{});
        assert_eq!(result.is_ok(), true, "Error {:?}", result);
    }
}

#[test]
fn test_binary_cmp_operator() {
    let success = [
        "==",
        "!=",
        ">",
        "<",
        "<=",
        ">=",
        "> >",  // Ok
        "< <",  // Ok
        "in " // has to have space after
    ];

    let expected = [
        BinaryOperator::Equals,
        BinaryOperator::NotEquals,
        BinaryOperator::Greater,
        BinaryOperator::Lesser,
        BinaryOperator::LesserThanEquals,
        BinaryOperator::GreaterThanEquals,
        BinaryOperator::Greater,
        BinaryOperator::Lesser,
        BinaryOperator::In
    ];

    success.iter().zip(&expected).for_each(|(to_parse, expected)| {
        let span = Span::new_extra(*to_parse, "");
        let result = binary_cmp_operator(span);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap().1, *expected);
    });

    let failures= [
        "",
        ">>",
        "<<",
        "not"
    ];

    failures.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = binary_cmp_operator(span);
        assert_eq!(result.is_err(), true);
    });
}

#[test]
fn test_here_doc() {
    let success = [
        "<<EOM\nthis is the message EOM ",
        r#"<<EOM
        This is a multiline message that end here EOM
        "#,
        r#"<<YAML
        message: %name instance was not compliant
        Guide: https://mycompany-guide.aws
        YAML
        "#
    ];

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = here_doc(span);
        assert_eq!(result.is_ok(), true);
    });

    let failures = [
        "",
        "<<EOM no newline after doc part EOM ",
        "<<START\n with no end ",
        "<<EOM\n no space at end EOM",
    ];

    failures.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = here_doc(span);
        assert_eq!(result.is_err(), true);
    });
}

#[test]
fn test_message_doc() {
    let success = [
        "<<this is the message>>",
        r#"<<
        This is a multiline message that end here EOM>>"#,
        r#"<<
        message: %name instance was not compliant
        Guide: https://mycompany-guide.aws
        >>"#
    ];

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = message_doc(span);
        assert_eq!(result.is_ok(), true);
        let remaining = result.unwrap().0;
        assert_eq!(remaining.fragment().is_empty(), true);

    });

    let failures = [
        "",
        "<<no ending doc part",
        "<< START with no end ",
    ];

    failures.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = message_doc(span);
        assert_eq!(result.is_err(), true);
    });
}

#[test]
fn test_binary_expr() {
    let success = [
        "Resources.*.Properties.Tags[*].Key != /^GG/",
        "CpuUnits >= 10",
        "Spec.Memory <= 200",
        "Spec.Memory IN [200, 300, 800]",
        "Spec.Cpu >= Parameters.AllowedMinValue",
        r#"Statements[*].Principal != '*'"#,
        r###"
        Statements[index].
            Principal != '*'
        "###,
        r###"
        IAM.Grants[*].Principal != '*' # okay we have some doc message here
            <<GRANT_ALL_DISALLOWED
            This is a here doc that is working this out well
            We are going to end this here
            GRANT_ALL_DISALLOWED
        "###,
        r#"Ensure < "Message" << Message still works >>"#
    ];

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_binary_bool_expr(span);
        println!("{} {:?}", to_parse, result);
        assert_eq!(result.is_ok(), true);
        struct BinaryExprChecker{}
        impl<'expr> Visitor<'expr> for BinaryExprChecker {
            type Value = ();
            type Error = ();

            fn visit_select(self, _expr: &'expr Expr, value: &'expr QueryExpr) -> Result<Self::Value, Self::Error> {
                for each in &value.parts {
                    each.accept(BinaryExprChecker{})?;
                }
                Ok(())
            }

            fn visit_binary_operation(self, _expr: &'expr Expr, value: &'expr BinaryExpr) -> Result<Self::Value, Self::Error> {
                assert_eq!(
                    value.operator == BinaryOperator::In ||
                    value.operator == BinaryOperator::NotEquals ||
                    value.operator == BinaryOperator::Lesser ||
                    value.operator == BinaryOperator::LesserThanEquals ||
                    value.operator == BinaryOperator::GreaterThanEquals,
                    true
                );
                value.lhs.accept(BinaryExprChecker{})?;
                value.rhs.accept(BinaryExprChecker{})?;
                if let Some(msg) = &value.message {
                    assert_eq!(msg.contains("here doc that is working") || msg.contains("still works"), true);
                }
                Ok(())
            }

            fn visit_array(self, _expr: &'expr Expr, value: &'expr ArrayExpr) -> Result<Self::Value, Self::Error> {
                for each in &value.elements {
                    each.accept(BinaryExprChecker{})?;
                }
                Ok(())
            }

            fn visit_string(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
                assert_eq!(
                    value.value == "Resources" ||
                    value.value == "*" ||
                    value.value == "Properties" ||
                    value.value == "Tags" ||
                    value.value == "Key" ||
                    value.value == "CpuUnits" ||
                    value.value == "Spec" ||
                    value.value == "Memory" ||
                    value.value == "Cpu" ||
                    value.value == "Statements" ||
                    value.value == "Principal" ||
                    value.value == "Parameters" ||
                    value.value == "AllowedMinValue" ||
                    value.value == "IAM" ||
                    value.value == "Grants" ||
                    value.value == "Ensure" ||
                    value.value == "Message" ||
                    value.value == "Resources",
                    true
                );
                Ok(())
            }

            fn visit_regex(self, _expr: &'expr Expr, value: &'expr RegexExpr) -> Result<Self::Value, Self::Error> {
                assert_eq!(value.value, "^GG");
                Ok(())
            }

            fn visit_int(self, _expr: &'expr Expr, value: &'expr IntExpr) -> Result<Self::Value, Self::Error> {
                assert_eq!(
                    value.value == 10 ||
                    value.value == 300 ||
                    value.value == 800 ||
                    value.value == 200,
                    true
                );
                Ok(())
            }

            fn visit_variable(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
                assert_eq!(value.value, "index");
                Ok(())
            }


            fn visit_any(self, _expr: &'expr Expr) -> Result<Self::Value, Self::Error> {
                todo!()
            }
        }
        let expr = result.unwrap().1;
        let result = expr.accept(BinaryExprChecker{});
        assert_eq!(result.is_ok(), true);

    });
}

#[test]
fn test_let_expr() {
    let success = [
        "let literal = [10, 20]",
        r#"let map = { hi: "there", bye: 20 }"#,
        r#"let query = Parameters.AWS.allowedPrefixLists"#,
        r#"let query = Parameters.AWS.allowedPrefixLists || []"#,
        r#"let query = keys Parameters"#, // keys used to extract keys
        r#"let query = keysParameters"#, // Okay when keys is part of query
        r#"let query = 'keys'"#, // Okay when keys isn't part of query
    ];

    struct LetExprAssertions{}
    impl<'expr> Visitor<'expr> for LetExprAssertions {
        type Value = ();
        type Error = ();

        fn visit_let(self, _expr: &'expr Expr, value: &'expr LetExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.name == "literal"                ||
                value.name == "map"                    ||
                value.name == "query",
                true
            );
            value.value.accept(LetExprAssertions{})?;
            Ok(())
        }


        fn visit_select(self, _expr: &'expr Expr, value: &'expr QueryExpr) -> Result<Self::Value, Self::Error> {
            for each in &value.parts {
                each.accept(LetExprAssertions{})?;
            }

            Ok(())
        }

        fn visit_binary_operation(self, _expr: &'expr Expr, value: &'expr BinaryExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(value.operator, BinaryOperator::Or);
            value.lhs.accept(LetExprAssertions{})?;
            value.rhs.accept(LetExprAssertions{})?;
            Ok(())
        }

        fn visit_unary_operation(self, _expr: &'expr Expr, value: &'expr UnaryExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(value.operator, UnaryOperator::Keys);
            value.expr.accept(LetExprAssertions{})?;
            Ok(())
        }


        fn visit_array(self, _expr: &'expr Expr, value: &'expr ArrayExpr) -> Result<Self::Value, Self::Error> {
            // Empty for the Or assignment case
            assert_eq!(
                value.elements.is_empty() ||
                value.elements.len() == 2,
                true
            );

            if !value.elements.is_empty() {
                for each in &value.elements {
                    each.accept(LetExprAssertions {})?;
                }
            }
            Ok(())
        }

        fn visit_map(self, _expr: &'expr Expr, value: &'expr MapExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(value.entries.len(), 2);
            for each in value.entries.keys() {
                assert_eq!(
                    each == "hi" ||
                    each == "bye",
                    true
                );
            }
            Ok(())
        }

        fn visit_string(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.value == "keys"                   ||
                value.value == "keysParameters"         ||
                value.value == "Parameters"             ||
                value.value == "AWS"                    ||
                value.value == "allowedPrefixLists",
                true
            );
            Ok(())
        }

        fn visit_int(self, _expr: &'expr Expr, value: &'expr IntExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.value == 10 ||
                value.value == 20,
                true
            );
            Ok(())
        }

        fn visit_any(self, _expr: &'expr Expr) -> Result<Self::Value, Self::Error> {
            todo!()
        }
    }

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_let_expr(span);
        println!("{}, {:?}", to_parse, result);
        assert_eq!(result.is_ok(), true);
        let result = result.unwrap().1;
        let r = result.accept(LetExprAssertions{});
        assert_eq!(r.is_ok(), true);
    });

    let failures = [
        "",
        "s = 10",
        "let s 10"
    ];

    failures.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_let_expr(span);
        assert_eq!(result.is_err(), true, "{}, {:?}", to_parse, result);
    });

}

#[test]
fn test_or_disjunctions() {
    let success = [
        "Resources EXISTS or resourceType EXISTS",
        "Resources EXISTS || configuration EXISTS",
        r#"Resources EXISTS OR configuration.Properties.Principals != '*'"#,
        r###"Resources EXISTS # this is embedded comment
             || resourceType EXISTS
        "###,
        "Resources EXISTS", // returns unary expr
        "Resources EXISTS or resourceType exists and configuration exists", // Wait why is this a success, it leaves
                                                                            // 'and configuration exists' as is which will fail
                                                                            // the next parser
    ];

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_disjunction_expr(span);
        println!("{}, {:?}", to_parse, result);
        assert_eq!(result.is_ok(), true);
        struct AssertionsVisitor{}
        impl<'expr> Visitor<'expr> for AssertionsVisitor {
            type Value = ();
            type Error = String;

            fn visit_select(self, _expr: &'expr Expr, value: &'expr QueryExpr) -> Result<Self::Value, Self::Error> {
                let length = value.parts.len();
                assert_eq!(length == 1 || length == 3, true);
                for each in &value.parts {
                    each.accept(AssertionsVisitor{})?;
                }
                Ok(())
            }

            fn visit_binary_operation(self, _expr: &'expr Expr, value: &'expr BinaryExpr) -> Result<Self::Value, Self::Error> {
                assert_eq!(
                    value.operator == BinaryOperator::Or ||
                    value.operator == BinaryOperator::NotEquals, true);
                value.lhs.accept(AssertionsVisitor{})?;
                value.rhs.accept(AssertionsVisitor{})?;
                Ok(())
            }

            fn visit_unary_operation(self, _expr: &'expr Expr, value: &'expr UnaryExpr) -> Result<Self::Value, Self::Error> {
                assert_eq!(value.operator, UnaryOperator::Exists);
                value.expr.accept(AssertionsVisitor{})?;
                Ok(())
            }

            fn visit_string(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
                assert_eq!(
                    value.value == "Resources"       ||
                    value.value == "resourceType"    ||
                    value.value == "configuration"   ||
                    value.value == "Properties"      ||
                    value.value == "Principals"      ||
                    value.value == "*",
                    true
                );
                Ok(())
            }

            fn visit_any(self, expr: &'expr Expr) -> Result<Self::Value, Self::Error> {
                Err(format!("Unexpected expression {:?}", expr))
            }
        }
        let result = result.unwrap().1.accept(AssertionsVisitor{});
        assert_eq!(result.is_ok(), true);
    });

    let failures = [
        "",
        "or Resource EXISTS",
        "Resources EXISTS ||",
    ];

    failures.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_disjunction_expr(span);
        assert_eq!(result.is_err(), true);
        println!("{} {:?}", to_parse, result);
        let pe = match result.unwrap_err() {
            nom::Err::Error(pe) |
            nom::Err::Failure(pe) => pe,
            _ => unreachable!()
        };
        println!("{:?} ", pe);
        assert_eq!(
            pe.get_location().column() == 1 ||
            pe.get_location().column() == "or ".len() + 1 ||
            pe.get_location().column() == "Resources EXISTS ||".len() + 1,
            true
        );
    });
}

#[test]
fn test_and_conjunctions() {
    let success = [
        "Resources EXISTS && Resources.*.Properties.Tags EXISTS",
        r###"Resources EXISTS
             Resources.*.Properties.Tags EXISTS
        "###,
        r###"Resources EXISTS &&
             AWSTemplateVersion == /2010/ and
             Hooks NOT EXISTS
        "###,
        r###"Resources EXISTS
             AWSTemplateVersion == /2010/ or
             Hooks.CodeDeploy EXISTS
        "###,
        r###"(Resources EXISTS && Hooks EXISTS) or
             (Hooks.CodeDeploy EXISTS)
        "###,
        r###"(Resources EXISTS && Hooks EXISTS)"###,
        "(Resources EXISTS or resourceType exists) and configuration exists",
    ];

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_and_conjunction(span);
        let mut my_vec: Vec<String> = Vec::new();
        assert_eq!(result.is_ok(), true, "{} {:?}", to_parse, result);
        #[derive(Debug)]
        struct Unhandled<'e> { expr: &'e Expr }
        struct AssertionVisitor<'a> { vec: &'a mut Vec<String> }
        impl<'expr, 'a> Visitor<'expr> for AssertionVisitor<'a> {
            type Value = ();
            type Error = Unhandled<'expr>;

            fn visit_select(self, _expr: &'expr Expr, value: &'expr QueryExpr) -> Result<Self::Value, Self::Error> {
                for each in &value.parts {
                    each.accept(AssertionVisitor{vec: self.vec})?;
                }
                Ok(())
            }

            fn visit_binary_operation(self, _expr: &'expr Expr, value: &'expr BinaryExpr) -> Result<Self::Value, Self::Error> {
                assert_eq!(
                    value.operator == BinaryOperator::And ||
                    value.operator == BinaryOperator::Equals ||
                    value.operator == BinaryOperator::Or,
                    true
                );
                value.lhs.accept(AssertionVisitor{vec: self.vec})?;
                value.rhs.accept(AssertionVisitor{vec: self.vec})?;
                Ok(())
            }

            fn visit_unary_operation(self, _expr: &'expr Expr, value: &'expr UnaryExpr) -> Result<Self::Value, Self::Error> {
                assert_eq!(value.operator == UnaryOperator::Exists || value.operator == UnaryOperator::NotExists, true);
                value.expr.accept(AssertionVisitor{vec: self.vec})?;
                Ok(())
            }

            fn visit_string(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
                self.vec.push(value.value.clone());
                assert_eq!(
                    value.value == "Resources" ||
                    value.value == "*" ||
                    value.value == "Properties" ||
                    value.value == "Tags" ||
                    value.value == "AWSTemplateVersion" ||
                    value.value == "CodeDeploy" ||
                    value.value == "resourceType" ||
                    value.value == "configuration" ||
                    value.value == "Hooks",
                    true
                );
                Ok(())
            }

            fn visit_regex(self, _expr: &'expr Expr, value: &'expr RegexExpr) -> Result<Self::Value, Self::Error> {
                assert_eq!(value.value, "2010");
                Ok(())
            }


            fn visit_any(self, expr: &'expr Expr) -> Result<Self::Value, Self::Error> {
                Err(Unhandled{ expr })
            }
        }
        let result = result.unwrap().1;
        let asserts = result.accept(AssertionVisitor{vec: &mut my_vec});
        assert_eq!(asserts.is_ok(), true);
        println!("{:?}", my_vec);
    });

    let failures = [
        "",
        "(a == true && b == true) or ", // trailing space matters, TODO fix
        "Resource exist and me not exists", // exist no keyword.
        "Resources != && me exists"
    ];

    failures.iter().for_each(|to_parse|{
        let span = Span::new_extra(*to_parse, "");
        let result = parse_and_conjunction(span);
        println!("{} {:?}", to_parse, result);
        assert_eq!(result.is_err(), true);
    });

}

#[test]
fn test_parse_block_expr() {
    let success = [
        r###"
        let literal = "literal string here"
        let important = ["imp1", "imp2"]

        Resources.*.Properties.Tags[*].Key == %literal
        Resources.%important.Properties.Tags[*].Value == /^Value/
        "###,

        r###"
        let literal = "literal string here"
        let important = ["imp1", "imp2"]

        Resources.*.Properties.Tags[*].Key == %literal
        Resources.%important.Properties.Tags[*].Value == /^Value/
        (Resources.%important.Properties EXISTS &&
         Resources.%important.Properties.Tags NOT EMPTY) OR
        Resources.*.Properties EXISTS
        "###,

        r###"
        let literal = "literal string here"
        let important = ["imp1", "imp2"]

        Resources.*.Properties.Tags[*].Key == %literal
        Resources.%important.Properties.Tags[*].Value == /^Value/ or
        Resources.%important.Properties.Tags[*].Value == /^Other/
        "###,
    ];

    struct ParseBlockAssertions{}
    impl<'expr> Visitor<'expr> for ParseBlockAssertions {
        type Value = ();
        type Error = ();

        fn visit_let(self, _expr: &'expr Expr, value: &'expr LetExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.name == "literal" ||
                value.name == "important",
                true,
                "{} not literal or important",
                value.name
            );
            value.value.accept(ParseBlockAssertions{})?;
            Ok(())
        }

        fn visit_select(self, _expr: &'expr Expr, value: &'expr QueryExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(value.parts.is_empty(), false);
            for each in &value.parts {
                each.accept(ParseBlockAssertions{})?;
            }
            Ok(())
        }

        fn visit_binary_operation(self, _expr: &'expr Expr, value: &'expr BinaryExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.operator == BinaryOperator::Or        ||
                value.operator == BinaryOperator::And       ||
                value.operator == BinaryOperator::Equals,
                true,
                "{:?} not Or, And or Equals",
                value.operator
            );
            value.lhs.accept(ParseBlockAssertions{})?;
            value.rhs.accept(ParseBlockAssertions{})?;
            Ok(())
        }

        fn visit_unary_operation(self, _expr: &'expr Expr, value: &'expr UnaryExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.operator == UnaryOperator::Exists ||
                value.operator == UnaryOperator::NotEmpty,
                true,
                "{:?} not Exist or NotEmpty",
                value.operator
            );
            value.expr.accept(ParseBlockAssertions{})?;
            Ok(())
        }


        fn visit_array(self, _expr: &'expr Expr, value: &'expr ArrayExpr) -> Result<Self::Value, Self::Error> {
            for each in &value.elements {
                each.accept(ParseBlockAssertions{})?;
            }
            Ok(())
        }


        fn visit_string(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.value == "literal string here"    ||
                value.value == "imp1"                   ||
                value.value == "imp2"                   ||
                value.value == "Resources"              ||
                value.value == "*"                      ||
                value.value == "Properties"             ||
                value.value == "Tags"                   ||
                value.value == "Value"                  ||
                value.value == "Key",
                true,
                "{} not expected",
                value.value
            );
            Ok(())
        }

        fn visit_regex(self, _expr: &'expr Expr, value: &'expr RegexExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.value == "^Value"  ||
                value.value == "^Other",
                true
            );
            Ok(())
        }

        fn visit_filter(self, _expr: &'expr Expr, value: &'expr BlockExpr) -> Result<Self::Value, Self::Error> {
            for each in &value.assignments {
                each.accept(ParseBlockAssertions{})?;
            }
            value.clause.accept(ParseBlockAssertions{})?;
            Ok(())
        }

        fn visit_variable_reference(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.value == "important" ||
                value.value == "literal",
                true
            );
            Ok(())
        }

        fn visit_any(self, _expr: &'expr Expr) -> Result<Self::Value, Self::Error> {
            todo!()
        }
    }

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_block_inner_expr(span);
        assert_eq!(result.is_ok(), true, "{} {:?}", to_parse, result);
        let expr = result.unwrap().1;
        let filter = Expr::Filter(Box::new(expr));
        let result = filter.accept(ParseBlockAssertions{});
        assert_eq!(result.is_ok(), true);
    });
}

#[test]
fn test_with_block_queries() {
    let success = [
        r####"
        Resources.* {
            Properties EXISTS or
            Metadata EXISTS
        }
        "####,
        // Filters
        r#"
        Resources[ name | Type == 'AWS::S3::Bucket' && Properties { Tags EXISTS and Metadata EXISTS } ].Properties {
            Tags[*].Key == /^Value/
            Metadata.'aws:cdk' EXISTS
        }
        "#,
        "Resources[*] { Properties EXISTS }",
        "Resources[ Type == 'AWS::S3::Bucket' ].Properties { Tags EXISTS }"
    ];

    struct BlockQueryAssertions{}
    impl<'expr> Visitor<'expr> for BlockQueryAssertions {
        type Value = ();
        type Error = ();

        fn visit_select(self, _expr: &'expr Expr, value: &'expr QueryExpr) -> Result<Self::Value, Self::Error> {
            for each in &value.parts {
                each.accept(BlockQueryAssertions{})?;
            }
            Ok(())
        }

        fn visit_binary_operation(self, _expr: &'expr Expr, value: &'expr BinaryExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.operator == BinaryOperator::Equals    ||
                value.operator == BinaryOperator::And       ||
                value.operator == BinaryOperator::Or,
                true
            );
            value.lhs.accept(BlockQueryAssertions{})?;
            value.rhs.accept(BlockQueryAssertions{})?;
            Ok(())
        }

        fn visit_unary_operation(self, _expr: &'expr Expr, value: &'expr UnaryExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(value.operator, UnaryOperator::Exists);
            value.expr.accept(BlockQueryAssertions{})?;
            Ok(())
        }

        fn visit_string(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.value == "Resources"          ||
                value.value == "AWS::S3::Bucket"    ||
                value.value == "Properties"         ||
                value.value == "Metadata"           ||
                value.value == "Type"               ||
                value.value == "Tags"               ||
                value.value == "*"                  ||
                value.value == "Key"                ||
                value.value == "aws:cdk",
                true,
                "Unexpected {}",
                value.value
            );
            Ok(())
        }

        fn visit_regex(self, _expr: &'expr Expr, value: &'expr RegexExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.value == "^Value",
                true
            );
            Ok(())
        }

        fn visit_filter(self, _expr: &'expr Expr, value: &'expr BlockExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(value.assignments.is_empty(), true, "{:?}", value.assignments);
            value.clause.accept(BlockQueryAssertions{})?;
            Ok(())
        }


        fn visit_variable(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.value == "name",
                true
            );
            Ok(())
        }

        fn visit_block(self, _expr: &'expr Expr, value: &'expr BlockClauseExpr) -> Result<Self::Value, Self::Error> {
            for each in &value.select.parts {
                each.accept(BlockQueryAssertions{})?;
            }
            assert_eq!(value.block.assignments.is_empty(), true, "{:?}", value.block.assignments);
            value.block.clause.accept(BlockQueryAssertions{})?;
            Ok(())
        }

        fn visit_any(self, _expr: &'expr Expr) -> Result<Self::Value, Self::Error> {
            todo!()
        }
    }

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_query_block_expr(span);
        assert_eq!(result.is_ok(), true, "{} {:?}", to_parse, result);
        let expr = result.unwrap().1;
        let visitation = expr.accept(BlockQueryAssertions{});
        assert_eq!(visitation.is_ok(), true, "{:?}", visitation);
    });

    let failures = [
        "",
        "Resources.* { Properties", // no operations present
        "Resources.* { Properties { Tags NOT EMPTY }", // no closing '}'
        "Resources[*] { Properties }", // no operation on Properties
        "Resources[*] { Properties { Tags NOT EXISTS && Tags[*] { Key == /^Key/ }",
        "Resources.* { let p = Properties }", // no conjunctions present

    ];

    failures.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_query_block_expr(span);
        println!("{:?}", result);
        assert_eq!(result.is_err(), true);
    });
}

#[test]
fn test_not_prefix() {
    let success = [
        "not Resources EXISTS",
        "!Resources EXISTS",
        " ! Resources EXISTS",
        "not Resources[*] { Properties EXISTS && Metadata EXISTS }",
        "not ((Resources EXISTS && AWSTemplateVersion EXISTS) or (Resources EXISTS &&  Hooks EXISTS))",
        "not Type == 'AWS::S3::Bucket'",
    ];

    struct NotAssertions{}
    impl Visitor<'_> for NotAssertions {
        type Value = ();
        type Error = ();

        fn visit_select(self, _expr: &'_ Expr, value: &'_ QueryExpr) -> Result<Self::Value, Self::Error> {
            for each in &value.parts {
                each.accept(NotAssertions{})?;
            }
            Ok(())
        }

        fn visit_binary_operation(self, _expr: &'_ Expr, value: &'_ BinaryExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.operator == BinaryOperator::Equals ||
                value.operator == BinaryOperator::And    ||
                value.operator == BinaryOperator::Or,
                true,
                "Unexpected binary operation {:?}",
                value.operator
            );
            value.lhs.accept(NotAssertions{})?;
            value.rhs.accept(NotAssertions{})?;
            Ok(())
        }

        fn visit_unary_operation(self, _expr: &'_ Expr, value: &'_ UnaryExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.operator == UnaryOperator::Exists ||
                value.operator == UnaryOperator::Not,
                true
            );
            value.expr.accept(NotAssertions{})?;
            Ok(())
        }

        fn visit_string(self, _expr: &'_ Expr, value: &'_ StringExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.value == "*"                      ||
                value.value == "Metadata"               ||
                value.value == "Resources"              ||
                value.value == "Tags"                   ||
                value.value == "AWSTemplateVersion"     ||
                value.value == "Type"                   ||
                value.value == "AWS::S3::Bucket"        ||
                value.value == "Hooks"                  ||
                value.value == "Properties",
                true,
                "Unexpected {:?}",
                value
            );
            Ok(())
        }

        fn visit_block(self, _expr: &'_ Expr, value: &'_ BlockClauseExpr) -> Result<Self::Value, Self::Error> {
            for each in &value.select.parts {
                each.accept(NotAssertions{})?;
            }
            assert_eq!(value.block.assignments.is_empty(), true, "{:?}", value.block.assignments);
            value.block.clause.accept(NotAssertions{})?;
            Ok(())
        }



        fn visit_any(self, _expr: &'_ Expr) -> Result<Self::Value, Self::Error> {
            todo!()
        }
    }
    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_and_conjunction(span);
        assert_eq!(result.is_ok(), true, "{} {:?}", to_parse, result);
        let expr = result.unwrap().1;
        let result = expr.accept(NotAssertions{});
        assert_eq!(result.is_ok(), true, "{:?}", result);
    });

}

#[test]
fn test_atleast_one() {
    let success = [
        r#"atleast-one Resources[*] {
            Properties EXISTS
            Metadata EXISTS
        }
        "#,
        r#"Resources[ name | Type == 'AWS::DynamoDB::Table' ].Properties {
            atleast-one Tags[*] {
                Key == /^PROD/
                Value == /App/
            } <<TXT
                The DDB table { $name } does not have any PROD App Key { $Tags }
            TXT
        }"#
    ];

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_unary_binary_or_block_expr(span);
        assert_eq!(result.is_ok(), true, "{} {:?}", to_parse, result);
    });

}

#[test]
fn test_rule_clause_expr() {
    let success = [
        "s3_encryption_at_rest",
        r#"s3_encryption_at_rest && ebs_volume_encryption_at_rest"#,
        r#"s3_encryption_at_rest
           ebs_volume_encryption_at_rest"#,
        r#"s3_encryption_at_rest <<s3 not encrypted>>
           ebs_volume_encryption_at_rest"#,
        r#"s3_encryption_at_rest <<s3 not encrypted>> or
           ebs_volume_encryption_at_rest and
           ddb_encryption_at_rest or databases_encryption_at_rest
           "#,
        r#"s3_encryption_at_rest<<s3 not encrypted>>||ebs_volume_encryption_at_rest"#,
        r#"s3_encryption_at_rest<<s3 not encrypted>>||ebs_volume_encryption_at_rest
           Type == 'AWS::S3::Bucket'
        "#,
        r#"check_allowed_types(Resources.*.Type || [])"#
    ];

    struct ExpectationAssertions{}
    impl<'expr> Visitor<'expr> for ExpectationAssertions {
        type Value = ();
        type Error = ();

        fn visit_rule_clause(self, _expr: &'expr Expr, rule_clause: &'expr RuleClauseExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                rule_clause.name == "s3_encryption_at_rest"             ||
                rule_clause.name == "ebs_volume_encryption_at_rest"     ||
                rule_clause.name == "check_allowed_types"               ||
                rule_clause.name == "ddb_encryption_at_rest"            ||
                rule_clause.name == "databases_encryption_at_rest",
                true
            );
            if let Some(message) = &rule_clause.message {
                assert_eq!(message, "s3 not encrypted");
            }

            if let Some(parameters) = &rule_clause.parameters {
                for each in parameters {
                    each.accept(ExpectationAssertions{})?;
                }
            }
            Ok(())
        }

        fn visit_select(self, _expr: &'expr Expr, value: &'expr QueryExpr) -> Result<Self::Value, Self::Error> {
            for each in &value.parts {
                each.accept(ExpectationAssertions{})?;
            }
            Ok(())
        }



        fn visit_binary_operation(self, _expr: &'expr Expr, value: &'expr BinaryExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.operator == BinaryOperator::And ||
                value.operator == BinaryOperator::Or  ||
                value.operator == BinaryOperator::Equals,
                true
            );
            value.lhs.accept(ExpectationAssertions{})?;
            value.rhs.accept(ExpectationAssertions{})?;
            Ok(())
        }

        fn visit_array(self, _expr: &'expr Expr, value: &'expr ArrayExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(value.elements.is_empty(), true);
            Ok(())
        }

        fn visit_string(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
            assert_eq!(
                value.value == "Type"       ||
                value.value == "Resources"  ||
                value.value == "*"          ||
                value.value == "AWS::S3::Bucket",
                true,
                "Unexpected {}",
                value.value
            );
            Ok(())
        }


        fn visit_any(self, _expr: &'expr Expr) -> Result<Self::Value, Self::Error> {
            todo!()
        }
    }
    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_rule_clause_expr(span.clone());
        assert_eq!(result.is_ok(), true, "{} {:?}", to_parse, result);
        let result = parse_and_conjunction(span);
        assert_eq!(result.is_ok(), true, "{} {:?}", to_parse, result);
        let expr = result.unwrap().1;
        let result = expr.accept(ExpectationAssertions{});
        assert_eq!(result.is_ok(), true, "{:?}", result);
    });

    let failures = [
        "",
        "Resource EXISTS",
        "Type == /S3/",
        "not rule",
    ];

    failures.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_rule_clause_expr(span);
        assert_eq!(result.is_err(), true, "{} {:?}", to_parse, result);
    })

}

#[test]
fn test_when_block() {
    let success = [
        r#"when Resources EXISTS {
            let keys = keys Resources
            %keys == /^MyPrefix/
         }"#,
        r#"WHEN %queues !EMPTY { %queues != '*' }"#,
        "when Resources.*.Type in %allowed_types { Resources.*.Properties.Tags !EMPTY }",
        r#"WHEN %version == /2010/ && %types in %allowed_types {
                Resources.*.Properties EXISTS
        }
        "#,
        r#"when no_sqs_queue_dlqs_must_not_exists {
             let qnames = keys %sqs_queues
             let refs = %dlqs.DeadLetterConfig.Arn.'Fn::GetAtt'
             %refs[0] in %qnames
             %refs[1] == "Arn"
          }
 "#,
        r#"when [ let types = Resources.*.Type
                  let version = AWSTemplateVersion
                  %types in Parameters.AllowedTypes && %version == /2020/ ] {
              Resources.*.Properties EXISTS
           }
        "#,
        r#"when [ let resources = Resources.*
                  let names     = keys %resources

                  %names == /^r/
                  %resources {
                    Type in %allowed_types
                    Properties EXISTS
                  }] {
              Resources.*.Properties.Tags EXISTS
           }
        "#,
        r#"when (Resources EXISTS && Resources.* !EMPTY) or (resourceType EXISTS && configuration EXISTS) {
            check_tags(Resources.*.Properties.Tags or configuration.Tags)
        }"#
    ];

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_when_block_expr(span);
        assert_eq!(result.is_ok(), true, "{} {:?}", to_parse, result);
    });
}

#[test]
fn test_rule_expr() {
    let success = [
        r###"rule check_certificate_local_ca_association(ca_references, expected_logical_ids) {
    %ca_references {
        Ref in %expected_logical_ids
            << Ref not associated with an AWS::ACMPCA::CertificateAuthority >>

        or

        'Fn::GetAtt' {
            this[0] in %expected_logical_ids
                << Ref not associated with an AWS::ACMPCA::CertificateAuthority >>

            this[1] == 'Arn'
                << Attribute for must be 'Arn' >>
        }
    }
}"###,


r###"rule check_acm_non_pca_certs when %acm_non_pca_certs not empty {
    %acm_non_pca_certs.Type == 'AWS::CertificateManager::Certificate'
}"###,
        r###"rule check_acm_certs_transparency {
    %certificates[ Type == 'AWS::CertificateManager::Certificate' ] {
        Properties {
            CertificateTransparencyLoggingPreference not exists or
            CertificateTransparencyLoggingPreference == 'ENABLED'
        }
    }
}
        "###,
        r###"rule check_local(references, expected_logical_ids, attribute) when Check.AssertLocal == true {
    %references {
        Ref in %expected_logical_ids ||
        'Fn::GetAtt' {
            this[0] in %expected_logical_ids && this[1] == %attribute
        }
    }
}

        "###
    ];

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_rule_expr(span);
        assert_eq!(result.is_ok(), true, "{} {:?}", to_parse, result);
    });
}

#[test]
fn test_rules_file() {
    let success = [r###"
#
# Constants
#
let certificate_association_resource_types      = [
    'AWS::CertificateManager::Certificate',
    'AWS::ACMPCA::Certificate',
    'AWS::ACMPCA::CertificateAuthorityActivation'
]

#
# NIST controls for this check
#
let nist_controls       = [
    "NIST-800-53-IA-5",
    "NIST-800-53-SC-17"
]

#
# Assignments
#
let acm_private_cas     = Resources[ Type == 'AWS::ACMPCA::CertificateAuthority' ]
let acm_private_cas_ids = keys %acm_private_cas
let certificates        = Resources[ Type in %certificate_association_resource_types ]
let acm_pca_certs       = %certificates[ Properties.CertificateAuthorityArn exists ]
let acm_non_pca_certs   = %certificates[ Properties.CertificateAuthorityArn not exists ]

rule check_local(references, expected_logical_ids, attribute) when Check.AssertLocal == true {
    %references {
        Ref in %expected_logical_ids ||
        'Fn::GetAtt' {
            this[0] in %expected_logical_ids && this[1] == %attribute
        }
    }
}

rule check_acm_non_pca_certs when %acm_non_pca_certs not empty {
    %acm_non_pca_certs.Type == 'AWS::CertificateManager::Certificate'
}

rule check_acm_pca_certs when %acm_pca_certs not empty {
    %acm_private_cas not empty
        << No private ACM PCAs configured in stack to associate >>
    #
    # If there are not ACM PCA in the template, then acm_private_ca_names will not
    # exist to check against
    #
    check_local(
        %acm_pca_certs.Properties.CertificateAuthorityArn,
        %acm_private_ca_ids
    )
}

rule check_acm_certs_transparency {
    %certificates[ Type == 'AWS::CertificateManager::Certificate' ] {
        Properties {
            CertificateTransparencyLoggingPreference not exists or
            CertificateTransparencyLoggingPreference == 'ENABLED'
        }
    }
}
    "###,
        r###"rule deny_kms_key_checks {
    Resources[ key_id | Type == 'AWS::KMS::Key' ].Properties {
        check_kms_key_usage_in_account(KeyPolicy.Statement[*] || KeyPolicy.Statement)
            <<KMS service actions are not DENIED access from outside account explicitly on %key_id>>
        EnableKeyRotation == true
            <<ALL KMS keys must have auto rotation of key enabled %key_id>>
    }
}

rule check_kms_key_usage_in_account(statements) {
    anyone %statements {
        Effect                == 'Deny'
        anyone Resource[*]    == '*'
        anyone Principal[*]   == '*'
        Action              in ['*', 'kms:*']
        Condition.StringNotEquals.'kms:CallerAccount'.Ref == 'AWS::AccountId'
    }
}
"###,
        ];

    success.iter().for_each(|to_parse| {
        let span = Span::new_extra(*to_parse, "");
        let result = parse_rules_file(span, "");
        assert_eq!(result.is_ok(), true, "{:?}", result);
    });
}