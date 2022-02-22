use super::exprs::*;
use super::Location;

use std::error::Error;
use std::fmt::Formatter;

#[derive(Debug)]
pub struct UnhandledExprError<'expr> {
    expr: &'expr str
}

impl<'expr> std::error::Error for UnhandledExprError<'expr> {}
impl<'expr> std::fmt::Display for UnhandledExprError<'expr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub trait Visitor<E: std::error::Error> {
    type Output;
    type Error: std::error::Error;

    fn visit(&mut self, expr: &Expr) -> Result<Self::Output, Self::Error> {
        todo!()
//        match expr {
//            Expr::Rule(value_expr) => self.visit_rule(value_expr),
//            Expr::Let(value_expr) => self.visit_let(value_expr),
//            Expr::When(value_expr) => self.visit_when(value_expr),
//            Expr::Type(value_expr) => self.visit_type(value_expr),
//            Expr::Select(value_expr) => self.visit_query(value_expr),
//            Expr::BinaryOperation(value_expr) => self.visit_binary_operation(value_expr),
//            Expr::UnaryOperation(value_expr) => self.visit_unary_operation(value_expr),
//            Expr::Array(value_expr) => self.visit_array(value_expr),
//            Expr::Map(value_expr) => self.visit_map(value_expr),
//            Expr::Null(value_expr) => self.visit_null(value_expr),
//            Expr::String(value_expr) => self.visit_string(value_expr),
//            Expr::Regex(value_expr) => self.visit_regex(value_expr),
//            Expr::Char(value_expr) => self.visit_char(value_expr),
//            Expr::Bool(value_expr) => self.visit_bool(value_expr),
//            Expr::Int(value_expr) => self.visit_int(value_expr),
//            Expr::Float(value_expr) => self.visit_float(value_expr),
//            Expr::RangeInt(value_expr) => self.visit_range_int(value_expr),
//            Expr::RangeFloat(value_expr) => self.visit_range_float(value_expr),
//        }
    }
}