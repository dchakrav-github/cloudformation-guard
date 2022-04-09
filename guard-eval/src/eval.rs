use crate::{EvaluationError, Value, EvalReporter, Status, DataFiles, DataFile, ValueType, Comparison, BinaryComparison};

use guard_lang::{
      Expr
	, FileExpr
	, Visitor
	, StringExpr
	, RegexExpr
	, CharExpr
	, BoolExpr
	, IntExpr
	, FloatExpr
	, RangeIntExpr
	, RangeFloatExpr
	, BinaryExpr
	, BinaryOperator
	, ArrayExpr
	, MapExpr
	, RuleExpr
	, RuleClauseExpr
	, LetExpr
	, WhenExpr
	, QueryExpr
	, UnaryExpr
	, Location
	, BlockExpr
	, BlockClauseExpr
	, UnaryOperator};

use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;
use std::io::Error;
use inflector::cases::*;
use lazy_static::lazy_static;

pub fn evaluate<'e, 's>(rule_file: &'s Expr,
                    data: &'s Value,
                    reporter: &'e mut dyn EvalReporter<'s>) -> Result<Status, EvaluationError<'s>>
{
    let mut hierarchy = RootScopeHierarchy {
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
    struct RootContext<'context, 'value> {
        scope: &'context mut Scope<'value>,
        hierarchy: &'context mut dyn ScopeHierarchy<'value>
    }
    impl<'context, 'value> Visitor<'value> for RootContext<'context, 'value> {
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

#[derive(Debug)]
struct RootScopeHierarchy<'value, 'reporter> {
    roots: &'value Value,
    scopes: Vec<Scope<'value>>,
    completed: Vec<Scope<'value>>,
    reporter: &'reporter mut dyn EvalReporter<'value>
}

trait ScopeHierarchy<'v> : EvalReporter<'v> {
    fn get_resolved_variable(&mut self, name: &str) -> Option<Vec<ValueType<'v>>> {
        match self.get_parent() {
            Some(parent) => parent.get_resolved_variable(name),
            None => None
        }
    }

    fn get_root(&mut self) -> &'v Value;

    fn get_variable_expression(&mut self, name: &str) -> Result<(usize, &'v LetExpr), EvaluationError<'v>> {
        match self.get_parent() {
            Some(parent) => parent.get_variable_expression(name),
            None => Err(EvaluationError::ComputationError(
                format!("Variable name {} could nt be found", name)
            ))
        }
    }

    fn add_variable_resolution(&mut self, scope_idx: usize, name: &'v str, value: Vec<ValueType<'v>>) -> Result<(), EvaluationError<'v>>{
        match self.get_parent() {
            Some(parent) => parent.add_variable_resolution(scope_idx, name, value),
            None => Err(EvaluationError::ComputationError(
                format!("Could not add variable resolution to scope {} could nt be found", name)
            ))
        }
    }

    fn resolve_rule<'s>(&mut self, name: &str) -> Result<Status, EvaluationError<'s>> {
        match self.get_parent() {
            Some(parent) => parent.resolve_rule(name),
            None => Err(EvaluationError::ComputationError(
                format!("Rule was not resolved {}", name)
            ))
        }
    }

    fn add_scope(&mut self, scope: Scope<'v>) -> Result<(), EvaluationError<'v>> {
        match self.get_parent() {
            Some(parent) => parent.add_scope(scope),
            None => Err(EvaluationError::ComputationError(
                format!("Can not add scope")
            ))
        }
    }

    fn drop_scope(&mut self) -> Result<(), EvaluationError<'v>>{
        match self.get_parent() {
            Some(parent) => parent.drop_scope(),
            None => Err(EvaluationError::ComputationError(
                format!("Cannot drop scope")
            ))
        }
    }

    fn get_parent(&mut self) -> Option<&mut dyn ScopeHierarchy<'v>>;
}

impl<'v, 'r> ScopeHierarchy<'v> for RootScopeHierarchy<'v, 'r> {
    fn get_resolved_variable(&mut self, name: &str) -> Option<Vec<ValueType<'v>>> {
        for each_scope in &self.scopes {
            match each_scope.variables.get(name) {
                Some(v) => return Some(v.clone()),
                None => continue
            }
        }
        None
    }

    fn get_root(&mut self) -> &'v Value {
        self.roots
    }

    fn get_variable_expression(&mut self, name: &str) -> Result<(usize, &'v LetExpr), EvaluationError<'v>> {
        for (idx, each_scope) in self.scopes.iter().enumerate() {
            match each_scope.variable_definitions.get(name) {
                Some(v) => return Ok((idx, *v)),
                None => continue
            }
        }
        return Err(EvaluationError::ComputationError(
            format!("Variable {} could not resolved in any scope", name)))
    }

    fn add_variable_resolution(&mut self, scope_idx: usize, name: &'v str, value: Vec<ValueType<'v>>) -> Result<(), EvaluationError<'v>> {
        self.scopes.get_mut(scope_idx).unwrap().variables.insert(name, value);
        Ok(())
    }


    fn resolve_rule<'s>(&mut self, name: &str) -> Result<Status, EvaluationError<'s>> {
        todo!()
    }

    fn add_scope(&mut self, scope: Scope<'v>) -> Result<(), EvaluationError<'v>> {
        self.scopes.insert(0, scope);
        Ok(())
    }

    fn drop_scope(&mut self) -> Result<(), EvaluationError<'v>> {
        if !self.scopes.is_empty() {
            self.completed.insert(0, self.scopes.remove(0));
        }
        Ok(())
    }

    fn get_parent(&mut self) -> Option<&mut dyn ScopeHierarchy<'v>> {
        None
    }
}

