use crate::{
    EvaluationError,
    Value,
    EvalReporter,
    Status,
    DataFiles,
    DataFile,
    ValueType
};

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
use std::collections::hash_map::Entry;
use std::cell::RefCell;
use std::process::exit;

pub fn evaluate<'e, 's>(rule_file: &'s Expr,
                    data: &'s DataFiles,
                    reporter: &'e mut dyn EvalReporter<'s>) -> Result<Status, EvaluationError<'s>>
{
    let mut hierarchy = ScopeHierarchy {
        roots: data,
        scopes: Vec::with_capacity(4),
        completed: Vec::with_capacity(4),
        reporter
    };
    let variable_extractor = ExtractVariableExprs {
        scope: Scope {
            variable_definitions: HashMap::with_capacity(4),
            variables: HashMap::with_capacity(4),
        }
    };

    hierarchy.add_scope(rule_file.accept(variable_extractor)?);

    //hierarchy.scopes.push(root);
    struct RootContext<'context, 'value, 'report> {
        scope: &'context mut Scope<'value>,
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
    roots: &'value Vec<DataFile>,
    scopes: Vec<Scope<'value>>,
    completed: Vec<Scope<'value>>,
    reporter: &'report mut dyn EvalReporter<'value>
}

impl<'v, 'r> ScopeHierarchy<'v, 'r> {
    fn get_data_roots(&self) -> &'v DataFiles {
        self.roots
    }

    fn get_resolved_variable(&self, name: &str) -> Option<ValueType<'v>> {
        for each_scope in &self.scopes {
            match each_scope.variables.get(name) {
                Some(v) => return Some(v.clone()),
                None => continue
            }
        }
        None
    }

    fn get_variable_expression(&self, name: &str) -> Result<(usize, &'v LetExpr), EvaluationError<'v>> {
        for (idx, each_scope) in self.scopes.iter().enumerate() {
            match each_scope.variable_definitions.get(name) {
                Some(v) => return Ok((idx, *v)),
                None => continue
            }
        }
        return Err(EvaluationError::ComputationError(
            format!("Variable {} could not resolved in any scope", name)))
    }

    fn add_variable_resolution(&mut self, scope_idx: usize, name: &'v str, value: ValueType<'v>) {
        self.scopes.get_mut(scope_idx).unwrap().variables.insert(name, value);
    }

    fn resolve_rule<'s>(&mut self, name: &str) -> Status {
        todo!()
    }

    fn add_scope(&mut self, scope: Scope<'v>) {
        self.scopes.insert(0, scope);
    }

    fn drop_scope(&mut self) {
        if !self.scopes.is_empty() {
            self.completed.insert(0, self.scopes.remove(0));
        }
    }
}


#[derive(Debug)]
struct Scope<'value> {
    variable_definitions: HashMap<&'value str, &'value LetExpr>,
    variables: HashMap<&'value str, ValueType<'value>>,
}

struct ExtractVariableExprs<'v> { scope: Scope<'v> }
impl<'v> Visitor<'v> for ExtractVariableExprs<'v> {
    type Value = Scope<'v>;
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

struct AssignHandler<'c, 'v, 'r> {
    hierarchy: &'c mut ScopeHierarchy<'v, 'r>
}

impl<'c, 'v, 'r> Visitor<'v> for AssignHandler<'c, 'v, 'r> {
    type Value = ValueType<'v>;
    type Error = EvaluationError<'v>;

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

    fn visit_variable_reference(mut self, _expr: &'v Expr, value: &'v StringExpr) -> Result<Self::Value, Self::Error> {
        match self.hierarchy.get_resolved_variable(&value.value) {
            Some(v) => Ok(v),
            None => {
                let (scopde_idx, lexpr) =
                    self.hierarchy.get_variable_expression(&value.value)?;
                let resolved = lexpr.value.accept(AssignHandler{hierarchy: self.hierarchy})?;
                self.hierarchy.add_variable_resolution(scopde_idx, &value.value, resolved.clone());
                Ok(resolved)
            }
        }
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
    hierarchy: &'c mut ScopeHierarchy<'v, 'r>,
    stack: &'c mut Vec<ValueType<'v>>
}

