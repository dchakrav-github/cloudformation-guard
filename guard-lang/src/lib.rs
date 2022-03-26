///
///  Guard Language Grammar and Parser
///

mod types;
mod parser;
mod visitor;
mod exprs;
mod internal;

pub use visitor::Visitor;
pub use exprs::*;
pub use types::*;

pub fn parse_rules(rules: &str, file: &str) -> Result<Expr> {
    let span = crate::Span::new_extra(rules, file);
    match crate::parser::parse_rules_file(span, file) {
        Ok((_input, rules)) => Ok(rules),
        Err(e) => match e {
            nom::Err::Failure(e) |
            nom::Err::Error(e) => {
                Err(LangError::ParseError(e))
            },
            nom::Err::Incomplete(_) => {
                Err(LangError::ParseError(
                    ParseError::new(Location::new(0, 0),
                    format!("More input needed"))))
            }
        }
    }
}

pub fn parse_json_value(value: &str, file: &str) -> Result<Expr> {
    let span = crate::Span::new_extra(value, file);
    match crate::parser::parse_value(span) {
        Ok((_input, value)) => Ok(value),
        Err(e) => match e {
            nom::Err::Failure(e) |
            nom::Err::Error(e) => {
                Err(LangError::ParseError(e))
            },
            nom::Err::Incomplete(_) => {
                Err(LangError::ParseError(
                    ParseError::new(Location::new(0, 0),
                                    format!("More input needed"))))
            }
        }
    }
}

pub use parser::{parse_select, parse_value, parse_unary_binary_or_block_expr};


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
