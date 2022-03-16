use crate::{EvaluationError, Value, EvalReporter, Status, DataFiles, DataFile};

use guard_lang::{
    Expr,
    FileExpr,
    Visitor,
    StringExpr,
    RegexExpr,
    CharExpr,
    BoolExpr,
    IntExpr,
    FloatExpr,
    RangeIntExpr,
    RangeFloatExpr,
    BinaryExpr,
    BinaryOperator,
    ArrayExpr,
    MapExpr,
    RuleExpr,
    RuleClauseExpr,
    LetExpr,
    WhenExpr,
    QueryExpr,
    UnaryExpr,
    Location,
    BlockExpr,
    BlockClauseExpr
};

use std::collections::HashMap;
use std::rc::Rc;
use std::convert::TryFrom;

pub fn evaluate<'e, 's>(rule_file: &'s Expr,
                    data: &'s DataFiles,
                    reporter: &'e mut dyn EvalReporter<'s>) -> Result<Status, EvaluationError<'s>>
{
    let mut hierarchy = ScopeHierarchy { scopes: Vec::with_capacity(4) };
    let variable_extractor = ExtractVariableExprs {
        scope: Scope {
            variable_definitions: HashMap::with_capacity(4),
            variables: HashMap::with_capacity(4),
            reporter,
            roots: data
        }
    };

    hierarchy.scopes.push(rule_file.accept(variable_extractor)?);

    //hierarchy.scopes.push(root);
    struct RootContext<'context, 'value, 'report> {
        scope: &'context mut Scope<'value, 'report>,
        hierarchy: &'context mut ScopeHierarchy<'value, 'report>
    }
    impl<'context, 'value, 'report> Visitor<'value> for RootContext<'context, 'value, 'report> {
        type Value = Status;
        type Error = EvaluationError<'value>;

        fn visit_file(self, _expr: &'value Expr, file: &'value FileExpr) -> Result<Self::Value, Self::Error> {

            for each in &file.assignments {
            }
            todo!()
        }


        fn visit_any(self, expr: &'value Expr) -> Result<Self::Value, Self::Error> {
            todo!()
        }
    }
    todo!()
}

struct ScopeHierarchy<'value, 'report> {
    scopes: Vec<Scope<'value, 'report>>
}

impl<'v, 'r> ScopeHierarchy<'v, 'r> {
    fn resolve_variable<'s>(&'s mut self, name: &str) -> &'s ValueType<'v> {
        todo!()
    }

    fn resolve_rule<'s>(&mut self, name: &str) -> Status {
        todo!()
    }
}


#[derive(Debug)]
enum ValueType<'value> {
    SingleValue(&'value Value),
    QueryValues(Vec<&'value Value>),
    LiteralValue(&'value Expr),
    ComputedValue(Value),
}

#[derive(Debug)]
struct Scope<'value, 'report> {
    roots: &'value Vec<DataFile>,
    variable_definitions: HashMap<&'value str, &'value LetExpr>,
    variables: HashMap<&'value str, ValueType<'value>>,
    reporter: &'report mut dyn EvalReporter<'value>
}

struct ExtractVariableExprs<'v, 'r> { scope: Scope<'v, 'r> }
impl<'v, 'r> Visitor<'v> for ExtractVariableExprs<'v, 'r> {
    type Value = Scope<'v, 'r>;
    type Error = EvaluationError<'v>;

    fn visit_file(mut self, _expr: &'v Expr, file: &'v FileExpr) -> Result<Self::Value, Self::Error> {
        for value in &file.assignments {
            self.scope.variable_definitions.insert(&value.name, value);
        }
        Ok(self.scope)
    }

    fn visit_rule(mut self, _expr: &'v Expr, rule: &'v RuleExpr) -> Result<Self::Value, Self::Error> {
        for each in &rule.block.assignments {
            self.scope = each.accept(ExtractVariableExprs {scope: self.scope})?;
        }
        Ok(self.scope)
    }

    fn visit_let(mut self, _expr: &'v Expr, value: &'v LetExpr) -> Result<Self::Value, Self::Error> {
        self.scope.variable_definitions.insert(&value.name, value);
        Ok(self.scope)
    }

    fn visit_when(mut self, _expr: &'v Expr, value: &'v WhenExpr) -> Result<Self::Value, Self::Error> {
        self.scope = value.when.accept(ExtractVariableExprs {scope: self.scope})?;
        for each in &value.block.assignments {
            self.scope = each.accept(ExtractVariableExprs {scope: self.scope})?;
        }
        Ok(self.scope)
    }

    fn visit_filter(mut self, _expr: &'v Expr, value: &'v BlockExpr) -> Result<Self::Value, Self::Error> {
        for each in &value.assignments {
            self.scope = each.accept(ExtractVariableExprs {scope: self.scope})?;
        }
        Ok(self.scope)
    }


    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        Err(EvaluationError::UnexpectedExpr(
            format!("When attempting extract variable assignment statements got unexpected Expr"),
            expr
        ))
    }
}

#[cfg(test)]
mod extract_variable_exprs_tests;

