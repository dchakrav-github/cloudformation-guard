///
///  Guard Language Grammar and Parser
///

mod rules;

pub use rules::ast::exprs;
pub use rules::ast::visitor;



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
