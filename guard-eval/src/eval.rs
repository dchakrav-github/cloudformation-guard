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
use std::convert::TryFrom;

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

    fn get_resolved_variable(&self, name: &str) -> Option<Vec<ValueType<'v>>> {
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

    fn add_variable_resolution(&mut self, scope_idx: usize, name: &'v str, value: Vec<ValueType<'v>>) {
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

struct AssignHandler<'c, 'v, 'r> {
    hierarchy: &'c mut ScopeHierarchy<'v, 'r>
}

enum SingleOrQuery<'v> {
    Single(ValueType<'v>),
    Query(Vec<ValueType<'v>>)
}

impl<'c, 'v, 'r> Visitor<'v> for AssignHandler<'c, 'v, 'r> {
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
        let mut converted = Vec::with_capacity(value.elements.len());
        for each in &value.elements {
            match each.accept(
                AssignHandler{hierarchy: self.hierarchy})? {
                SingleOrQuery::Single(v) => match v {
                    ValueType::LiteralValue(e) => converted.push(Value::try_from(e)?),
                    ValueType::ComputedValue(v) => converted.push(v),
                    _ => unreachable!()
                },
                SingleOrQuery::Query(resolved) => {
                    let mut value = Vec::with_capacity(resolved.len());
                    for each in resolved {
                        match each {
                            ValueType::LiteralValue(e) => value.push(Value::try_from(e)?),
                            ValueType::ComputedValue(v) => value.push(v),
                            _ => unreachable!()
                        }
                    }
                    converted.push(Value::List(value, Location::new(0, 0)));
                }
            }
        }
        Ok(SingleOrQuery::Single(ValueType::ComputedValue(
            Value::List(converted, expr.get_location().clone())
        )))
    }

    fn visit_map(self, expr: &'v Expr, value: &'v MapExpr) -> Result<Self::Value, Self::Error> {
        if expr.accept(CheckValueLiteral{})? {
            return literal(expr)
        }
        let mut converted = indexmap::IndexMap::new();
        for (key, value) in &value.entries {
            let value =  match value.accept(
                AssignHandler{hierarchy: self.hierarchy})? {
                SingleOrQuery::Single(v) => match v {
                    ValueType::LiteralValue(e) => Value::try_from(e)?,
                    ValueType::ComputedValue(v) => v,
                    _ => unreachable!()
                },
                SingleOrQuery::Query(resolved) => {
                    let mut value = Vec::with_capacity(resolved.len());
                    for each in resolved {
                        match each {
                            ValueType::LiteralValue(e) => value.push(Value::try_from(e)?),
                            ValueType::ComputedValue(v) => value.push(v),
                            _ => unreachable!()
                        }
                    }
                    Value::List(value, Location::new(0, 0))
                }
            };
            converted.insert(key.clone(), value);
        }
        Ok(SingleOrQuery::Single(ValueType::ComputedValue(
            Value::Map(converted, expr.get_location().clone())
        )))
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

struct FindFromDataFiles<'c, 'v, 'r> {
    hierarchy: &'c mut ScopeHierarchy<'v, 'r>,
    stack: Vec<ValueType<'v>>,
}

