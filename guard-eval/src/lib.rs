mod data_parser;
mod files;

mod types;
mod eval;
mod value_internal;

pub use types::*;

///
///  Guard Evaluator for files
///


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
