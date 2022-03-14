use crate::EvaluationError;
use std::path::PathBuf;
use std::io::{BufReader, Read};
use anyhow::Context;
use std::convert::TryFrom;

#[test]
fn test_json_yaml_parsing() -> anyhow::Result<()> {
    let file = PathBuf::from("./test-resources");
    let directory = walkdir::WalkDir::new(file);
    for each in directory.follow_links(true) {
        if let Ok(entry) = each {
            if entry.path().is_file() {
                let name = entry.path().file_name().map_or("", |s| s.to_str().unwrap());
                if  name.ends_with(".yaml")    ||
                    name.ends_with(".yml")     ||
                    name.ends_with(".json")    ||
                    name.ends_with(".jsn")     ||
                    name.ends_with(".template") {
                    let mut file_content = String::new();
                    let file = std::fs::File::open(entry.path()).context(
                        format!("Unable to open file {:?}", entry.path())
                    )?;
                    let mut buf_reader = BufReader::new(file);
                    buf_reader.read_to_string(&mut file_content)?;
                    let value = super::read_from(&file_content)
                        .context(format!("Unable to parse file {:?}", entry.path()))?;
                    match value {
                        crate::Value::Map(..) => {},
                        _ => unreachable!()
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn test_json_parsing() {
    let template = r#"
    {
  "AWSTemplateFormatVersion" : "2010-09-09",

  "Description" : "AWS CloudFormation Sample Template DynamoDB_Table: This template demonstrates the creation of a DynamoDB table.  **WARNING** This template creates an Amazon DynamoDB table. You will be billed for the AWS resources used if you create a stack from this template.",

  "Parameters" : {
    "HashKeyElementName" : {
      "Description" : "HashType PrimaryKey Name",
      "Type" : "String",
      "AllowedPattern" : "[a-zA-Z0-9]*",
      "MinLength": "1",
      "MaxLength": "2048",
      "ConstraintDescription" : "must contain only alphanumberic characters"
    },

    "HashKeyElementType" : {
      "Description" : "HashType PrimaryKey Type",
      "Type" : "String",
      "Default" : "S",
      "AllowedPattern" : "[S|N]",
      "MinLength": "1",
      "MaxLength": "1",
      "ConstraintDescription" : "must be either S or N"
    },

    "ReadCapacityUnits" : {
      "Description" : "Provisioned read throughput",
      "Type" : "Number",
      "Default" : "5",
      "MinValue": "5",
      "MaxValue": "10000",
      "ConstraintDescription" : "must be between 5 and 10000"
    },

    "WriteCapacityUnits" : {
      "Description" : "Provisioned write throughput",
      "Type" : "Number",
      "Default" : "10",
      "MinValue": "5",
      "MaxValue": "10000",
      "ConstraintDescription" : "must be between 5 and 10000"
    }
  },

  "Resources" : {
    "myDynamoDBTable" : {
      "Type" :"AWS::DynamoDB::Table",
      "Properties" : {
        "AttributeDefinitions": [ {
          "AttributeName" : {"Ref" : "HashKeyElementName"},
          "AttributeType" : {"Ref" : "HashKeyElementType"}
        } ],
        "KeySchema": [
          { "AttributeName": {"Ref" : "HashKeyElementName"}, "KeyType": "HASH" }
        ],
        "ProvisionedThroughput" : {
          "ReadCapacityUnits" : {"Ref" : "ReadCapacityUnits"},
          "WriteCapacityUnits" : {"Ref" : "WriteCapacityUnits"}
        }
      }
    }
  },

  "Outputs" : {
    "TableName" : {
      "Value" : {"Ref" : "myDynamoDBTable"},
      "Description" : "Table name of the newly created DynamoDB table"
    }
  }
}"#;

    let expr = guard_lang::parse_json_value(template, "");
    assert_eq!(expr.is_ok(), true, "{:?}", expr);
    let value = super::Value::try_from(expr.unwrap());
    assert_eq!(value.is_ok(), true);
    println!("{:?}", value.unwrap());

}