impl<'c, 'v, 'r> Visitor<'v> for FindFromDataFiles<'c, 'v, 'r> {
    type Value = Vec<ValueType<'v>>;
    type Error = EvaluationError<'v>;

    fn visit_string(mut self, expr: &'v Expr, value: &'v StringExpr) -> Result<Self::Value, Self::Error> {
        Ok('exit: loop {
            for each in self.hierarchy.get_data_roots() {
                if value.value == "*" {
                    if let Value::List(v, _) = &each.root {
                        for each in v {
                            self.stack.push(ValueType::DataValue(each));
                        }
                        break 'exit self.stack;
                    } else if let Value::Map(map, _) = &each.root {
                        for each_value in map.values() {
                            self.stack.push(ValueType::DataValue(each_value));
                        }
                        break 'exit self.stack;
                    }
                } else {
                    if let Value::Map(map, _) = &each.root {
                        if let Some(value) = map.get(&value.value) {
                            self.stack.push(ValueType::DataValue(value));
                            break 'exit self.stack;
                        }
                    }
                }
            }
            return Err(EvaluationError::ComputationError(
                format!("Could not find any datafile that satisfies the query {:?}. Data file {:?}",
                        expr,
                        self.hierarchy.roots)
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

struct FromIndexLookup<'c, 'v, 'r> {
    hierarchy: &'c mut ScopeHierarchy<'v, 'r>,
    stack: Vec<ValueType<'v>>,
}

impl<'c, 'v, 'r> Visitor<'v> for FromIndexLookup<'c, 'v, 'r> {
    type Value = Vec<ValueType<'v>>;
    type Error = EvaluationError<'v>;

    fn visit_string(mut self, expr: &'v Expr, value: &'v StringExpr) -> Result<Self::Value, Self::Error> {
        match value.value.parse::<i32>() {
            Ok(i) => {
                let mut current: Vec<ValueType<'_>> = self.stack.drain(..).collect();
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

struct FromQueryIndexLookup<'c, 'v, 'r> {
    hierarchy: &'c mut ScopeHierarchy<'v, 'r>,
    stack: Vec<ValueType<'v>>,
}

impl<'c, 'v, 'r> Visitor<'v> for FromQueryIndexLookup<'c, 'v, 'r> {
    type Value = Vec<ValueType<'v>>;
    type Error = EvaluationError<'v>;

    fn visit_string(mut self, expr: &'v Expr, value: &'v StringExpr) -> Result<Self::Value, Self::Error> {
        let mut current: Vec<ValueType<'_>> = self.stack.drain(..).collect();
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
        Ok(self.stack)
    }

    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        Err(EvaluationError::UnexpectedExpr(
            format!("When attempting extract assignment statements got unexpected Expr"),
            expr
        ))
    }
}

struct QueryHandler<'c, 'v, 'r> {
    hierarchy: &'c mut ScopeHierarchy<'v, 'r>,
    stack: Vec<ValueType<'v>>,
}


impl<'c, 'v, 'r> Visitor<'v> for QueryHandler<'c, 'v, 'r> {
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

    fn visit_string(mut self, expr: &'v Expr, value: &'v StringExpr) -> Result<Self::Value, Self::Error> {
        if self.stack.is_empty() {
            expr.accept(FindFromDataFiles{hierarchy: self.hierarchy, stack: self.stack})
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
                        for each in values.iter() {
                            match each {
                                ValueType::DataValue(v) =>
                                    self.map_value(top.clone(), expr, *v, map)?,

                                ValueType::LiteralValue(v) =>
                                    self.map_literal_expr(top.clone(), expr, v, map)?,

                                _ => {
                                    self.hierarchy.reporter.report_mismatch_value_traversal(
                                        each.clone(),
                                        "",
                                        expr
                                    )?;
                                }
                            }
                        }
                    }
                    _ => {
                        self.hierarchy.reporter.report_missing_value(
                            top,
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

impl<'c, 'v, 'r> QueryHandler<'c, 'v, 'r> {
    fn map_value(&mut self,
                 top: ValueType<'v>,
                 expr: &'v Expr,
                 key: &'v Value,
                 map: &'v indexmap::IndexMap<String, Value>) -> Result<(), EvaluationError<'v>> {
        match key {
            Value::String(value, _) => {
                self.map_key_value(top, expr, value.as_str(), map)?;
            },

            Value::List(list, _) => {
                for each in list {
                    self.map_value(top.clone(), expr, each, map)?;
                }
            },

            _ => {
                self.hierarchy.reporter.report_missing_value(
                    top,
                    "",
                    expr
                )?;
            }
        }
        Ok(())
    }

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
                self.hierarchy.reporter.report_missing_value(
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
                self.hierarchy.reporter.report_mismatch_value_traversal(
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

struct BinaryOperationsHandler<'c, 'v, 'r> {
    hierarchy: &'c mut ScopeHierarchy<'v, 'r>,
    stack: Vec<ValueType<'v>>,
}

impl<'c, 'v, 'r> Visitor<'v> for BinaryOperationsHandler<'c, 'v, 'r> {
    type Value = bool;
    type Error = EvaluationError<'v>;

    fn visit_binary_operation(mut self,
                              expr: &'v Expr,
                              value: &'v BinaryExpr) -> Result<Self::Value, Self::Error> {
        if value.operator != BinaryOperator::Equals             ||
           value.operator != BinaryOperator::NotEquals          ||
           value.operator != BinaryOperator::LesserThanEquals   ||
           value.operator != BinaryOperator::Lesser             ||
           value.operator != BinaryOperator::GreaterThanEquals  ||
           value.operator != BinaryOperator::Greater            ||
           value.operator != BinaryOperator::NotIn              ||
           value.operator != BinaryOperator::In {
            return self.visit_any(expr)
        }

        let lhs_query = value.lhs.accept(QueryHandler{hierarchy: self.hierarchy, stack: self.stack.clone()})?;
        let rhs_query = value.rhs.accept(QueryHandler{hierarchy: self.hierarchy, stack: self.stack})?;

        Ok(false)
    }


    fn visit_any(self, expr: &'v Expr) -> Result<Self::Value, Self::Error> {
        todo!()
    }
}

fn check_operator<'v>(
    op: BinaryOperator
    , lhs: Vec<ValueType<'v>>
    , rhs: Vec<ValueType<'v>>) -> Result<bool, EvaluationError<'v>>
{
    let mut result = true;
    for each_lhs in lhs {
        match each_lhs {
            ValueType::LiteralValue(literal) => {
                result &= check_literal_operator(op, literal, &rhs)?;
            },
            rest => {
                let value = match rest {
                    ValueType::DataValue(ref v) => *v,
                    ValueType::ComputedValue(ref v) => v,
                    _ => unreachable!()
                };
            }
        }
    }
    Ok(result)
}


fn check_value_operator<'v>(
    op: BinaryOperator
    , literal: &'v Value
    , rhs: &Vec<ValueType<'v>>) -> Result<bool, EvaluationError<'v>>
{
    let mut result = true;
    for each in rhs {
        match each {
            ValueType::LiteralValue(rhs) => {
                result &= match_operator_value(op, literal, *rhs);
            },
            rest => {
                let value = match rest {
                    ValueType::DataValue(ref v) => *v,
                    ValueType::ComputedValue(ref v) => v,
                    _ => unreachable!()
                };
                result &= match_in_both_values(literal, value);
            }
        }
    }
    Ok(result)
}

fn check_literal_operator<'v>(
    op: BinaryOperator
    , literal: &'v Expr
    , rhs: &Vec<ValueType<'v>>) -> Result<bool, EvaluationError<'v>>
{
    let mut result = true;
    for each in rhs {
        match each {
            ValueType::LiteralValue(rhs) => {
                result &= match_operator_literal(op, literal, *rhs);
            },
            rest => {
                let value = match rest {
                    ValueType::DataValue(ref v) => *v,
                    ValueType::ComputedValue(ref v) => v,
                    _ => unreachable!()
                };
                result &= match_operator_literal_with_value(op, literal, value);
            }
        }
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
        BinaryOperator::Equals => lhs_literal == rhs_literal,
        BinaryOperator::NotEquals => lhs_literal != rhs_literal,
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

