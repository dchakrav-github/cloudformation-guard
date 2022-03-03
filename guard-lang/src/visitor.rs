use super::exprs::*;
use super::Location;

pub trait Visitor<'expr>: Sized {

    type Value;
    type Error;

    fn visit_rule(self,
                  expr: &'expr Expr,
                  _rule: &'expr RuleExpr)     -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_let(self,
                 expr: &'expr Expr,
                 _value: &'expr LetExpr)      -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_when(self,
                  expr: &'expr Expr,
                  _value: &'expr WhenExpr)    -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_select(self,
                    expr: &'expr Expr,
                    _value: &'expr QueryExpr) -> Result<Self::Value, Self::Error> {
		self.visit_any(expr)
	}

    fn visit_binary_operation(self,
                              expr: &'expr Expr,
                              _value: &'expr BinaryExpr) -> Result<Self::Value, Self::Error> {
		self.visit_any(expr)
	}

    fn visit_unary_operation(self,
                             expr: &'expr Expr,
                             _value: &'expr UnaryExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_array(self,
                   expr: &'expr Expr,
                   _value: &'expr ArrayExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_map(self,
                 expr: &'expr Expr,
                 _value: &'expr MapExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_null(self,
                  expr: &'expr Expr,
                  _value: &'expr Location) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_string(self,
                    expr: &'expr Expr,
                    _value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_regex(self,
                   expr: &'expr Expr,
                   _value: &'expr RegexExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_char(self,
                  expr: &'expr Expr,
                  _value: &'expr CharExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_bool(self,
                  expr: &'expr Expr,
                  _value: &'expr BoolExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_int(self,
                 expr: &'expr Expr,
                 _value: &'expr IntExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_float(self,
                   expr: &'expr Expr,
                   _value: &'expr FloatExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_range_int(self,
                       expr: &'expr Expr,
                       _value: &'expr RangeIntExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_range_float(self,
                         expr: &'expr Expr,
                         _value: &'expr RangeFloatExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_filter(self,
                    expr: &'expr Expr,
                    _value: &'expr BlockExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_variable(self,
                      expr: &'expr Expr,
                      _value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_variable_reference(self,
                                expr: &'expr Expr,
                                _value: &'expr StringExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_block(self,
                   expr: &'expr Expr,
                   _value: &'expr BlockClauseExpr) -> Result<Self::Value, Self::Error> {
        self.visit_any(expr)
    }

    fn visit_any(self, expr: &'expr Expr) -> Result<Self::Value, Self::Error>;


}