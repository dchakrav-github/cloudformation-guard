use super::exprs::*;
use crate::rules::Result;
use crate::rules::types::RangeType;

pub(crate) trait Visitor {
     type Result;

     fn visit_rule(&mut self, _rule: &RuleExpr)          -> Result<Self::Result>;
     fn visit_let(&mut self, _let: &LetExpr)             -> Result<Self::Result>;
     fn visit_array(&mut self, _array: &ArrayExpr)       -> Result<Self::Result>;

     fn visit_null(&mut self, _null: Expr::Null(_))           -> Result<R> { Ok(R::default()) }
     fn visit_string(&mut self, _str: Expr::String()String, Location) -> Result<R> { Ok(R::default()) }
     fn visit_regex(&mut self, _regex: Expr::Regex(String, Location)) -> Result<R> { Ok(R::default()) }
     fn visit_bool(&mut self, _bool: Expr::Bool(bool, Location)) -> Result<R> { Ok(R::default()) }
     fn visit_int(&mut self, _i64: Expr::Int(i64, Location)) -> Result<R> { Ok(R::default()) }
     fn visit_float(&mut self, _f64: Expr::Float(f64, Location)) -> Result<R> { Ok(R::default()) }
     fn visit_char(&mut self, _char: Expr::Char(char, Location)) -> Result<R> { Ok(R::default()) }
     fn visit_map(&mut self, _map: Expr::Map(indexmap::IndexMap<String, Expr>, Location)) -> Result<R> { Ok(R::default()) }
     fn visit_range_int(&mut self, _range: Expr::RangeInt(RangeType<i64>, Location)) -> Result<R> { Ok(R::default()) }
     fn visit_range_float(&mut self, _range: Expr::RangeFloat(RangeType<f64>, Location)) -> Result<R> { Ok(R::default()) }
     fn visit_range_char(&mut self, _range: Expr::RangeChar(RangeType<char>, Location)) -> Result<R> { Ok(R::default()) }

     fn visit_query(&mut self, _query: Expr::Query(QueryExpr)) -> Result<R> { Ok(R::default()) }
     fn visit_binary_operation(&mut self, _bin: Expr::BinaryOperation(BinaryOperator, BinaryExpr)) -> Result<R> { Ok(R::default()) }
     fn visit_unary_operation(&mut self, uni: Expr::UnaryOperation(UnaryOperator, Expr)) -> Result<R> { Ok(R::default()) }

     fn visit(&mut self, expr: &Expr) -> Result<R> {
          match expr {
              Expr::Rule(RuleExpr) => { self.visit_rule()}
              Expr::Let(LetExpr) => {}
              Expr::Array(ArrayExpr) => {}
              Expr::Null(Location) => {}
              Expr::String(String, Location) => {}
              Expr::Regex(String, Location) => {}
              Expr::Bool(bool, Location) => {}
              Expr::Int(i64, Location) => {}
              Expr::Float(f64, Location) => {}
              Expr::Char(char, Location) => {}
              Expr::Map(indexmap::IndexMap<String, Expr>, Location) => {}
              Expr::RangeInt(RangeType<i64>, Location) => {}
              Expr::RangeFloat(RangeType<f64>, Location) => {}
              Expr::RangeChar(RangeType<char>, Location) => {}
              Expr::Query(QueryExpr) => {}
              Expr::BinaryOperation(BinaryOperator, BinaryExpr) => {}
              Expr::UnaryOperation(UnaryOperator, Expr)

          }
     }
}