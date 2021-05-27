use clap::{App, Arg, ArgMatches};


use crate::command::Command;
use crate::commands::files::read_file_content;
use crate::rules::Result;
use crate::migrate::parser::{parse_rules_file, RuleLineType, Rule, Clause};
use std::fs::File;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::collections::{HashSet, HashMap};
use crate::rules::errors::{Error, ErrorKind};
use std::path::PathBuf;
use std::str::FromStr;
use itertools::Itertools;
use std::collections::hash_map::Keys;
use indexmap::set::IndexSet;

#[cfg(test)]
#[path = "migrate_tests.rs"]
mod migrate_tests;



#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct Migrate {}

impl Migrate {
    pub(crate) fn new() -> Self {
        Migrate{}
    }
}

impl Command for Migrate {
    fn name(&self) -> &'static str {
        "migrate"
    }


    fn command(&self) -> App<'static, 'static> {
        App::new("migrate")
            .about(r#"Migrates 1.0 rules to 2.0 compatible rules.
"#)
            .arg(Arg::with_name("rules").long("rules").short("r").takes_value(true).help("Provide a rules file").required(true))
            .arg(Arg::with_name("output").long("output").short("o").takes_value(true).help("Write migrated rules to output file").required(false))
    }

    fn execute(&self, app: &ArgMatches<'_>) -> Result<i32> {
        let file_input = app.value_of("rules").unwrap();
        let path = PathBuf::from_str(file_input).unwrap();
        let file_name = path.to_str().unwrap_or("").to_string();
        let file = File::open(file_input)?;

        let mut out= match app.value_of("output") {
            Some(file) => Box::new(File::create(file)?) as Box<dyn std::io::Write>,
            None => Box::new(std::io::stdout()) as Box<dyn std::io::Write>
        };
        match read_file_content(file) {
            Err(e) => {
                println!("Unable read content from file {}", e);
                Err(Error::new(ErrorKind::IoError(e)))
            },
            Ok(file_content) => {
                match parse_rules_file(&file_content, &file_name) {
                    Err(e) => {
                        println!("Could not parse 1.0 rule file: {}. Please ensure the file is valid with the old version of the tool and try again.", file_name);
                        Err(e)
                    },
                    Ok(rules) => {
                        let migrated_rules = migrate_rules(&rules)?;
                        let span = crate::rules::parser::Span::new_extra(&migrated_rules, "");
                        match crate::rules::parser::rules_file(span) {
                            Ok(_rules) => {
                                write!(out,"{}", migrated_rules)?;
                                Ok(0 as i32)
                            },
                            Err(e) => {
                                println!("Could not parse migrated ruleset for file: '{}': {}", &file_name, e);
                                Err(e)
                            }
                        }
                    }
                }
            }
        }
    }
}

fn migrate_rules(rules: &Vec<RuleLineType>) -> Result<String> {
    let (aggr_by_type, mixed_type_clauses) = aggregate_by_type(&rules);
    let migrated_rules = migrate_rules_by_type(&rules, &aggr_by_type)?;
    Ok(format!("{}\n{}", migrated_rules, handle_mixed_types_rule(
        aggr_by_type.keys().map(|s| s.to_string()).collect::<HashSet<String>>(), &mixed_type_clauses)?))
}

pub(crate) fn migrate_rules_by_type(rules: &[RuleLineType],
                                    by_type: &HashMap<String, indexmap::IndexSet<&Clause>>) -> Result<String> {
    let mut migrated = String::new();
    for rule in rules {
        if let RuleLineType::Assignment(assignment) = rule {
            writeln!(&mut migrated, "{}", assignment);
        }
    }

    let mut types = by_type.keys().map(|elem| elem.clone()).collect_vec();
    types.sort();
    for each_type in &types {
        let snake_cased_name = each_type.to_lowercase().replace("::", "_");
        writeln!(&mut migrated, "let {} = Resources.*[ Type == \"{}\" ]", snake_cased_name, each_type);
        writeln!(&mut migrated, "rule {name}_checks WHEN %{name} NOT EMPTY {{", name=snake_cased_name);
        writeln!(&mut migrated, "    %{} {{", snake_cased_name);
        for each_clause in by_type.get(each_type).unwrap() {
            writeln!(&mut migrated, "        {}", *each_clause);
        }
        writeln!(&mut migrated, "    }}\n}}\n");
    }

    Ok(migrated)
}

