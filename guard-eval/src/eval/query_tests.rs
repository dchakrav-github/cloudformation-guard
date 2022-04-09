use super::*;
use crate::eval::tests_common::NoOpReporter;
use std::path::PathBuf;
use guard_lang::Span;
use crate::Comparison;

#[test]
fn test_simple_query() {
    let value = super::tests_common::get_value();
    assert_eq!(value.is_ok(), true);
    let value = value.unwrap();
    let mut reporter = NoOpReporter{};
    let mut hierarchy = RootScopeHierarchy {
        reporter: &mut reporter,
        roots: &value,
        scopes: Vec::new(),
        completed: Vec::new()
    };

    let query = guard_lang::parse_select(guard_lang::Span::new_extra("Resources.*.Type", ""));
    assert_eq!(query.is_ok(), true, "{:?}", query);
    let query = query.unwrap().1;
    let mut stack = Vec::new();
    let query_handler = QueryHandler{ hierarchy: &mut hierarchy, stack };
    let r = query.accept(query_handler);
    assert_eq!(r.is_ok(), true, "{:?}", r);
    stack = r.unwrap();
    assert_eq!(stack.len(), 6);
    let expected =
        ["AWS::AutoScaling::AutoScalingGroup",
        "AWS::AutoScaling::LaunchConfiguration",
        "AWS::ElasticLoadBalancing::LoadBalancer",
        "AWS::EC2::SecurityGroup",
        "AWS::IAM::InstanceProfile",
        "AWS::IAM::Role",];
    assert_eq!(stack.iter().all(|s| match s {
        ValueType::DataValue(Value::String(s, _)) => expected.contains(&s.as_str()),
        _ => unreachable!()
    }), true);

    let mut scope = Scope {
        variables: HashMap::new(),
        variable_definitions: HashMap::new(),
    };

    let expr = guard_lang::parse_value(
        Span::new_extra(r#"[ /Web/, /Launch/ ]    "#, "")).unwrap().1;
    scope.variables.insert("lookup", vec![ValueType::LiteralValue(&expr)]);
    hierarchy.scopes.insert(0, scope);

    let query = guard_lang::parse_select(guard_lang::Span::new_extra("Resources.%lookup", ""))
        .unwrap().1;
    stack.clear();
    let r = query.accept(
        QueryHandler{ hierarchy: &mut hierarchy, stack});
    assert_eq!(r.is_ok(), true, "{:?}", r);
    stack = r.unwrap();
    assert_eq!(stack.len(), 3);

}

#[test]
fn test_simple_query_missing_values() {
    let value = r###"Resources: {}"###;
    let value = crate::value_internal::read_from(value).unwrap();
    #[derive(Debug)]
    struct Reporter{}
    impl<'v> EvalReporter<'v> for Reporter {
        fn report_missing_value(&mut self, until: ValueType<'v>, data_file_name: &'v str, expr: &'v Expr) -> Result<(), std::io::Error> {
            assert_eq!(matches!(until, ValueType::DataValue(Value::Map(..))), true);
            Ok(())
        }

        fn report_mismatch_value_traversal(&mut self, until: ValueType<'v>, data_file_name: &'v str, expr: &'v Expr) -> Result<(), std::io::Error> {
            todo!()
        }

        fn report_evaluation(&mut self, status: Status, comparison: Comparison<'v>, data_file: &'v str, expr: &'v Expr) -> Result<(), std::io::Error> {
            todo!()
        }
    }
    let query = guard_lang::parse_select(guard_lang::Span::new_extra("Resources.*.Type", ""));
    assert_eq!(query.is_ok(), true, "{:?}", query);
    let query = query.unwrap().1;
    let mut stack = Vec::new();
    let mut reporter = Reporter{};
    let mut hierarchy = RootScopeHierarchy {
        reporter: &mut reporter,
        roots: &value,
        scopes: Vec::new(),
        completed: Vec::new()
    };

    let query_handler = QueryHandler{ hierarchy: &mut hierarchy, stack };
    let r = query.accept(query_handler);
    assert_eq!(r.is_ok(), true);
    stack = r.unwrap();
    assert_eq!(stack.is_empty(), true);

}

#[test]
fn test_binary_comparison() {
    let value = r###"
    Resources:
        iam:
            Type: AWS::IAM::Role
            Properties:
                Action: Allow
                Principal: '*'
                Resource: '*'
        iam2:
            Type: AWS::IAM::Role
            Properties:
                Action: Allow
                Principal:
                  AWS: s3.amazonaws.com
                Resource: '*'
    "###;
    let value = crate::value_internal::read_from(value).unwrap();
    let binary_cmp = r#"Resources.*.Type == /Role$/"#;
    let cmp = guard_lang::parse_unary_binary_or_block_expr(Span::new_extra(binary_cmp, ""));
    assert_eq!(cmp.is_ok(), true, "{:?}", cmp);
    let cmp = cmp.unwrap().1;
    let mut reporter = NoOpReporter{};
    let mut hierarchy = RootScopeHierarchy {
        reporter: &mut reporter,
        roots: &value,
        scopes: Vec::new(),
        completed: Vec::new()
    };

    let binop = BinaryOperationsHandler {
        hierarchy: &mut hierarchy,
        stack: Vec::new()
    };
    let result = cmp.accept(binop);
    assert_eq!(result.is_ok(), true, "{:?}", result);
    assert_eq!(result.unwrap(), true);

    let binary_cmp = r#"Resources.*.Properties.Principal[*] != '*'"#;
    let cmp = guard_lang::parse_unary_binary_or_block_expr(Span::new_extra(binary_cmp, ""));
    assert_eq!(cmp.is_ok(), true, "{:?}", cmp);
    let cmp = cmp.unwrap().1;
    let mut reporter = NoOpReporter{};
    let mut hierarchy = RootScopeHierarchy {
        reporter: &mut reporter,
        roots: &value,
        scopes: Vec::new(),
        completed: Vec::new()
    };

    let binop = BinaryOperationsHandler {
        hierarchy: &mut hierarchy,
        stack: Vec::new()
    };
    let result = cmp.accept(binop);
    assert_eq!(result.is_ok(), true, "{:?}", result);
    assert_eq!(result.unwrap(), false);
}

#[test]
fn test_binary_operators() {
    let value = r###"
    PARAMETERS:
       allowed_ports: [600, 800]
    Resources:
      sgs:
        Type: AWS::EC2::SecurityGroup
        Properties:
          ingress:
            - From: 10
              To: 100
              Cidr: 0.0.0.0/0
    "###;

    let value = crate::value_internal::read_from(value).unwrap();
    let expr = guard_lang::parse_select(Span::new_extra("PARAMETERS.AllowedPorts", "")).unwrap().1;
    let mut reporter = NoOpReporter{};
    let mut hierarchy = RootScopeHierarchy {
        reporter: &mut reporter,
        roots: &value,
        scopes: Vec::new(),
        completed: Vec::new()
    };
    let values = expr.accept(QueryHandler{stack: vec![ValueType::DataValue(&value)], hierarchy: &mut hierarchy});
    assert_eq!(values.is_ok(), true, "{:?}", values);
    let values = values.unwrap();
    assert_eq!(values.len(), 1);
    assert_eq!(matches!(values.get(0), Some(ValueType::DataValue(Value::List(..)))), true);

}