impl<'v, 'r> EvalReporter<'v> for RootScopeHierarchy<'v, 'r> {
    fn report_missing_value(&mut self, until: ValueType<'v>, data_file_name: &'v str, expr: &'v Expr) -> Result<(), Error> {
        self.reporter.report_missing_value(until.clone(), data_file_name, expr)
    }

    fn report_mismatch_value_traversal(&mut self, until: ValueType<'v>, data_file_name: &'v str, expr: &'v Expr) -> Result<(), Error> {
        self.reporter.report_mismatch_value_traversal(until, data_file_name, expr)
    }

    fn report_evaluation(&mut self, status: Status, comparison: Comparison<'v>, data_file_name: &'v str, expr: &'v Expr) -> Result<(), Error> {
        self.reporter.report_evaluation(status, comparison, data_file_name, expr)
    }
}


#[derive(Debug)]
struct Scope<'value> {
    variable_definitions: HashMap<&'value str, &'value LetExpr>,
    variables: HashMap<&'value str, Vec<ValueType<'value>>>,
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

struct AssignHandler<'c, 'v> {
    hierarchy: &'c mut dyn ScopeHierarchy<'v>
}

enum SingleOrQuery<'v> {
    Single(ValueType<'v>),
    Query(Vec<ValueType<'v>>)
}

impl<'c, 'v> Visitor<'v> for AssignHandler<'c, 'v> {
    type Value = SingleOrQuery<'v>;
    type Error = EvaluationError<'v>;

    fn visit_select(self, expr: &'v Expr, _value: &'v QueryExpr) -> Result<Self::Value, Self::Error> {
        let mut stack = Vec::new();
        stack = expr.accept(QueryHandler{hierarchy: self.hierarchy, stack} )?;
        Ok(SingleOrQuery::Query(stack))
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
        unimplemented!()
//        let mut converted = Vec::with_capacity(value.elements.len());
//        for each in &value.elements {
//            match each.accept(
//                AssignHandler{hierarchy: self.hierarchy})? {
//                SingleOrQuery::Single(v) => match v {
//                    ValueType::LiteralValue(e) => converted.push(Value::try_from(e)?),
//                    _ => unreachable!()
//                },
//                SingleOrQuery::Query(resolved) => {
//                    let mut value = Vec::with_capacity(resolved.len());
//                    for each in resolved {
//                        match each {
//                            ValueType::LiteralValue(e) => value.push(Value::try_from(e)?),
//                            ValueType::ComputedValue(v) => value.push(v),
//                            _ => unreachable!()
//                        }
//                    }
//                    converted.push(Value::List(value, Location::new(0, 0)));
//                }
//            }
//        }
//        Ok(SingleOrQuery::Single(ValueType::ComputedValue(
//            Value::List(converted, expr.get_location().clone())
//        )))
    }

    fn visit_map(self, expr: &'v Expr, value: &'v MapExpr) -> Result<Self::Value, Self::Error> {
        if expr.accept(CheckValueLiteral{})? {
            return literal(expr)
        }
        unimplemented!()
//        let mut converted = indexmap::IndexMap::new();
//        for (key, value) in &value.entries {
//            let value =  match value.accept(
//                AssignHandler{hierarchy: self.hierarchy})? {
//                SingleOrQuery::Single(v) => match v {
//                    ValueType::LiteralValue(e) => Value::try_from(e)?,
//                    ValueType::ComputedValue(v) => v,
//                    _ => unreachable!()
//                },
//                SingleOrQuery::Query(resolved) => {
//                    let mut value = Vec::with_capacity(resolved.len());
//                    for each in resolved {
//                        match each {
//                            ValueType::LiteralValue(e) => value.push(Value::try_from(e)?),
//                            ValueType::ComputedValue(v) => value.push(v),
//                            _ => unreachable!()
//                        }
//                    }
//                    Value::List(value, Location::new(0, 0))
//                }
//            };
//            converted.insert(key.clone(), value);
//        }
//        Ok(SingleOrQuery::Single(ValueType::ComputedValue(
//            Value::Map(converted, expr.get_location().clone())
//        )))
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

fn literal(expr: &Expr) -> Result<SingleOrQuery, EvaluationError> {
    Ok(SingleOrQuery::Single(ValueType::LiteralValue(expr)))
}

struct FindFromDataFiles<'c, 'v> {
    hierarchy: &'c mut dyn ScopeHierarchy<'v>,
}

impl<'c, 'v> Visitor<'v> for FindFromDataFiles<'c, 'v> {
    type Value = Vec<ValueType<'v>>;
    type Error = EvaluationError<'v>;

    fn visit_string(self, expr: &'v Expr, value: &'v StringExpr) -> Result<Self::Value, Self::Error> {
        Ok('exit: loop {
            let mut stack = Vec::new();
            let each = self.hierarchy.get_root();
            if value.value == "*" {
                if let Value::List(v, _) = each {
                    for each in v {
                        stack.push(ValueType::DataValue(each));
                    }
                    break 'exit stack;
                } else if let Value::Map(map, _) = each {
                    for each_value in map.values() {
                        stack.push(ValueType::DataValue(each_value));
                    }
                    break 'exit stack;
                }
            } else {
                if let Value::Map(map, _) = each {
                    if let Some(value) = map.get(&value.value) {
                        stack.push(ValueType::DataValue(value));
                        break 'exit stack;
                    }
                }
            }
            return Err(EvaluationError::ComputationError(
                format!("Could not find any datafile that satisfies the query {:?}. Data file {:?}",
                        expr,
                        self.hierarchy.get_root())
            ))
        })
    }

    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        Err(EvaluationError::UnexpectedExpr(
            format!("When attempting extract assignment statements got unexpected Expr"),
            expr
        ))
    }
}

