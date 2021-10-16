// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::rules::errors::{Error, ErrorKind};
use crate::rules::eval_context::simplifed_json_from_root;
use crate::rules::path_value::PathAwareValue;
use crate::rules::Result;
use std::convert::TryFrom;

pub fn validate_and_return_json(
    data: &str,
    rules: &str,
    parse_output: &bool
) -> Result<String> {
    let input_data = match serde_json::from_str::<serde_json::Value>(&data) {
       Ok(value) => PathAwareValue::try_from(value),
       Err(e) => return Err(Error::new(ErrorKind::ParseError(e.to_string()))),
    };

    let span = crate::rules::parser::Span::new_extra(&rules, "lambda");

    match crate::rules::parser::rules_file(span) {

        Ok(rules) => {
            match input_data {
                Ok(root) => {
                    let mut root_scope = crate::rules::eval_context::root_scope(&rules, &root)?;
                    let mut tracker = crate::rules::eval_context::RecordTracker::new(&mut root_scope);
                    let _status = crate::rules::eval::eval_rules_file(&rules, &mut tracker)?;
                    let event = tracker.extract();

                    if *parse_output {
                        Ok(serde_json::to_string_pretty(&simplifed_json_from_root(
                            &event,
                        )?)?)
                    } else {
                        Ok(serde_json::to_string_pretty(&event)?)
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Err(e) =>  return Err(Error::new(ErrorKind::ParseError(e.to_string()))),
    }
}
