use guard_lang::{Location, Expr, LangError};
use yaml_rust::parser::{MarkedEventReceiver, Parser};
use yaml_rust::{Event, Yaml};
use yaml_rust::scanner::{Marker, TokenType, TScalarStyle};

use crate::{Value, EvaluationError};
use std::convert::TryFrom;

#[derive(Debug, Default)]
struct StructureReader {
    stack: Vec<Value>,
    documents: Vec<Value>,
    last_container_index: Vec<usize>,
    func_support_index: Vec<(usize, (String, Location))>,
}

impl StructureReader {
    fn new() -> StructureReader {
        StructureReader::default()
    }
}

impl MarkedEventReceiver for StructureReader {
    fn on_event(&mut self, ev: Event, mark: Marker) {
        let line = mark.line() as u32;
        let col = mark.col();

        match ev {
            Event::StreamStart |
            Event::StreamEnd   |
            Event::DocumentStart => {},

            Event::DocumentEnd => {
                self.documents.push(self.stack.pop().unwrap());
                self.stack.clear();
                self.last_container_index.clear();
            },

            Event::MappingStart(..) => {
                self.stack.push(
                    Value::Map(
                        indexmap::IndexMap::new(),
                        Location::new(line, col))
                );
                self.last_container_index.push(self.stack.len()-1);
            },

            Event::MappingEnd => {
                let map_index = self.last_container_index.pop().unwrap();
                let mut key_values: Vec<Value> = self.stack.drain(map_index+1..).collect();
                let map = match self.stack.last_mut().unwrap() {
                    Value::Map(map, _) => map,
                    _ => unreachable!()
                };
                while !key_values.is_empty() {
                    let key = key_values.remove(0);
                    let value = key_values.remove(0);
                    let key_str = match key {
                        Value::String(val, loc) => (val, loc),
                        _ => unreachable!()
                    };
                    map.insert(key_str, value);
                }
            },

            Event::SequenceStart(0, tag) => {
                if let Some(TokenType::Tag(handle, suffix)) = &tag {
                    if handle == "!" {
                        let location = Location::new(line, col);
                        match Self::handle_sequence_value_func_ref(location.clone(), suffix) {
                            Some(value) => {
                                self.stack.push(value);
                                let fn_ref = Self::short_form_to_long(suffix);
                                self.func_support_index.push((self.stack.len()-1, (fn_ref.to_owned(), Location::new(line, col))));
                            },
                            None => {}
                        }
                    }
                }
                self.stack.push(
                    Value::List(vec![], Location::new(line, col))
                );
                self.last_container_index.push(self.stack.len()-1);
            },

            Event::SequenceEnd => {
                let array_idx = self.last_container_index.pop().unwrap();
                let values: Vec<Value> = self.stack.drain(array_idx+1..).collect();
                let array = self.stack.last_mut().unwrap();
                match array {
                    Value::List(vec, _) => vec.extend(values),
                    _ => unreachable!()
                }

                if self.func_support_index.last().map_or(false, |(idx, _)| *idx == array_idx-1) {
                    let (_, fn_ref) = self.func_support_index.pop().unwrap();
                    let array = self.stack.pop().unwrap();
                    let map = self.stack.last_mut().unwrap();
                    match map {
                        Value::Map(map, _) => {
                            let _ = map.insert(fn_ref, array);
                        },
                        Value::BadValue(..) => {},
                        _ => unreachable!()
                    }
                }
            }

            Event::Scalar(val, stype, _, token) => {
                //let path = self.create_path(mark);
                let location = Location::new(line, col);
                let path_value =
                    if let Some(TokenType::Tag(ref handle, ref suffix)) = token {
                        if handle == "!!" {
                            Self::handle_type_ref(val, location, suffix.as_ref())
                        }
                        else if handle == "!" {
                            Self::handle_single_value_func_ref(val.clone(), location.clone(), suffix.as_ref())
                                .map_or(
                                    Value::String(val, location),
                                    std::convert::identity
                                )
                        }
                        else {
                            Value::String(val, location)
                        }
                    } else if stype != TScalarStyle::Plain {
                        Value::String(val, location)
                    }
                    else {
                        match Yaml::from_str(&val) {
                            Yaml::Integer(i) => Value::Int(i, location),
                            Yaml::Real(_) => val.parse::<f64>().ok().map_or(
                                Value::BadValue(val, location.clone()),
                                |f| Value::Float(f, location)
                            ),
                            Yaml::Boolean(b) => Value::Bool(b, location),
                            Yaml::String(s) => Value::String(s, location),
                            Yaml::Null => Value::Null(location),
                            _ => Value::String(val, location)
                        }
                    };
                self.stack.push(path_value);
            },

            _ => todo!()
        }
    }
}

