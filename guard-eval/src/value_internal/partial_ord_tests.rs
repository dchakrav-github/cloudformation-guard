use super::*;
use guard_lang::Span;

#[test]
fn test_simple_comparisons() {
    let expr = guard_lang::parse_value(Span::new_extra(r#"'simple'"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("simple");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value, expr);

    let expr = guard_lang::parse_value(Span::new_extra(r#"10"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("10");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value, expr);

    let expr = guard_lang::parse_value(Span::new_extra(r#"true"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("true");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value, expr);

    let expr = guard_lang::parse_value(Span::new_extra(r#"10.0"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("10.0");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value, expr);

    let expr = guard_lang::parse_value(Span::new_extra(r#"10.0"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("10.0");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value, expr);

    let expr = guard_lang::parse_value(Span::new_extra(r#"10.0"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("10");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value, expr);

    let expr = guard_lang::parse_value(Span::new_extra(r#"10"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("10.0");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value, expr);

}

#[test]
fn test_simple_collections() {

    let expr = guard_lang::parse_value(Span::new_extra(r#"['simple', 'truth']"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("[simple, truth]");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value, expr);

    let expr = guard_lang::parse_value(Span::new_extra(r#"[10,20,30,]"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("[10,20,30]");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value, expr);

    let expr = guard_lang::parse_value(Span::new_extra(r#"[10,20,30,40]"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("[10,20,30]");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value == expr, false);

    let expr = guard_lang::parse_value(Span::new_extra(r#"[{simple: 'truth', value: 20}]"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from(r#"[{simple: truth, value: 20}]"#);
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value, expr);

}

#[test]
fn test_simple_maps() {

    let expr = guard_lang::parse_value(Span::new_extra(r#"{simple: 'truth', value: 20}"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from(r#"{simple: truth, value: 20}"#);
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value, expr);

    let expr = guard_lang::parse_value(Span::new_extra(r#"{simple_no_match: 'truth', value: 20}"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from(r#"{simple: truth, value: 20}"#);
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value == expr, false);

    let expr = guard_lang::parse_value(Span::new_extra(r#"{simple: 'truth', value: 30}"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from(r#"{simple: truth, value: 20}"#);
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value == expr, false);

    let expr = guard_lang::parse_value(Span::new_extra(r#"{simple: 'truth', value: [20]}"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from(r#"{simple: truth, value: 20}"#);
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value == expr, false);
}

#[test]
fn test_other_binary_operators() {

    let expr = guard_lang::parse_value(Span::new_extra(r#"10"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("10");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value < expr, false);
    assert_eq!(value > expr, false);
    assert_eq!(value <= expr, true);
    assert_eq!(value >= expr, true);

    //
    // Testing cases like security_groups.ports > 100 && security_groups.ports <= 8000
    //
    let value = read_from("[10, 20]");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value < expr, false);
    assert_eq!(value > expr, false);
    assert_eq!(value <= expr, false);
    assert_eq!(value >= expr, true);

    let expr = guard_lang::parse_value(Span::new_extra(r#"true"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("true");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value < expr, false);
    assert_eq!(value > expr, false);
    assert_eq!(value <= expr, true);
    assert_eq!(value >= expr, true);

    let expr = guard_lang::parse_value(Span::new_extra(r#"10.0"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("10.0");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value < expr, false);
    assert_eq!(value > expr, false);
    assert_eq!(value <= expr, true);
    assert_eq!(value >= expr, true);

    let expr = guard_lang::parse_value(Span::new_extra(r#"10.0"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("20");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value < expr, false);
    assert_eq!(value > expr, true);
    assert_eq!(value <= expr, false);
    assert_eq!(value >= expr, true);

    let expr = guard_lang::parse_value(Span::new_extra(r#"10"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("10.0");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value < expr, false);
    assert_eq!(value > expr, false);
    assert_eq!(value <= expr, true);
    assert_eq!(value >= expr, true);

    let expr = guard_lang::parse_value(Span::new_extra(r#"20.0"#, ""));
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let expr = expr.unwrap().1;
    let value = read_from("10");
    assert_eq!(value.is_ok(), true, "{:?}", value);
    let value = value.unwrap();
    assert_eq!(value < expr, true);
    assert_eq!(value > expr, false);
    assert_eq!(value <= expr, true);
    assert_eq!(value >= expr, false);

}