struct AssignHandler<'c, 'v, 'r> {
    hierarchy: &'c mut ScopeHierarchy<'v, 'r>
}

struct CheckValueLiteral{}
impl<'v> Visitor<'v> for CheckValueLiteral {
    type Value = bool;
    type Error = EvaluationError<'v>; // never thrown

    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        match expr {
            Expr::Array(array) => {
                let mut is_all_values = true;
                for each in &array.elements {
                    is_all_values &= each.accept(CheckValueLiteral{})?;
                }
                Ok(is_all_values)
            },

            Expr::Map(map) => {
                let mut is_all_values = true;
                for each in map.entries.values() {
                    is_all_values &= each.accept(CheckValueLiteral{})?;
                }
                Ok(is_all_values)
            }

            Expr::Null(_) |
            Expr::String(_) |
            Expr::Regex(_) |
            Expr::Char(_) |
            Expr::Bool(_) |
            Expr::Int(_) |
            Expr::Float(_) |
            Expr::RangeInt(_) |
            Expr::RangeFloat(_) => Ok(true),

            _ => Ok(false)
        }
    }
}

impl<'c, 'v, 'r> Visitor<'v> for AssignHandler<'c, 'v, 'r> {
    type Value = ValueType<'v>;
    type Error = EvaluationError<'v>;

    fn visit_select(self, expr: &'v Expr, _value: &'v QueryExpr) -> Result<Self::Value, Self::Error> {
        todo!()
    }

    fn visit_binary_operation(self, expr: &'v Expr, value: &'v BinaryExpr) -> Result<Self::Value, Self::Error> {
        if value.operator != BinaryOperator::Or {
            return self.visit_any(expr)
        }
        let lhs = value.lhs.accept(
            AssignHandler{hierarchy: self.hierarchy});
        let rhs = value.rhs.accept(
            AssignHandler{hierarchy: self.hierarchy}
        );
        lhs.map(std::convert::identity).or_else(|_| rhs.map(std::convert::identity))
    }

    fn visit_array(self, expr: &'v Expr, value: &'v ArrayExpr) -> Result<Self::Value, Self::Error> {
        if expr.accept(CheckValueLiteral{})? {
            return literal(expr)
        }
        let mut converted = Vec::with_capacity(value.elements.len());
        for each in &value.elements {
            match each.accept(
                AssignHandler{hierarchy: self.hierarchy})? {
                ValueType::LiteralValue(v) => converted.push(Value::try_from(v)?),
                ValueType::ComputedValue(v) => converted.push(v),
                _ => unreachable!()
            }
        }
        Ok(ValueType::ComputedValue(
            Value::List(converted, expr.get_location().clone())
        ))
    }

    fn visit_map(self, expr: &'v Expr, value: &'v MapExpr) -> Result<Self::Value, Self::Error> {
        if expr.accept(CheckValueLiteral{})? {
            return literal(expr)
        }
        let mut converted = indexmap::IndexMap::new();
        for (key, value) in &value.entries {
            let value =  match value.accept(
                AssignHandler{hierarchy: self.hierarchy})? {
                ValueType::LiteralValue(v) => Value::try_from(v)?,
                ValueType::ComputedValue(v) => v,
                _ => unreachable!()
            };
            converted.insert(key.clone(), value);
        }
        Ok(ValueType::ComputedValue(
            Value::Map(converted, expr.get_location().clone())
        ))
    }


    fn visit_string(self, expr: &'v Expr, _value: &'v StringExpr) -> Result<Self::Value, Self::Error> {
        literal(expr)
    }

    fn visit_regex(self, expr: &'v Expr, _value: &'v RegexExpr) -> Result<Self::Value, Self::Error> {
        literal(expr)
    }

    fn visit_char(self, expr: &'v Expr, _value: &'v CharExpr) -> Result<Self::Value, Self::Error> {
        literal(expr)
    }

    fn visit_bool(self, expr: &'v Expr, _value: &'v BoolExpr) -> Result<Self::Value, Self::Error> {
        literal(expr)
    }

    fn visit_int(self, expr: &'v Expr, _value: &'v IntExpr) -> Result<Self::Value, Self::Error> {
        literal(expr)
    }

    fn visit_float(self, expr: &'v Expr, _value: &'v FloatExpr) -> Result<Self::Value, Self::Error> {
        literal(expr)
    }

    fn visit_range_int(self, expr: &'v Expr, _value: &'v RangeIntExpr) -> Result<Self::Value, Self::Error> {
        literal(expr)
    }

    fn visit_range_float(self, expr: &'v Expr, _value: &'v RangeFloatExpr) -> Result<Self::Value, Self::Error> {
        literal(expr)
    }

    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        Err(EvaluationError::UnexpectedExpr(
            format!("When attempting extract assignment statements got unexpected Expr"),
            expr
        ))
    }
}

fn literal(expr: &Expr) -> Result<ValueType, EvaluationError> {
    Ok(ValueType::LiteralValue(expr))
}

struct QueryHandler<'c, 'v, 'r> {
    scope: &'c mut Scope<'v, 'r>,
    hierarchy: &'c mut ScopeHierarchy<'v, 'r>
}

