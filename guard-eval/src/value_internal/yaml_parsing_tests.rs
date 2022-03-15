use crate::EvaluationError;
use std::path::PathBuf;
use std::io::{BufReader, Read};
use anyhow::Context;
use std::convert::TryFrom;

#[test]
fn test_json_yaml_parsing() -> anyhow::Result<()> {
    let file = PathBuf::from("./test-resources");
    let directory = walkdir::WalkDir::new(file);
    for each in directory.follow_links(true) {
        if let Ok(entry) = each {
            if entry.path().is_file() {
                let name = entry.path().file_name().map_or("", |s| s.to_str().unwrap());
                if  name.ends_with(".yaml")    ||
                    name.ends_with(".yml")     ||
                    name.ends_with(".json")    ||
                    name.ends_with(".jsn")     ||
                    name.ends_with(".template") {
                    let mut file_content = String::new();
                    let file = std::fs::File::open(entry.path()).context(
                        format!("Unable to open file {:?}", entry.path())
                    )?;
                    let mut buf_reader = BufReader::new(file);
                    buf_reader.read_to_string(&mut file_content)?;
                    let value = super::read_from(&file_content)
                        .context(format!("Unable to parse file {:?}", entry.path()))?;
                    match value {
                        crate::Value::Map(..) => {},
                        _ => unreachable!()
                    }
                }
            }
        }
    }
    Ok(())
}

