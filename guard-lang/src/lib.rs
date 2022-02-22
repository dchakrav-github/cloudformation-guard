///
///  Guard Language Grammar and Parser
///

mod types;
mod parser;
mod visitor;
mod exprs;

pub use visitor::Visitor;
pub use exprs::*;
pub use types::*;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