struct FromIndexLookup<'c, 'v> {
    hierarchy: &'c mut dyn ScopeHierarchy<'v>,
    stack: Vec<ValueType<'v>>,
}

impl<'c, 'v> Visitor<'v> for FromIndexLookup<'c, 'v> {
    type Value = Vec<ValueType<'v>>;
    type Error = EvaluationError<'v>;

    fn visit_string(mut self, expr: &'v Expr, value: &'v StringExpr) -> Result<Self::Value, Self::Error> {
        match value.value.parse::<i32>() {
            Ok(i) => {
                let mut current: Vec<ValueType<'_>> = self.stack.drain(..).collect();
                while let Some(top) = current.pop() {
                    match top {
                        ValueType::DataValue(Value::List(ref list, _)) => {
                            let i = (if i < 0 { list.len() as i32 + i } else { i }) as usize;
                            if let Some(v) = list.get(i) {
                                self.stack.push(ValueType::DataValue(v));
                                continue
                            }
                            self.hierarchy.report_mismatch_value_traversal(
                                top.clone(),
                                "",
                                expr
                            )?;
                        },

                        rest => {
                            self.hierarchy.report_mismatch_value_traversal(
                                rest,
                                "",
                                expr
                            )?;
                        }
                    };
                }
                Ok(self.stack)
            },
            Err(e) => Err(EvaluationError::QueryEvaluationError(
                format!("Not an index to lookup {}", e), self.stack
            ))
        }
    }

    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        Err(EvaluationError::UnexpectedExpr(
            format!("When attempting extract assignment statements got unexpected Expr"),
            expr
        ))
    }
}

struct FromQueryIndexLookup<'c, 'v> {
    hierarchy: &'c mut dyn ScopeHierarchy<'v>,
    stack: Vec<ValueType<'v>>,
}

lazy_static! {
    static ref CONVERTERS: &'static [(fn(&str) -> bool, fn(&str) -> String)] =
        &[
            (camelcase::is_camel_case, camelcase::to_camel_case),
            (classcase::is_class_case, classcase::to_class_case),
            (kebabcase::is_kebab_case, kebabcase::to_kebab_case),
            (pascalcase::is_pascal_case, pascalcase::to_pascal_case),
            (snakecase::is_snake_case, snakecase::to_snake_case),
            (titlecase::is_title_case, titlecase::to_title_case),
            (traincase::is_train_case, traincase::to_train_case),
        ];
}

