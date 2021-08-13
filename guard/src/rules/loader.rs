//
//
//

use std::path::PathBuf;
use crate::rules::exprs::RulesFile;
use std::collections::{HashMap, hash_map::Entry};
use std::fs::File;
use crate::rules::errors::{Error, ErrorKind};
use std::rc::Rc;

#[derive(Debug)]
pub(crate) struct FileTracker {
    rule_files: HashMap<PathBuf, String>,
    root_path: PathBuf
}


#[derive(Debug)]
pub(crate) struct Loader<'loc> {
    rules: std::cell::RefCell<HashMap<&'loc PathBuf, Rc<RulesFile<'loc>>>>,
    tracker: &'loc FileTracker
}

impl<'loc> Loader<'loc> {
    pub(crate) fn find_rules_from_path(&self, file_path: PathBuf) -> crate::rules::Result<Rc<RulesFile<'loc>>> {
        match self.tracker.rule_files.get_key_value(&file_path) {
            Some((path_buf, name)) => {
                if let Some(rc) = self.rules.borrow().get(path_buf) {
                    return Ok(rc.clone())
                }

                let content = crate::commands::files::read_file_content(File::open(file_path.as_path())?)?;
                let span = super::parser::Span::new_extra(&content, name.as_str());
                let rules = super::parser::rules_file(span)?;
                let rc = Rc::new(rules);
                self.rules.borrow().insert(path_buf, rc.clone());
                Ok(rc)
            },

            None => {
                return Err(Error::new(ErrorKind::MissingValue(format!("Can not find path at {}", file_path.as_path().display()))))
            }
        }
    }

    pub(crate) fn find_rules_file(&self, file: &[String]) -> crate::rules::Result<Rc<RulesFile<'loc>>> {
        let mut file_path = self.tracker.root_path.clone();
        file_path.extend(file);
        self.find_rules_from_path(file_path)
    }
}

