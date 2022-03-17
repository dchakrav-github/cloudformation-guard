use guard_lang::{
    parse_rules,
    Expr,
    RuleExpr,
    Visitor,
    FileExpr,
    RuleClauseExpr,
    LetExpr,
    WhenExpr,
    QueryExpr,
    BinaryExpr,
    UnaryExpr,
    ArrayExpr,
    MapExpr,
    Location,
    StringExpr,
    RegexExpr,
    CharExpr,
    BoolExpr,
    IntExpr,
    FloatExpr,
    RangeIntExpr,
    RangeFloatExpr,
    BlockExpr,
    BlockClauseExpr,
    UnaryOperator
};

use crate::{EvalReporter, Value, EvaluationError, Status};
use std::collections::{HashMap, HashSet};
use crate::eval::CheckValueLiteral;
use guard_lang::Expr::UnaryOperation;

#[test]
fn test_rule_file_extraction() {
    let rules = r###"
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
let computed            = {
    all_tags: keys Resources[ Properties.Tags NOT EMPTY ],
    others: "others"
}
let non_computed_literal    = {
  all: "values", are: "pure",
}

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

rule deny_kms_key_checks {
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
}"###;

    let data_files: super::DataFiles = Vec::with_capacity(1);
    #[derive(Debug)]
    struct Reporter{};
    impl<'value> EvalReporter<'value> for Reporter {
        fn report_missing_value(&mut self,
                                _until: &'value Value,
                                _data_file_name: &'value str,
                                _expr: &'value Expr) -> Result<(), EvaluationError> {
            todo!()
        }

        fn report_evaluation(&mut self,
                             _value: &'value Value,
                             _data_file_name: &'value str,
                             _status: Status) -> Result<(), EvaluationError> {
            todo!()
        }
    }
    let mut reporter = Reporter{};
    let rules_file = parse_rules(rules, "");
    assert_eq!(rules_file.is_ok(), true, "{:?}", rules_file);
    let rules_file = rules_file.unwrap();
    let variable_names = [
        "certificate_association_resource_types",
        "nist_controls",
        "acm_private_cas",
        "acm_private_cas_ids",
        "certificates",
        "acm_pca_certs",
        "computed",
        "non_computed_literal",
        "acm_non_pca_certs",];
    let extract_vars = super::ExtractVariableExprs {
        scope: super::Scope {
            roots: &data_files,
            reporter: &mut reporter,
            variables: HashMap::new(),
            variable_definitions: HashMap::with_capacity(variable_names.len()),
        }
    };

    let scope = rules_file.accept(extract_vars);
    assert_eq!(scope.is_ok(), true, "{:?}", scope);
    let scope = scope.unwrap();
    assert_eq!(scope.variable_definitions.is_empty(), false,
               "Expected variables {:?} {:?}", variable_names, scope);
    let keys: Vec<&str> = scope.variable_definitions.keys().map(|k| *k).collect();
    assert_eq!(keys.iter().all(|s| variable_names.contains(s)), true,
        "{:?} {:?}", variable_names, keys);

    let let_expr = *(scope.variable_definitions.get(
        "certificate_association_resource_types").unwrap());

    assert_eq!(let_expr.value.accept(CheckValueLiteral{}).unwrap(), true);

    let let_expr = *(scope.variable_definitions.get(
        "computed").unwrap());
    assert_eq!(let_expr.value.accept(CheckValueLiteral{}).unwrap(), false);
    struct CheckComputed{};
    impl Visitor<'_> for CheckComputed {
        type Value = ();
        type Error = ();

        fn visit_unary_operation(self, _expr: &'_ Expr, value: &'_ UnaryExpr)
            -> Result<Self::Value, Self::Error>
        {
            assert_eq!(value.operator, UnaryOperator::Keys);
            Ok(())
        }

        fn visit_map(self, _expr: &'_ Expr, value: &'_ MapExpr) -> Result<Self::Value, Self::Error> {
            if let Some(expr) = value.entries.get("all_tags") {
                expr.accept(CheckComputed{})?;
            }
            Ok(())
        }



        fn visit_any(self, expr: &'_ Expr) -> Result<Self::Value, Self::Error> {
            todo!()
        }
    }
    let expr_checking = let_expr.value.accept(CheckComputed{});
    assert_eq!(expr_checking.is_ok(), true);

    let let_expr = *(scope.variable_definitions.get(
        "non_computed_literal").unwrap());
    assert_eq!(let_expr.value.accept(CheckValueLiteral{}).unwrap(), true);

    if let Expr::File(file) = &rules_file {
        assert_eq!(file.rules.len(), 6, "Expected 6, {}", file.rules.len());
        let (named_rules, normal) : (Vec<&RuleExpr>, Vec<&RuleExpr>) =
            file.rules.iter().partition(|r| r.parameters.is_some());
        assert_eq!(named_rules.len(), 2);
        assert_eq!(normal.len(), 4);
        let expected_named_rules = ["check_kms_key_usage_in_account", "check_local"].iter()
            .map(|s| *s)
            .collect::<HashSet<&str>>();
        assert_eq!(named_rules.iter().map(|r| r.name.as_str()).collect::<HashSet<&str>>(),
                   expected_named_rules);
        let expected = [
            "check_acm_non_pca_certs",
            "deny_kms_key_checks",
            "check_acm_certs_transparency",
            "check_acm_pca_certs",
            "check_acm_non_pca_certs"].iter().map(|s| *s).collect::<HashSet<&str>>();
        assert_eq!(normal.iter().map(|r| r.name.as_str()).collect::<HashSet<&str>>(),
            expected);

    }

}