impl<'c, 'v> Visitor<'v> for FromQueryIndexLookup<'c, 'v> {
    type Value = Vec<ValueType<'v>>;
    type Error = EvaluationError<'v>;

    fn visit_string(mut self, expr: &'v Expr, value: &'v StringExpr) -> Result<Self::Value, Self::Error> {
        let mut current: Vec<ValueType<'_>> = self.stack.drain(..).collect();
        'top: while let Some(top) = current.pop() {
            match top {
                ValueType::DataValue(Value::Map(map, _)) => {
                    match value.value.as_str() {
                        "*" => {
                            if map.is_empty() {
                                self.hierarchy.report_missing_value(
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
                                    for (_is_case_type, convert_case_type) in CONVERTERS.iter() {
                                        if let Some(value) = map.get(&convert_case_type(rest)) {
                                            self.stack.push(ValueType::DataValue(value));
                                            continue 'top;
                                        }
                                    }
                                    self.hierarchy.report_missing_value(
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
                                self.hierarchy.report_missing_value(
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
                            self.hierarchy.report_mismatch_value_traversal(
                                top,
                                "",
                                expr
                            )?;
                        }
                    }
                },

                rest => {
                    match value.value.as_str() {
                        "*" => self.stack.push(rest),
                        _   => self.hierarchy.report_mismatch_value_traversal(
                            rest,
                            "",
                            expr
                        )?
                    }
                }
            }
        }
        Ok(self.stack)
    }

    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        Err(EvaluationError::UnexpectedExpr(
            format!("When attempting extract assignment statements got unexpected Expr"),
            expr
        ))
    }
}

struct QueryHandler<'c, 'v> {
    hierarchy: &'c mut dyn ScopeHierarchy<'v>,
    stack: Vec<ValueType<'v>>,
}

struct AddRemoveScope<'c, 'v> {
    hierarchy: &'c mut dyn ScopeHierarchy<'v>,
}

impl<'c, 'v > AddRemoveScope<'c, 'v> {
    fn add_scope(&mut self, expr: &'v Expr) -> Result<(), EvaluationError<'v>> {
        let scope = expr.accept(ExtractVariableExprs {
            scope: Scope {
                variables: HashMap::new(),
                variable_definitions: HashMap::new()
            }
        })?;
        self.hierarchy.add_scope(scope);
        Ok(())
    }
}

impl<'c, 'v > Drop for AddRemoveScope<'c, 'v> {
    fn drop(&mut self) {
        self.hierarchy.drop_scope();
    }
}

impl<'c, 'v> Visitor<'v> for QueryHandler<'c, 'v> {
    type Value = Vec<ValueType<'v>>;
    type Error = EvaluationError<'v>;

    fn visit_select(mut self, _expr: &'v Expr, value: &'v QueryExpr) -> Result<Self::Value, Self::Error> {
        for each in &value.parts {
            self.stack = each.accept(QueryHandler{hierarchy: self.hierarchy, stack: self.stack})?;
            if self.stack.is_empty() {
                break;
            }
        }
        Ok(self.stack)
    }

    fn visit_string(self, expr: &'v Expr, value: &'v StringExpr) -> Result<Self::Value, Self::Error> {
        if self.stack.is_empty() {
            expr.accept(FindFromDataFiles{hierarchy: self.hierarchy})
        }
        else  {
            if value.value == "this" {
                return Ok(self.stack)
            }

            match expr.accept(FromIndexLookup{hierarchy: self.hierarchy, stack: self.stack}) {
                Ok(stack) => return Ok(stack),
                Err(EvaluationError::QueryEvaluationError(_, stack)) => expr.accept(FromQueryIndexLookup{hierarchy: self.hierarchy, stack}),
                Err(e) => Err(e)
            }

        }
    }

    fn visit_filter(mut self, expr: &'v Expr, _value: &'v BlockExpr) -> Result<Self::Value, Self::Error> {
        if self.stack.is_empty() {
            return Ok(self.stack)
        }

        let mut current: Vec<ValueType<'_>> = self.stack.drain(..).collect();
        let mut add_scope = AddRemoveScope{ hierarchy: self.hierarchy };
        add_scope.add_scope(expr);
        while let Some(top) = current.pop() {

        }

        todo!()
    }

    fn visit_variable_reference(mut self, expr: &'v Expr, value: &'v StringExpr) -> Result<Self::Value, Self::Error> {
        let values = match self.hierarchy.get_resolved_variable(&value.value) {
            Some(v) => v,
            None => {
                let (scopde_idx, lexpr) =
                    self.hierarchy.get_variable_expression(&value.value)?;
                let resolved = lexpr.value.accept(AssignHandler { hierarchy: self.hierarchy })?;
                let resolved = match resolved {
                    SingleOrQuery::Single(v) => vec![v],
                    SingleOrQuery::Query(q) => q,
                };
                self.hierarchy.add_variable_resolution(scopde_idx, &value.value, resolved.clone());
                resolved
            }
        };
        if self.stack.is_empty() {
            self.stack.extend(values);
            Ok(self.stack)
        } else {
            let mut current: Vec<ValueType<'_>> = self.stack.drain(..).collect();
            while let Some(top) = current.pop() {
                match top {
                    ValueType::DataValue(Value::Map(map, _)) => {
                        for each in &values {
                            match each {
                                ValueType::DataValue(Value::String(v, _)) =>
                                    self.map_key_value(top.clone(), expr, v, map)?,

                                ValueType::LiteralValue(v) =>
                                    self.map_literal_expr(top.clone(), expr, v, map)?,

                                _ => {
                                    self.hierarchy.report_mismatch_value_traversal(
                                        each.clone(),
                                        "",
                                        expr
                                    )?;
                                }
                            }
                        }
                    },
                    rest => {
                        self.hierarchy.report_missing_value(
                            rest,
                            "",
                            expr
                        )?;
                    }
                }
            }
            Ok(self.stack)
        }
    }

    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        todo!()
    }
}

