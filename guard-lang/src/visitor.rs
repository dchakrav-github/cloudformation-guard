use super::exprs::*;
use super::Location;

use std::error::Error;
use std::fmt::Formatter;

pub trait Visitor: Sized {

    type Value;
    type Error;

    fn visit_rule(self,
                  expr: &Expr,
                  _rule: &RuleExpr)     -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_let(self,
                 expr: &Expr,
                 _value: &LetExpr)      -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_when(self,
                  expr: &Expr,
                  _value: &WhenExpr)    -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_select(self,
                    expr: &Expr,
                    _value: &QueryExpr) -> Result<Self::Value, Self::Error> {
		self.visit_any(expr)
	}

    fn visit_binary_operation(self,
                              expr: &Expr,
                              _value: &BinaryExpr) -> Result<Self::Value, Self::Error> {
		self.visit_any(expr)
	}

    fn visit_unary_operation(self,
                             expr: &Expr,
                             _value: &UnaryExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_array(self,
                   expr: &Expr,
                   _value: &ArrayExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_map(self,
                 expr: &Expr,
                 _value: &MapExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_null(self,
                  expr: &Expr,
                  _value: &Location) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_string(self,
                    expr: &Expr,
                    _value: &StringExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_regex(self,
                   expr: &Expr,
                   _value: &RegexExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_char(self,
                  expr: &Expr,
                  _value: &CharExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_bool(self,
                  expr: &Expr,
                  _value: &BoolExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_int(self,
                 expr: &Expr,
                 _value: &IntExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_float(self,
                   expr: &Expr,
                   _value: &FloatExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_range_int(self,
                       expr: &Expr,
                       _value: &RangeIntExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_range_float(self,
                         expr: &Expr,
                         _value: &RangeFloatExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_filter(self,
                    expr: &Expr,
                    _value: &BlockExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_variable(self,
                      expr: &Expr,
                      _value: &StringExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_variable_reference(self,
                                expr: &Expr,
                                _value: &StringExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_block(self,
                   expr: &Expr,
                   _value: &BlockClauseExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_any(self, expr: &Expr) -> Result<Self::Value, Self::Error>;


}