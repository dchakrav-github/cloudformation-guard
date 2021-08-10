//
//
//

use std::path::PathBuf;
use crate::rules::exprs::RulesFile;
use std::collections::{HashMap, hash_map::Entry};
use crate::rules::PackageLoader;
use std::fs::File;
use crate::rules::errors::{Error, ErrorKind};

#[derive(Debug)]
struct FileTracker {
    rule_files: HashMap<PathBuf, String>,
    root_path: PathBuf
}


#[derive(Debug)]
struct Loader<'loc> {
    rules: HashMap<&'loc PathBuf, RulesFile<'loc>>,
    tracker: &'loc FileTracker
}

impl<'loc> PackageLoader<'loc> for Loader<'loc> {
    fn find_rules_file(&mut self, file: &[String]) -> crate::rules::Result<&RulesFile<'loc>> {
        let relative_path : PathBuf = file.iter().collect();
        let mut file_path = self.tracker.root_path.clone();
        file_path.extend(file);
        match self.tracker.rule_files.get_key_value(&relative_path) {
            Some((path_buf, name)) => {
                match self.rules.entry(path_buf) {
                    Entry::Occupied(entry) => {
                        Ok(&*entry.into_mut())
                    },

                    Entry::Vacant(vacant) => {
                        let content = crate::commands::files::read_file_content(File::open(file_path)?)?;
                        let span = super::parser::Span::new_extra(&content, name.as_str());
                        let rules = super::parser::rules_file(span)?;
                        Ok(&*vacant.insert(rules))
                    }
                }
            },

            None => {
                return Err(Error::new(ErrorKind::MissingValue(format!("Can not find path at {}", file_path.as_path().display()))))
            }
        }
    }
}