pub(crate) fn handle_mixed_types_rule(types_addressed: HashSet<String>,
                                      rules: &indexmap::IndexSet<&Clause>) -> Result<String> {
    let mut types_to_be_addressed = HashSet::with_capacity(types_addressed.len());
    let mut mixed_rules = String::new();
    if !rules.is_empty() {
        for each in rules {
            for rule in &each.rules {
                match rule {
                    Rule::Basic(br) => {
                        if !types_addressed.contains(&br.type_name) {
                            types_to_be_addressed.insert(br.type_name.clone());
                        }
                    },
                    Rule::Conditional(cr) => {
                        if !types_addressed.contains(&cr.type_name) {
                            types_to_be_addressed.insert(cr.type_name.clone());
                        }
                    },
                };
            }
        }
        for each_type in types_to_be_addressed {
            let snake_cased_name = each_type.to_lowercase().replace("::", "_");
            writeln!(&mut mixed_rules, "let {} = Resources.*[ Type == \"{}\" ]", snake_cased_name, each_type);
        }

        writeln!(&mut mixed_rules, "rule mixed_types_checks {{");
        for (idx, each) in rules.iter().enumerate() {
            for (idx, inner_rule) in each.rules.iter().enumerate() {
                match inner_rule {
                    Rule::Basic(br) => {
                        let snake_case_name = br.type_name.to_lowercase().replace("::", "_");
                        writeln!(&mut mixed_rules, "    WHEN %{name} NOT EMPTY {{", name=snake_case_name);
                        writeln!(&mut mixed_rules, "        %{name} {{", name=snake_case_name);
                        writeln!(&mut mixed_rules, "            {}", br);
                        writeln!(&mut mixed_rules, "          }}");
                        write!(&mut mixed_rules, "    }}");
                    },
                    Rule::Conditional(cr) => {
                        let snake_case_name = cr.type_name.to_lowercase().replace("::", "_");
                        writeln!(&mut mixed_rules, "    WHEN %{name} NOT EMPTY {{", name=snake_case_name);
                        writeln!(&mut mixed_rules, "        %{name} {{", name=snake_case_name);
                        writeln!(&mut mixed_rules, "            {}", cr);
                        writeln!(&mut mixed_rules, "        }}");
                        write!(&mut mixed_rules, "    }}");
                    }
                }
                if idx != each.rules.len() - 1 {
                    writeln!(&mut mixed_rules, "  OR");
                }
            }
            writeln!(&mut mixed_rules, "\n");
        }
        writeln!(&mut mixed_rules, "}}");
    }
    Ok(mixed_rules)
}

fn all_rules_are_same_type(rules: &[Rule]) -> (bool, Option<String>) {
    let mut set = rules.iter().map(|r| match r {
        Rule::Basic(br) => br.type_name.clone(),
        Rule::Conditional(cr) => cr.type_name.clone()
    }).collect::<indexmap::IndexSet<String>>();
    (set.len() == 1, Some(set.pop().unwrap()))
}

pub(crate) fn aggregate_by_type(rules: &Vec<RuleLineType>) -> (HashMap<String, indexmap::IndexSet<&Clause>>, indexmap::IndexSet<&Clause>) {
    let mut by_type = HashMap::with_capacity(rules.len());
    let mut mixed_type_clauses = indexmap::IndexSet::new();
    for rule in rules {
        if let RuleLineType::Clause(clause) = rule {
            match all_rules_are_same_type(&clause.rules) {
                (true, Some(name)) => {
                    by_type.entry(name).or_insert(indexmap::IndexSet::new()).insert(clause);
                },

                (_, _) => {
                    mixed_type_clauses.insert(clause);
                }
            }
        }
    }
    (by_type, mixed_type_clauses)
}
