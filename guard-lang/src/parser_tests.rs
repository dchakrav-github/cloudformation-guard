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
        "Resources[*].Properties.Tags EXISTS",
        "Resources.*.Properties.Tags !EXISTS",
        "Resources[name].Properties.Tags EMPTY",
        "Resources[%buckets].Properties.Tags NOT EMPTY",
        "Resources.%buckets.Properties.Tags NOT EMPTY",
        r#"Resources.
            %buckets.
            Properties.
            Tags NOT EMPTY"#,
        r###"Resources.
            # extract the list of buckets we used
            %buckets.
            Properties.
            # checking if Tags EXISTS and is not EMPTY
            Tags !EMPTY"###,
        r#"not Resources EXISTS"#
    ];

    for (_idx, expr) in success.iter().enumerate() {
        let span = Span::new_extra(expr, "");
        let result = parse_unary_bool_expr(span);
        assert_eq!(result.is_ok(), true);
        let unary = result.unwrap().1;
        struct UnaryVisitor{};
        impl<'expr> Visitor<'expr> for UnaryVisitor {
            type Value = ();
            type Error = String;

            fn visit_unary_operation(self, _expr: &'expr Expr, value: &'expr UnaryExpr) -> Result<Self::Value, Self::Error> {
                if value.operator == UnaryOperator::Not {
                    return value.expr.accept(UnaryVisitor{});
                }
                assert_eq!(value.operator == UnaryOperator::Exists ||
                           value.operator == UnaryOperator::NotExists ||
                           value.operator == UnaryOperator::Empty ||
                           value.operator == UnaryOperator::NotEmpty, true);
                struct ExpectQuery{};
                impl<'expr> Visitor<'expr> for ExpectQuery {
                    type Value = ();
                    type Error = String;

                    fn visit_select(self, _expr: &'expr Expr, value: &'expr QueryExpr) -> Result<Self::Value, Self::Error> {
                        assert_eq!(value.parts.len() == 4 || value.parts.len() == 1, true);
                        struct ExpectedPart{};
                        impl<'expr> Visitor<'expr> for ExpectedPart {
                            type Value = ();
                            type Error = String;

                            fn visit_string(self, _expr: &'expr Expr, value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
                                assert_eq!(
                                    value.value == "Resources" ||
                                    value.value == "*" ||
                                    value.value == "Properties" ||
                                    value.value == "Tags",
                                    true
                                );
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
                                    value.value, "buckets"
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
        assert_eq!(result.is_ok(), true);
    }
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
        let result = inline_expressions(parse_unary_or_binary_expr)(span);
        println!("{}, {:?}", to_parse, result);
        assert_eq!(result.is_ok(), true);
        struct AssertionsVisitor{};
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
        let result = inline_expressions(parse_unary_or_binary_expr)(span);
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
        let result = and_conjunctions(span);
        let mut my_vec: Vec<String> = Vec::new();
        assert_eq!(result.is_ok(), true);
        #[derive(Debug)]
        struct Unhandled<'e> { expr: &'e Expr };
        struct AssertionVisitor<'a> { vec: &'a mut Vec<String> };
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
        "(a == true && b == true) or",
        "Resource exist and me not exists", // exist no keyword.
        "Resources != && me exists"
    ];

    failures.iter().for_each(|to_parse|{
        let span = Span::new_extra(*to_parse, "");
        let result = and_conjunctions(span);
        println!("{} {:?}", to_parse, result);
        assert_eq!(result.is_err(), true);
    });

}