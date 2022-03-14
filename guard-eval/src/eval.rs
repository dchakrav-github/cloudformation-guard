use crate::{
    EvaluationError,
    Value
};

use guard_lang::{
    FileExpr,
};

use std::collections::HashMap;
use std::rc::Rc;

pub fn evaluate(rule_file: FileExpr,
                data: Value) -> Result<(), EvaluationError>
{
    todo!()
}

type ScopeHierarchy<'value, 'hierarchy> = HashMap<Rc<String>, Scope<'value, 'hierarchy>>;

struct Scope<'value, 'hierarchy> {
    root_value: &'value Value,
    hierarchy:  &'hierarchy mut ScopeHierarchy<'value, 'hierarchy>
}

