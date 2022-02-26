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
