// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use cfn_guard;

mod tests {
    use super::*;
    #[test]
    fn test_run_check() {
        let data = String::from(
            r#"
                {
                    "Resources": {
                        "VPC" : {
                            "Type" : "AWS::ApiGateway::Method",
                            "Properties" : {
                                "AuthorizationType" : "10.0.0.0/24"
                            }
                        }
                    }
                }
            "#,
        );
        let rule = "AWS::ApiGateway::Method { Properties.AuthorizationType == \"NONE\"}";
        let mut expected = String::from(
            r#"
            [
              {
                "eval_type": "Rule",
                "context": "default",
                "msg": "DEFAULT MESSAGE(FAIL)",
                "from": null,
                "to": null,
                "status": "FAIL",
                "children": [
                  {
                    "eval_type": "Type",
                    "context": "AWS::ApiGateway::Method",
                    "msg": "DEFAULT MESSAGE(FAIL)",
                    "from": null,
                    "to": null,
                    "status": "FAIL",
                    "children": [
                      {
                        "eval_type": "Filter",
                        "context": "Path=/Resources/VPC,Type=MapElement",
                        "msg": "DEFAULT MESSAGE(PASS)",
                        "from": null,
                        "to": null,
                        "status": "PASS",
                        "children": [
                          {
                            "eval_type": "Conjunction",
                            "context": "cfn_guard::rules::exprs::GuardClause",
                            "msg": "DEFAULT MESSAGE(PASS)",
                            "from": null,
                            "to": null,
                            "status": "PASS",
                            "children": [
                              {
                                "eval_type": "Clause",
                                "context": "Clause(Location[file:, line:1, column:14], Check: Type  EQUALS String(\"AWS::ApiGateway::Method\"))",
                                "msg": "DEFAULT MESSAGE(PASS)",
                                "from": null,
                                "to": null,
                                "status": "PASS",
                                "children": []
                              }
                            ]
                          }
                        ]
                      },
                      {
                        "eval_type": "Type",
                        "context": "AWS::ApiGateway::Method#0(/Resources/VPC)",
                        "msg": "DEFAULT MESSAGE(FAIL)",
                        "from": null,
                        "to": null,
                        "status": "FAIL",
                        "children": [
                          {
                            "eval_type": "Conjunction",
                            "context": "cfn_guard::rules::exprs::GuardClause",
                            "msg": "DEFAULT MESSAGE(FAIL)",
                            "from": null,
                            "to": null,
                            "status": "FAIL",
                            "children": [
                              {
                                "eval_type": "Clause",
                                "context": "Clause(Location[file:lambda, line:1, column:27], Check: Properties.AuthorizationType  EQUALS String(\"NONE\"))",
                                "msg": "DEFAULT MESSAGE(FAIL)",
                                "from": {
                                  "String": [
                                    "/Resources/VPC/Properties/AuthorizationType",
                                    "10.0.0.0/24"
                                  ]
                                },
                                "to": {
                                  "String": [
                                    "lambda/1/27/Clause/",
                                    "NONE"
                                  ]
                                },
                                "status": "FAIL",
                                "children": []
                              }
                            ]
                          }
                        ]
                      }
                    ]
                  }
                ]
              }
            ]
            "#,
            );

        // Remove white spaces from expected and calculated result for easy comparison.
        let mut serialized =   cfn_guard::run_checks(&data, &rule).unwrap();
        let expected = serde_json::from_str::<serde_json::Value>(&expected).ok().unwrap();
        let serialized = serde_json::from_str::<serde_json::Value>(&serialized).ok().unwrap();
        println!("{}", serde_yaml::to_string(&serialized).ok().unwrap());
        println!("{}", serde_yaml::to_string(&expected).ok().unwrap());
        assert_eq!(expected, serialized);
    }

}