impl<'c, 'v> QueryHandler<'c, 'v> {
    fn map_key_value(&mut self,
                     top: ValueType<'v>,
                     expr: &'v Expr,
                     key: &'v str,
                     map: &'v indexmap::IndexMap<String, Value>) -> Result<(), EvaluationError<'v>> {
        match map.get(key) {
            Some(value) => {
                self.stack.push(ValueType::DataValue(value));
            }
            None => {
                self.hierarchy.report_missing_value(
                    top,
                    "",
                    expr
                )?;
            }
        }
        Ok(())
    }

    fn map_literal_expr(&mut self,
                     top: ValueType<'v>,
                     expr: &'v Expr,
                     key: &'v Expr,
                     map: &'v indexmap::IndexMap<String, Value>) -> Result<(), EvaluationError<'v>> {
        match key {
            Expr::String(value) =>
                self.map_key_value(top.clone(), expr, &value.value, map)?,

            Expr::Regex(value) => {
                let regex = regex::Regex::new(&value.value).unwrap();
                for key in map.keys().filter(|k| regex.is_match(k)) {
                    self.stack.push(
                        ValueType::DataValue(
                            map.get(key).unwrap()
                        )
                    );
                }
            },

            Expr::Array(value) => {
                for each in &value.elements {
                    self.map_literal_expr(top.clone(), expr, each, map)?;
                }
            },

            _ => {
                self.hierarchy.report_mismatch_value_traversal(
                    ValueType::LiteralValue(key),
                    "",
                    expr
                )?;
            }


        }
        Ok(())
    }

}

#[cfg(test)]
mod query_tests;

#[cfg(test)]
mod tests_common;


struct AndBinaryOperationsHandler<'c, 'v> {
    hierarchy: &'c mut dyn ScopeHierarchy<'v>,
    stack: Vec<ValueType<'v>>,
}

impl<'c, 'v> Visitor<'v> for AndBinaryOperationsHandler<'c, 'v> {
    type Value = bool;
    type Error = EvaluationError<'v>;

    fn visit_binary_operation(self, expr: &'v Expr, value: &'v BinaryExpr) -> Result<Self::Value, Self::Error> {
        match value.operator {
            BinaryOperator::And => {

            },

            BinaryOperator::Or => {

            },

            _ => todo!()
        }
        todo!()
    }

    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        todo!()
    }
}

struct BinaryOperationsHandler<'c, 'v> {
    hierarchy: &'c mut dyn ScopeHierarchy<'v>,
    stack: Vec<ValueType<'v>>,
}

impl<'c, 'v> Visitor<'v> for BinaryOperationsHandler<'c, 'v> {
    type Value = bool;
    type Error = EvaluationError<'v>;

    fn visit_binary_operation(mut self,
                              expr: &'v Expr,
                              value: &'v BinaryExpr) -> Result<Self::Value, Self::Error> {
        match value.operator {
            BinaryOperator::Equals             |
            BinaryOperator::NotEquals          |
            BinaryOperator::LesserThanEquals   |
            BinaryOperator::Lesser             |
            BinaryOperator::GreaterThanEquals  |
            BinaryOperator::Greater            |
            BinaryOperator::NotIn              |
            BinaryOperator::In  => {},
            _ => return self.visit_any(expr)
        }

        let lhs_query = match &value.lhs {
            Expr::Select(_) => SingleOrQuery::Query(value.lhs.accept(QueryHandler{hierarchy: self.hierarchy, stack: self.stack.clone()})?),
            _ => SingleOrQuery::Single(ValueType::LiteralValue(&value.lhs))
        };
        let rhs_query = match &value.rhs {
            Expr::Select(_) => SingleOrQuery::Query(value.rhs.accept(QueryHandler{hierarchy: self.hierarchy, stack: self.stack.clone()})?),
            _ => SingleOrQuery::Single(ValueType::LiteralValue(&value.rhs))
        };

        Ok(check_operator(value.operator, expr, lhs_query, rhs_query, self.hierarchy)?)
    }


    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        todo!()
    }
}

