use super::*;
use crate::eval::tests_common::NoOpReporter;
use std::path::PathBuf;

#[test]
fn test_simple_query() {
    let value = super::tests_common::get_value();
    assert_eq!(value.is_ok(), true);
    let value = value.unwrap();
    let data_files = vec![DataFile {
        root: value, file: PathBuf::new()
    }];
    let mut reporter = NoOpReporter{};
    let mut hierarchy = ScopeHierarchy {
        reporter: &mut reporter,
        roots: &data_files,
        scopes: Vec::new(),
        completed: Vec::new()
    };

    let query = guard_lang::parse_select(guard_lang::Span::new_extra("Resources.*.Type", ""));
    assert_eq!(query.is_ok(), true, "{:?}", query);
    let query = query.unwrap().1;
    let mut stack = Vec::new();
    let query_handler = QueryHandler{ hierarchy: &mut hierarchy, stack: &mut stack };
    let r = query.accept(query_handler);
    assert_eq!(r.is_ok(), true, "{:?}", r);
    assert_eq!(stack.len(), 6);
    let expected =
        ["AWS::AutoScaling::AutoScalingGroup",
        "AWS::AutoScaling::LaunchConfiguration",
        "AWS::ElasticLoadBalancing::LoadBalancer",
        "AWS::EC2::SecurityGroup",
        "AWS::IAM::InstanceProfile",
        "AWS::IAM::Role",];
    assert_eq!(stack.iter().all(|s| match s {
        ValueType::SingleValue(Value::String(s,_)) => expected.contains(&s.as_str()),
        _ => unreachable!()
    }), true);
}

#[test]
fn test_simple_query_missing_values() {
    let value = r###"Resources: {}"###;
    let value = crate::value_internal::read_from(value).unwrap();
    let data_files = vec![DataFile {
        root: value, file: PathBuf::new()
    }];
    #[derive(Debug)]
    struct Reporter{};
    impl<'v> EvalReporter<'v> for Reporter {
        fn report_missing_value(&mut self, until: ValueType<'v>, data_file_name: &'v str, expr: &'v Expr) -> Result<(), EvaluationError<'v>> {
            assert_eq!(matches!(until, ValueType::SingleValue(Value::Map(..))), true);
            Ok(())
        }

        fn report_mismatch_value_traversal(&mut self, until: ValueType<'v>, data_file_name: &'v str, expr: &'v Expr) -> Result<(), EvaluationError<'v>> {
            todo!()
        }

        fn report_evaluation(&mut self, value: ValueType<'v>, data_file_name: &'v str, expr: &'v Expr, status: Status) -> Result<(), EvaluationError<'v>> {
            todo!()
        }
    }
    let query = guard_lang::parse_select(guard_lang::Span::new_extra("Resources.*.Type", ""));
    assert_eq!(query.is_ok(), true, "{:?}", query);
    let query = query.unwrap().1;
    let mut stack = Vec::new();
    let mut reporter = Reporter{};
    let mut hierarchy = ScopeHierarchy {
        reporter: &mut reporter,
        roots: &data_files,
        scopes: Vec::new(),
        completed: Vec::new()
    };

    let query_handler = QueryHandler{ hierarchy: &mut hierarchy, stack: &mut stack };
    let r = query.accept(query_handler);
    assert_eq!(r.is_ok(), true);
    let r = r.unwrap();
    assert_eq!(r, false);
    assert_eq!(stack.is_empty(), true);

}
