use super::*;

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