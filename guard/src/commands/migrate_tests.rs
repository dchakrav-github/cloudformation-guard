use super::*;
use crate::migrate::parser::{Clause, BaseRule, PropertyComparison, CmpOperator, OldGuardValues, ConditionalRule};
use crate::rules::values::Value;
use crate::rules::parser::rules_file;

#[test]
fn test_migrate_rules() -> Result<()> {
    let old_ruleset = String::from(
        r#"
        AWS::S3::Bucket WHEN .property.path.* IN ["a", "b", "c"] CHECK BucketName.Encryption == "Enabled"
        let my_variable = true

        # this is a comment
        AWS::EC2::Instance InstanceType == "m2.large"
        AWS::S3::Bucket BucketName == /Encrypted/ << Buckets should be encrypted, or instance type large, or property path in a,b,c |OR| AWS::EC2::Instance WHEN InstanceType == "m2.large" CHECK .DeletionPolicy == Retain |OR| AWS::S3::Bucket Properties.Foo.Bar == 2 << this must equal 2"#,
    );
    let rule_lines = parse_rules_file(&old_ruleset, &String::from("test-file")).unwrap();
    let result = migrate_rules(&rule_lines).unwrap();
    let span = crate::rules::parser::Span::new_extra(&result, "");
    rules_file(span)?;
    Ok(())
}