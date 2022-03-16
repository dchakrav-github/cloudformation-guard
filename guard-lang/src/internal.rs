//
// Internal Helpers
//

use super::exprs::*;
use std::cmp::Ordering;

impl PartialEq<i64> for IntExpr {
    fn eq(&self, other: &i64) -> bool {
        self.value() == *other
    }
}

impl PartialEq<f64> for FloatExpr {
    fn eq(&self, other: &f64) -> bool {
        match self.value().partial_cmp(other) {
            Some(Ordering::Equal) => true,
            _ => false
        }
    }
}

impl PartialEq for RuleExpr {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name &&
            self.parameters == other.parameters &&
            self.block == other.block &&
            self.when == other.when
    }
}

impl PartialEq for BlockExpr {
    fn eq(&self, other: &Self) -> bool {
        self.assignments == other.assignments &&
            self.clause == other.clause
    }
}

impl PartialEq for LetExpr {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name             &&
            self.key == other.key           &&
            self.value == other.value
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::File(mine),
                Expr::File(theirs)) => {
                mine.name == theirs.name                        &&
                    mine.rules == theirs.rules                  &&
                    mine.assignments == theirs.assignments
            },

            (Expr::Rule(mine),
                Expr::Rule(theirs)) => {
                mine.name == theirs.name                    &&
                    mine.when == theirs.when                &&
                    mine.parameters == theirs.parameters    &&
                    mine.block == theirs.block
            },

            (Expr::RuleClause(mine),
                Expr::RuleClause(theirs)) => {
                mine.name == theirs.name &&
                    mine.parameters == theirs.parameters &&
                    mine.message == theirs.message
            },

            (Expr::Let(mine),
                Expr::Let(theirs)) => {
                mine.name == theirs.name &&
                    mine.value == theirs.value &&
                    mine.key == theirs.key
            },

            (Expr::When(mine),
                Expr::When(theirs)) => {
                mine.when == theirs.when &&
                    mine.block == theirs.block
            },

            (Expr::Select(mine),
                Expr::Select(theirs)) => {
                mine.parts == theirs.parts
            },

            (Expr::BinaryOperation(mine),
                Expr::BinaryOperation(theirs)) => {
                mine.operator == theirs.operator    &&
                    mine.lhs == theirs.lhs          &&
                    mine.rhs == theirs.rhs          &&
                    mine.message == theirs.message
            },

            (Expr::UnaryOperation(mine),
                Expr::UnaryOperation(theirs)) => {
                mine.operator == theirs.operator        &&
                    mine.message == theirs.message      &&
                    mine.expr == theirs.expr
            },

            (Expr::Array(mine),
                Expr::Array(theirs)) => {
                mine.elements == theirs.elements
            },

            (Expr::Map(mine),
                Expr::Map(theirs)) => {
                mine.entries == theirs.entries
            },

            (Expr::Null(_mine),
                Expr::Null(_theirs)) => { true },

            (Expr::String(mine),
                Expr::String(theirs)) => {
                mine.value == theirs.value
            },

            (Expr::Regex(mine),
                Expr::Regex(theirs)) => {
                mine.value == theirs.value
            },

            (Expr::Char(mine),
                Expr::Char(theirs)) => {
                mine.value == theirs.value
            },

            (Expr::Bool(mine),
                Expr::Bool(theirs)) => {
                mine.value == theirs.value
            },

            (Expr::Int(mine),
                Expr::Int(theirs)) => {
                mine.value == theirs.value
            },

            (Expr::Float(mine),
                Expr::Float(theirs)) => {
                match mine.value.partial_cmp(&theirs.value) {
                    Some(Ordering::Equal) => true,
                    _ => false
                }
            },

            (Expr::RangeInt(mine),
                Expr::RangeInt(theirs)) => {
                mine.value == theirs.value
            },

            (Expr::RangeFloat(mine),
                Expr::RangeFloat(theirs)) => {
                mine.value == theirs.value
            },

            (Expr::Filter(mine),
                Expr::Filter(theirs)) => {
                mine == theirs
            },

            (Expr::Variable(mine),
                Expr::Variable(theirs)) => {
                mine.value == theirs.value
            },

            (Expr::VariableReference(mine),
                Expr::VariableReference(theirs)) => {
                mine.value == theirs.value
            },

            (Expr::Block(mine),
                Expr::Block(theirs)) => {
                mine.select.parts == theirs.select.parts        &&
                    mine.message == theirs.message              &&
                    mine.block == theirs.block
            },

            _ => false,
        }
    }
}