impl StructureReader {

    fn short_form_to_long(fn_ref: &str) -> &'static str {
        match fn_ref {
            "Ref"           => "Ref",
            "GetAtt"        => "Fn::GetAtt",
            "Base64"        => "Fn::Base64",
            "Sub"           => "Fn::Sub",
            "GetAZs"        => "Fn::GetAZs",
            "ImportValue"   => "Fn::ImportValue",
            "Condition"     => "Condition",
            "RefAll"        => "Fn::RefAll",
            "Select"            => "Fn::Select",
            "Split"             => "Fn::Split",
            "Join"              => "Fn::Join",
            "FindInMap"         => "Fn::FindInMap",
            "And"               => "Fn::And",
            "Equals"            => "Fn::Equals",
            "Contains"          => "Fn::Contains",
            "EachMemberIn"      => "Fn::EachMemberIn",
            "EachMemberEquals"  => "Fn::EachMemberEquals",
            "ValueOf"           => "Fn::ValueOf",
            "If"                => "Fn::If",
            "Not"               => "Fn::Not",
            "Or"                => "Fn::Or",
            _ => unreachable!()
        }
    }

    fn handle_single_value_func_ref(
        val: String,
        loc: Location,
        fn_ref: &str) -> Option<Value>
    {
        match fn_ref {
            "Ref"           |
            "Base64"        |
            "Sub"           |
            "GetAZs"        |
            "ImportValue"   |
            "GetAtt"        |
            "Condition"     |
            "RefAll" => {
                let mut map = indexmap::IndexMap::new();
                let fn_ref = Self::short_form_to_long(fn_ref);
                map.insert((fn_ref.to_string(), loc.clone()), Value::String(val, loc.clone()));
                Some(Value::Map(map, loc))
            },

            _ => None,
        }
    }

    fn handle_sequence_value_func_ref(
        loc: Location,
        fn_ref: &str) -> Option<Value> {
        match fn_ref {
            "GetAtt"            |
            "Sub"               |
            "Select"            |
            "Split"             |
            "Join"              |
            "FindInMap"         |
            "And"               |
            "Equals"            |
            "Contains"          |
            "EachMemberIn"      |
            "EachMemberEquals"  |
            "ValueOf"           |
            "If"                |
            "Not"               |
            "Or" => {
                let mut map = indexmap::IndexMap::new();
                let fn_ref = Self::short_form_to_long(fn_ref);
                map.insert((fn_ref.to_string(), loc.clone()), Value::Null(loc.clone()));
                Some(Value::Map(map, loc))
            },

            _ => None,
        }
    }

    fn handle_type_ref(
        val: String,
        loc: Location,
        type_ref: &str) -> Value
    {
        match type_ref {
            "bool" => {
                // "true" or "false"
                match val.parse::<bool>() {
                    Err(_) => Value::String(val, loc),
                    Ok(v) => Value::Bool(v, loc)
                }
            }
            "int" => match val.parse::<i64>() {
                Err(_) => Value::BadValue(val, loc),
                Ok(v) => Value::Int(v, loc),
            },
            "float" => match val.parse::<f64>() {
                Err(_) => Value::BadValue(val, loc),
                Ok(v) => Value::Float(v, loc),
            },
            "null" => match val.as_ref() {
                "~" | "null" => Value::Null(loc),
                _ => Value::BadValue(val, loc)
            },
            _ => Value::String(val, loc)
        }
    }
}

