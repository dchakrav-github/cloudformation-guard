//
// Internal Helpers
//

use super::exprs::*;
use std::cmp::Ordering;
use crate::visitor::Visitor;

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
    pub(crate) fn accept<V>(&self, visitor: V) -> Result<V::Value, V::Error>
    where
        V: Visitor
    {
        match self {
            Expr::Rule(value_expr) =>  visitor.visit_rule(self, value_expr),
            Expr::Let(value_expr) => visitor.visit_let(self, value_expr),
            Expr::When(value_expr) => visitor.visit_when(self, value_expr),
            Expr::Select(value_expr) => visitor.visit_select(self, value_expr),
            Expr::BinaryOperation(value_expr) => visitor.visit_binary_operation(self, value_expr),
            Expr::UnaryOperation(value_expr) => visitor.visit_unary_operation(self, value_expr),
            Expr::Array(value_expr) => visitor.visit_array(self, value_expr),
            Expr::Map(value_expr) => visitor.visit_map(self, value_expr),
            Expr::Null(value_expr) => visitor.visit_null(self, value_expr),
            Expr::String(value_expr) => visitor.visit_string(self, value_expr),
            Expr::Regex(value_expr) => visitor.visit_regex(self, value_expr),
            Expr::Char(value_expr) => visitor.visit_char(self, value_expr),
            Expr::Bool(value_expr) => visitor.visit_bool(self, value_expr),
            Expr::Int(value_expr) => visitor.visit_int(self, value_expr),
            Expr::Float(value_expr) => visitor.visit_float(self, value_expr),
            Expr::RangeInt(value_expr) => visitor.visit_range_int(self, value_expr),
            Expr::RangeFloat(value_expr) => visitor.visit_range_float(self, value_expr),
            Expr::Filter(value_expr) => visitor.visit_filter(self, value_expr),
            Expr::Variable(value_expr) => visitor.visit_variable(self, value_expr),
            Expr::VariableReference(value_expr) => visitor.visit_variable_reference(self, value_expr),
            Expr::Block(value_expr) => visitor.visit_block(self, value_expr),
        }

    }
}