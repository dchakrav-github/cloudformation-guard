use super::exprs::*;
use crate::rules::types::Result;
use crate::rules::errors::Error;
use crate::rules::errors::ErrorKind::IncompatibleError;

pub trait Visitor {
    type Result;

    fn visit_rule(&mut self, _rule: &RuleExpr)          -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("RuleExpr is unexpected".to_string())))
    }

    fn visit_let(&mut self, _let: &LetExpr)             -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("LetExpr is unexpected".to_string())))
    }

    fn visit_when(&mut self, _when: &WhenExpr) -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("When expr is unexpected".to_string())))
    }

    fn visit_type(&mut self, _type: &TypeExpr) -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("Type expr is unexpected".to_string())))
    }

    fn visit_array(&mut self, _array: &ArrayExpr)       -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("ArrayExpr is unexpected".to_string())))
    }

    fn visit_null(&mut self, _null: &Location)           -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("Null value is unexpected".to_string())))
    }

    fn visit_string(&mut self, _str: &StringExpr) -> Result<Self::Result> {
         Err(Error::new(IncompatibleError("String value is unexpected".to_string())))
     }

     fn visit_regex(&mut self, _regex: &RegexExpr) -> Result<Self::Result> {
         Err(Error::new(IncompatibleError("Regex expression is unexpected".to_string())))
     }

    fn visit_bool(&mut self, _bool: &BoolExpr) -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("Bool value is unexpected".to_string())))
    }

    fn visit_int(&mut self, _i64: &IntExpr) -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("Integer value is unexpected".to_string())))
    }

    fn visit_float(&mut self, _f64: &FloatExpr) -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("Float value is unexpected".to_string())))
    }

    fn visit_char(&mut self, _char: &CharExpr) -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("Char value is unexpected".to_string())))
    }

    fn visit_map(&mut self, _map: &MapExpr) -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("Map value is unexpected".to_string())))
    }

    fn visit_range_int(&mut self, _range: &RangeIntExpr) -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("Range interval for Intergers is unexpected".to_string())))
    }

    fn visit_range_float(&mut self, _range: &RangeFloatExpr) -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("Range interval for floats is unexpected".to_string())))
    }

    fn visit_query(&mut self, _query: &QueryExpr) -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("unexpected".to_string())))
    }

    fn visit_binary_operation(&mut self, _bin_op: &BinaryExpr) -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("Binary operation is unexpected".to_string())))
    }

    fn visit_unary_operation(&mut self, _uni_op: &UnaryExpr) -> Result<Self::Result> {
        Err(Error::new(IncompatibleError("unexpected".to_string())))
    }

    fn visit(&mut self, expr: &Expr) -> Result<Self::Result> {
        match expr {
            Expr::Rule(value_expr) => self.visit_rule(value_expr),
            Expr::Let(value_expr) => self.visit_let(value_expr),
            Expr::When(value_expr) => self.visit_when(value_expr),
            Expr::Type(value_expr) => self.visit_type(value_expr),
            Expr::Query(value_expr) => self.visit_query(value_expr),
            Expr::BinaryOperation(value_expr) => self.visit_binary_operation(value_expr),
            Expr::UnaryOperation(value_expr) => self.visit_unary_operation(value_expr),
            Expr::Array(value_expr) => self.visit_array(value_expr),
            Expr::Map(value_expr) => self.visit_map(value_expr),
            Expr::Null(value_expr) => self.visit_null(value_expr),
            Expr::String(value_expr) => self.visit_string(value_expr),
            Expr::Regex(value_expr) => self.visit_regex(value_expr),
            Expr::Char(value_expr) => self.visit_char(value_expr),
            Expr::Bool(value_expr) => self.visit_bool(value_expr),
            Expr::Int(value_expr) => self.visit_int(value_expr),
            Expr::Float(value_expr) => self.visit_float(value_expr),
            Expr::RangeInt(value_expr) => self.visit_range_int(value_expr),
            Expr::RangeFloat(value_expr) => self.visit_range_float(value_expr),
        }
    }
}