fn check_operator<'v>(
    op: BinaryOperator
    , binop: &'v Expr
    , lhs: SingleOrQuery<'v>
    , rhs: SingleOrQuery<'v>
    , reporter: &mut dyn ScopeHierarchy<'v>) -> Result<bool, EvaluationError<'v>>
{
    let mut result = true;
    match lhs {
        SingleOrQuery::Single(ValueType::LiteralValue(lhs)) => {
            match rhs {
                SingleOrQuery::Single(ValueType::LiteralValue(rhs)) => {
                    result &= match_operator_literal(op, lhs, rhs);
                    let status = if result { Status::PASS } else { Status::FAIL };
                    reporter.report_evaluation(
                        status,
                        Comparison::Binary(BinaryComparison {
                            operator: op,
                            lhs: ValueType::LiteralValue(lhs),
                            rhs: ValueType::LiteralValue(rhs),
                        }),
                        "",
                        binop
                    )?;
                },
                SingleOrQuery::Query(rhs) => {
                    result &= check_literal_operator(op, binop, lhs, &rhs, reporter)?;
                },
                _ => unreachable!()
            }
        }
        SingleOrQuery::Query(lhs) => {
            match rhs {
                SingleOrQuery::Single(ValueType::LiteralValue(rhs)) => {
                    for each_lhs in lhs {
                        let value = match each_lhs {
                            ValueType::DataValue(ref v) => *v,
                            _ => unreachable!()
                        };
                        result &= match_operator_value(op, value, rhs);
                        let status = if result { Status::PASS } else { Status::FAIL };
                        reporter.report_evaluation(
                            status,
                            Comparison::Binary(BinaryComparison {
                                operator: op,
                                lhs: each_lhs,
                                rhs: ValueType::LiteralValue(rhs),
                            }),
                            "",
                            binop
                        )?;
                    }
                },
                SingleOrQuery::Query(rhs) => {
                    for each_lhs in lhs {
                        let lhs_value = match &each_lhs {
                            ValueType::DataValue(v) => *v,
                            _ => unreachable!()
                        };
                        for each in &rhs {
                            let value = match each {
                                ValueType::DataValue(ref v) => *v,
                                _ => unreachable!()
                            };
                            result &= match_in_both_values(lhs_value, value);
                            let status = if result { Status::PASS } else { Status::FAIL };
                            reporter.report_evaluation(
                                status,
                                Comparison::Binary(BinaryComparison {
                                    operator: op,
                                    lhs: each_lhs.clone(),
                                    rhs: each.clone()
                                }),
                                "",
                                binop
                            )?;
                        }
                    }
                },
                _ => unreachable!()
            }
        },
        _ => unreachable!()
    }
    Ok(result)
}

fn check_literal_operator<'v>(
    op: BinaryOperator
    , binop: &'v Expr
    , literal: &'v Expr
    , rhs: &Vec<ValueType<'v>>
    , reporter: &mut dyn ScopeHierarchy<'v>) -> Result<bool, EvaluationError<'v>>
{
    let mut result = true;
    for each in rhs {
        let value = match each {
            ValueType::DataValue(v) => *v,
            _ => unreachable!()
        };
        result &= match_operator_literal_with_value(op, literal, value);
        let status = if result { Status::PASS } else { Status::FAIL };
        reporter.report_evaluation(
            status,
            Comparison::Binary(BinaryComparison {
                operator: op,
                lhs: ValueType::LiteralValue(literal),
                rhs: each.clone()
            }),
            "",
            binop
        )?;
    }
    Ok(result)
}

fn match_operator_literal_with_value<'v>(
    op: BinaryOperator
    , lhs_literal: &'v Expr
    , rhs_literal: &'v Value) -> bool
{
    match op {
        BinaryOperator::Lesser => rhs_literal >= lhs_literal,
        BinaryOperator::LesserThanEquals => rhs_literal > lhs_literal,
        BinaryOperator::Greater => rhs_literal <= lhs_literal,
        BinaryOperator::GreaterThanEquals => rhs_literal < lhs_literal,
        BinaryOperator::Equals => rhs_literal == lhs_literal,
        BinaryOperator::NotEquals => rhs_literal != lhs_literal,
        BinaryOperator::In => match_in_literal_with_value(lhs_literal, rhs_literal),
        BinaryOperator::NotIn => !match_in_literal_with_value(lhs_literal, rhs_literal),
        _ => unreachable!()
    }
}

fn match_in_literal_with_value<'v>(
    lhs_literal: &'v Expr
    , rhs_literal: &'v Value) -> bool
{
    match (lhs_literal, rhs_literal) {
        (Expr::String(lhs), Value::String(rhs, _)) => {
            rhs.contains(&lhs.value)
        },
        (Expr::Array(lhs), Value::List(rhs, _)) => {
            lhs.elements.iter().all(|expr| rhs.iter().any(|elem| elem == expr))
        }
        _ => false
    }
}


fn match_operator_value<'v>(
    op: BinaryOperator
    , lhs_literal: &'v Value
    , rhs_literal: &'v Expr) -> bool
{
    match op {
        BinaryOperator::Lesser => lhs_literal < rhs_literal,
        BinaryOperator::LesserThanEquals => lhs_literal <= rhs_literal,
        BinaryOperator::Greater => lhs_literal > rhs_literal,
        BinaryOperator::GreaterThanEquals => lhs_literal >= rhs_literal,
        BinaryOperator::Equals => lhs_literal == rhs_literal,
        BinaryOperator::NotEquals => lhs_literal != rhs_literal,
        BinaryOperator::In => match_in_value(lhs_literal, rhs_literal),
        BinaryOperator::NotIn => !match_in_value(lhs_literal, rhs_literal),
        _ => unreachable!()
    }
}

