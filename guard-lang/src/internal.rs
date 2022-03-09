//
// Internal Helpers
//

use super::exprs::*;
use std::cmp::Ordering;
use crate::Location;

impl PartialEq<i64> for IntExpr {
    fn eq(&self, other: &i64) -> bool {
        self.value() == *other
    }
}

impl PartialEq<f64> for FloatExpr {
    fn eq(&self, other: &f64) -> bool {
        match self.value().partial_cmp(other) {
            Some(Ordering::Equal) => true,
            _ => false
        }
    }
}

impl Expr {
    pub(crate) fn get_location(&self) -> &Location {
        match self {
            Expr::File(value_expr) => &value_expr.location,
            Expr::Rule(value_expr) =>  &value_expr.location,
            Expr::Let(value_expr) => &value_expr.location,
            Expr::When(value_expr) => &value_expr.location,
            Expr::Select(value_expr) => &value_expr.location,
            Expr::BinaryOperation(value_expr) => &value_expr.location,
            Expr::UnaryOperation(value_expr) => &value_expr.location,
            Expr::Array(value_expr) => &value_expr.location,
            Expr::Map(value_expr) => &value_expr.location,
            Expr::Null(value_expr) => &value_expr,
            Expr::String(value_expr) => &value_expr.location,
            Expr::Regex(value_expr) => &value_expr.location,
            Expr::Char(value_expr) => &value_expr.location,
            Expr::Bool(value_expr) => &value_expr.location,
            Expr::Int(value_expr) => &value_expr.location,
            Expr::Float(value_expr) => &value_expr.location,
            Expr::RangeInt(value_expr) => &value_expr.location,
            Expr::RangeFloat(value_expr) => &value_expr.location,
            Expr::Filter(value_expr) => &value_expr.location,
            Expr::Variable(value_expr) => &value_expr.location,
            Expr::VariableReference(value_expr) => &value_expr.location,
            Expr::Block(value_expr) => &value_expr.location,
        }
    }
}