impl<'c, 'v, 'r> Visitor<'v> for QueryHandler<'c, 'v, 'r> {
    type Value = bool;
    type Error = EvaluationError<'v>;

    fn visit_select(mut self, _expr: &'v Expr, value: &'v QueryExpr) -> Result<Self::Value, Self::Error> {
        for each in &value.parts {
            if !each.accept(QueryHandler{hierarchy: self.hierarchy, stack: self.stack})? {
                return Ok(false)
            }
        }
        Ok(true)
    }

    fn visit_string(mut self, expr: &'v Expr, value: &'v StringExpr) -> Result<Self::Value, Self::Error> {
        if self.stack.is_empty() {
            'exit: loop {
                for each in self.hierarchy.get_data_roots() {
                    if value.value == "*" {
                        if let Value::List(v, _) = &each.root {
                            for each in v {
                                self.stack.push(ValueType::DataValue(each));
                            }
                            break 'exit;
                        } else if let Value::Map(map, _) = &each.root {
                            for each_value in map.values() {
                                self.stack.push(ValueType::DataValue(each_value));
                            }
                            break 'exit;
                        }
                    } else {
                        if let Value::Map(map, _) = &each.root {
                            if let Some(value) = map.get(&value.value) {
                                self.stack.push(ValueType::DataValue(value));
                                break 'exit;
                            }
                        }
                    }
                }
                return Err(EvaluationError::ComputationError(
                    format!("Could not find any datafile that satisfies the query {:?}. Data file {:?}",
                            expr,
                    self.hierarchy.roots)
                ))
            }
        }
        else  {
            if value.value == "this" {
                return Ok(true)
            }
            let mut current: Vec<ValueType<'_>> = self.stack.drain(..).collect();
            let index = value.value.parse::<i32>().map_or(None, |i| Some(i));
            if let Some(i) = index {
                while let Some(top) = current.pop() {
                    match top {
                        ValueType::DataValue(Value::List(list, _)) => {
                            let i = (if i < 0 { list.len() as i32 + i } else { i }) as usize;
                            if let Some(v) = list.get(i) {
                                self.stack.push(ValueType::DataValue(v));
                            }
                            else {
                                self.hierarchy.reporter.report_missing_value(
                                    top,
                                    "",
                                    expr
                                )?;
                            }
                            continue
                        },
                        _ =>  {
                            self.hierarchy.reporter.report_mismatch_value_traversal(
                                top,
                                "",
                                expr
                            )?;
                        }
                    }
                }
            }
            else {
                while let Some(top) = current.pop() {
                    match top {
                        ValueType::DataValue(Value::Map(map, _)) => {
                            match value.value.as_str() {
                                "*" => {
                                    if map.is_empty() {
                                        self.hierarchy.reporter.report_missing_value(
                                            top,
                                            "",
                                            expr
                                        )?;
                                        continue;
                                    }
                                    for each_value in map.values() {
                                        self.stack.push(ValueType::DataValue(each_value));
                                    }
                                },

                                rest => {
                                    match map.get(rest) {
                                        Some(v) => {
                                            self.stack.push(ValueType::DataValue(v));
                                        },
                                        None => {
                                            self.hierarchy.reporter.report_missing_value(
                                                top,
                                                "",
                                                expr
                                            )?;
                                        }
                                    }
                                }
                            }
                        },
                        ValueType::DataValue(Value::List(list, _)) => {
                            match value.value.as_str() {
                                "*" => {
                                    if list.is_empty() {
                                        self.hierarchy.reporter.report_missing_value(
                                            top,
                                            "",
                                            expr
                                        )?;
                                        continue;
                                    }
                                    for each_value in list {
                                        self.stack.push(ValueType::DataValue(each_value));
                                    }
                                },
                                _ => {
                                    self.hierarchy.reporter.report_mismatch_value_traversal(
                                        top,
                                        "",
                                        expr
                                    )?;
                                }
                            }
                        }
                        _ => {
                            self.hierarchy.reporter.report_mismatch_value_traversal(
                                top,
                                "",
                                expr
                            )?;
                        }
                    }
                }
            }
        }
        Ok(!self.stack.is_empty())
    }


    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod query_tests;

#[cfg(test)]
mod tests_common;