fn match_operator_literal<'v>(
    op: BinaryOperator
    , lhs_literal: &'v Expr
    , rhs_literal: &'v Expr) -> bool
{
    match op {
        BinaryOperator::Lesser => lhs_literal < rhs_literal,
        BinaryOperator::LesserThanEquals => lhs_literal <= rhs_literal,
        BinaryOperator::Greater => lhs_literal > rhs_literal,
        BinaryOperator::GreaterThanEquals => lhs_literal >= rhs_literal,
        BinaryOperator::Equals => lhs_literal == rhs_literal,
        BinaryOperator::NotEquals => lhs_literal != rhs_literal,
        BinaryOperator::In => match_in_expr(lhs_literal, rhs_literal),
        BinaryOperator::NotIn => !match_in_expr(lhs_literal, rhs_literal),
        _ => unreachable!()
    }
}

fn match_operator_both_value<'v>(
    op: BinaryOperator
    , lhs_literal: &'v Value
    , rhs_literal: &'v Value) -> bool
{
    match op {
        BinaryOperator::Lesser => lhs_literal < rhs_literal,
        BinaryOperator::LesserThanEquals => lhs_literal <= rhs_literal,
        BinaryOperator::Greater => lhs_literal > rhs_literal,
        BinaryOperator::GreaterThanEquals => lhs_literal >= rhs_literal,
        BinaryOperator::Equals => lhs_literal == rhs_literal,
        BinaryOperator::NotEquals => lhs_literal != rhs_literal,
        BinaryOperator::In => match_in_both_values(lhs_literal, rhs_literal),
        BinaryOperator::NotIn => !match_in_both_values(lhs_literal, rhs_literal),
        _ => unreachable!()
    }
}

fn match_in_expr<'v>(
    lhs_literal: &'v Expr
    , rhs_literal: &'v Expr) -> bool
{
    match (lhs_literal, rhs_literal) {
        (Expr::String(lhs),
            Expr::String(rhs)) => rhs.value.contains(&lhs.value),
        (Expr::Array(lhs),
            Expr::Array(rhs)) => lhs.elements.len() == rhs.elements.len() &&
                lhs.elements.iter().all(|e| rhs.elements.contains(e)),
        (_, _) => false,
    }
}

fn match_in_value<'v>(
    lhs_literal: &'v Value
    , rhs_literal: &'v Expr) -> bool
{
    match (lhs_literal, rhs_literal) {
        (Value::String(lhs, _),
            Expr::String(rhs)) => rhs.value.contains(lhs),
        (Value::List(lhs, _),
            Expr::Array(rhs)) => lhs.len() == rhs.elements.len() &&
                lhs.iter().all(|elem| rhs.elements.iter().any(|expr| elem == expr)),
        (_, _) => false,
    }
}

fn match_in_both_values<'v>(
    lhs_literal: &'v Value
    , rhs_literal: &'v Value) -> bool
{
    match (lhs_literal, rhs_literal) {
        (Value::String(lhs, _),
            Value::String(rhs, _)) =>  rhs.contains(lhs),
        (Value::List(lhs, _),
            Value::List(rhs, _)) => lhs.iter().all(|e| rhs.contains(e)),
        (_, _) => false,
    }
}

#[derive(Debug)]
struct TrackReportMissing<'p, 'v> {
    missing: bool,
    parent: &'p mut dyn ScopeHierarchy<'v>
}

impl<'p, 'v> ScopeHierarchy<'v> for TrackReportMissing<'p, 'v> {
    fn get_root(&mut self) -> &'v Value {
        self.parent.get_root()
    }

    fn get_parent(&mut self) -> Option<&mut dyn ScopeHierarchy<'v>> {
        Some(self.parent)
    }
}

impl<'p, 'v> EvalReporter<'v> for TrackReportMissing<'p, 'v> {
    fn report_missing_value(&mut self, until: ValueType<'v>, data_file_name: &'v str, expr: &'v Expr) -> Result<(), Error> {
        self.missing = true;
        self.parent.report_missing_value(until, data_file_name, expr)
    }

    fn report_mismatch_value_traversal(&mut self, until: ValueType<'v>, data_file_name: &'v str, expr: &'v Expr) -> Result<(), Error> {
        self.missing = true;
        self.parent.report_mismatch_value_traversal(until, data_file_name, expr)
    }

    fn report_evaluation(&mut self, status: Status, comparison: Comparison<'v>, data_file: &'v str, expr: &'v Expr) -> Result<(), Error> {
        self.parent.report_evaluation(status, comparison, data_file, expr)
    }
}