pub(crate) fn read_from<'s, 'e>(from_reader: &'s str) -> Result<Value, EvaluationError<'e>> {
    let mut reader = StructureReader::new();
    let mut parser = Parser::new(from_reader.chars());
    match parser.load(&mut reader, false) {
        Ok(_) => Ok(reader.documents.pop().unwrap()),
        Err(e) => {
            println!("Error {}", e);
            let value = guard_lang::parse_json_value(from_reader, "")?;
            Ok(Value::try_from(value)?)
        }
    }
}

impl TryFrom<Expr> for Value {
    type Error = LangError;

    fn try_from(value: Expr) -> std::result::Result<Self, Self::Error> {
        match value {
            Expr::Map(map) => {
                let map = *map;
                let mut index = indexmap::IndexMap::new();
                for (key, each) in map.entries {
                    index.insert((key.value, key.location), Value::try_from(each)?);
                }
                Ok(Value::Map(index, map.location))
            },

            Expr::Array(array) => {
                let array = *array;
                let mut list = Vec::with_capacity(array.elements.len());
                for each in array.elements {
                    list.push(Value::try_from(each)?);
                }
                Ok(Value::List(list, array.location))
            },

            Expr::String(string) => {
                let string = *string;
                Ok(Value::String(string.value, string.location))
            },

            Expr::Int(int) => {
                let int = *int;
                Ok(Value::Int(int.value, int.location))

            },

            Expr::Float(f) => {
                let f = *f;
                Ok(Value::Float(f.value, f.location))
            },

            Expr::Bool(b) => {
                let b = *b;
                Ok(Value::Bool(b.value, b.location))
            },

            Expr::Null(n) => {
                Ok(Value::Null(*n))
            },

            Expr::RangeInt(r) => {
                let r = *r;
                Ok(Value::RangeInt(r.value, r.location))
            },

            Expr::RangeFloat(r) => {
                let r = *r;
                Ok(Value::RangeFloat(r.value, r.location))
            }

            rest => return Err(LangError::ParseError(guard_lang::ParseError::new(
                rest.get_location().clone(),
                format!("Unknown expression for JSON {:?}", rest)
            )))
        }
    }
}

impl<'expr> TryFrom<&'expr Expr> for Value {
    type Error = LangError;

    fn try_from(value: &'expr Expr) -> std::result::Result<Self, Self::Error> {
        match value {
            Expr::Map(map) => {
                let mut index = indexmap::IndexMap::new();
                for (key, each) in &map.entries {
                    index.insert((key.value.clone(), key.location.clone()), Value::try_from(each)?);
                }
                Ok(Value::Map(index, map.location.clone()))
            },

            Expr::Array(array) => {
                let mut list = Vec::with_capacity(array.elements.len());
                for each in &array.elements {
                    list.push(Value::try_from(each)?);
                }
                Ok(Value::List(list, array.location.clone()))
            },

            Expr::String(string) => {
                Ok(Value::String(string.value.clone(), string.location.clone()))
            },

            Expr::Int(int) => {
                Ok(Value::Int(int.value, int.location.clone()))

            },

            Expr::Float(f) => {
                Ok(Value::Float(f.value, f.location.clone()))
            },

            Expr::Bool(b) => {
                Ok(Value::Bool(b.value, b.location.clone()))
            },

            Expr::Null(n) => {
                Ok(Value::Null(n.as_ref().clone()))
            },

            Expr::RangeInt(r) => {
                Ok(Value::RangeInt(r.value.clone(), r.location.clone()))
            },

            Expr::RangeFloat(r) => {
                Ok(Value::RangeFloat(r.value.clone(), r.location.clone()))
            }


            rest => return Err(LangError::ParseError(guard_lang::ParseError::new(
                rest.get_location().clone(),
                format!("Unknown expression for JSON {:?}", rest)
            )))
        }
    }
}

#[cfg(test)]
mod yaml_parsing_tests;

#[cfg(test)]
mod json_parsing_tests;