//
// Unary functions
//
struct UnaryOperations<'c, 'v> {
    hierarchy: &'c mut dyn ScopeHierarchy<'v>,
    stack: Vec<ValueType<'v>>,
}

impl<'c, 'v> Visitor<'v> for UnaryOperations<'c, 'v> {
    type Value = bool;
    type Error = EvaluationError<'v>;

    fn visit_unary_operation(self, _expr: &'v Expr, value: &'v UnaryExpr) -> Result<Self::Value, Self::Error> {
        match &value.expr {
            binop @ Expr::BinaryOperation(_) if value.operator == UnaryOperator::Not => {
                return if binop.accept(BinaryOperationsHandler{hierarchy: self.hierarchy, stack: self.stack})? {
                    Ok(false)
                }
                else {
                    Ok(true)
                }
            },

            select @ Expr::Select(_) => {
                let (values, missing) = {
                    let mut tracker = TrackReportMissing { missing: false, parent: self.hierarchy };
                    let values = select.accept(QueryHandler{ hierarchy: &mut tracker, stack: self.stack})?;
                    (values, tracker.missing)
                };
                return Ok(check_unary_operator(values, value.operator, self.hierarchy) && missing)
            }

            _ => todo!()
        }
        todo!()
    }

    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        todo!()
    }
}

macro_rules! is_type_check {
    ($name: ident, $data: pat, $literal: pat) => {
        fn $name<'v>(vec: Vec<ValueType<'v>>, eval: &mut dyn ScopeHierarchy<'v>) -> bool {
            !vec.is_empty() &&
                vec.iter().all(|elem| match elem {
                    ValueType::DataValue(data) => match data {
                        $data => true,
                        _ => false,
                    },
                    ValueType::LiteralValue(literal) => match literal {
                        $literal => true,
                        _ => false
                    }
                })
        }
    };
}

is_type_check!(match_is_string, Value::String(..), Expr::String(_));
is_type_check!(match_is_float, Value::Float(..), Expr::Float(_));
is_type_check!(match_is_bool, Value::Bool(..), Expr::Bool(_));
is_type_check!(match_is_int, Value::Int(..), Expr::Int(_));
is_type_check!(match_is_regex, Value::Regex(..), Expr::Regex(_));
is_type_check!(match_is_null, Value::Null(..), Expr::Null(_));
is_type_check!(match_is_range_float, Value::RangeFloat(..), Expr::RangeFloat(_));
is_type_check!(match_is_range_int, Value::RangeInt(..), Expr::RangeInt(_));

is_type_check!(match_is_list, Value::List(..), Expr::Array(_));
is_type_check!(match_is_map, Value::Map(..), Expr::Map(_));


fn check_unary_operator<'v>(vec: Vec<ValueType<'v>>, operation: UnaryOperator, eval: &mut dyn ScopeHierarchy<'v>) -> bool {
    match operation {
        UnaryOperator::Exists => !vec.is_empty(),
        UnaryOperator::Empty => match_empty(vec),

        UnaryOperator::NotExists => vec.is_empty(),
        UnaryOperator::NotEmpty => !match_empty(vec),

        UnaryOperator::IsString => match_is_string(vec, eval),
        UnaryOperator::IsList => match_is_list(vec, eval),
        UnaryOperator::IsMap => match_is_map(vec, eval),
        UnaryOperator::IsInt => match_is_int(vec, eval),
        UnaryOperator::IsFloat => match_is_float(vec, eval),
        UnaryOperator::IsBool => match_is_bool(vec, eval),
        UnaryOperator::IsRegex => match_is_regex(vec, eval),

        UnaryOperator::IsNotString => !match_is_string(vec, eval),
        UnaryOperator::IsNotList => !match_is_list(vec, eval),
        UnaryOperator::IsNotMap => !match_is_map(vec, eval),
        UnaryOperator::IsNotInt => !match_is_int(vec, eval),
        UnaryOperator::IsNotFloat => !match_is_float(vec, eval),
        UnaryOperator::IsNotBool => !match_is_bool(vec, eval),
        UnaryOperator::IsNotRegex => !match_is_regex(vec, eval),

        _ => false
    }
}

fn match_empty(vec: Vec<ValueType<'_>>) -> bool {
    vec.is_empty() ||
        vec.iter().all(|elem| match elem {
            ValueType::DataValue(Value::List(l, _)) => l.is_empty(),
            ValueType::DataValue(Value::String(s, _)) => s.is_empty(),
            ValueType::DataValue(Value::Map(m, _)) => m.is_empty(),
            ValueType::LiteralValue(Expr::String(s)) => s.value.is_empty(),
            ValueType::LiteralValue(Expr::Array(a)) => a.elements.is_empty(),
            ValueType::LiteralValue(Expr::Map(m)) => m.entries.is_empty(),
            _ => false
        })
}
