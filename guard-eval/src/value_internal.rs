use guard_lang::{Location, Expr, LangError, Visitor, ArrayExpr, MapExpr, StringExpr, RegexExpr, CharExpr, BoolExpr, IntExpr, FloatExpr, RangeIntExpr, RangeFloatExpr, WithinRange};
use yaml_rust::parser::{MarkedEventReceiver, Parser};
use yaml_rust::{Event, Yaml};
use yaml_rust::scanner::{Marker, TokenType, TScalarStyle};

use crate::{Value, EvaluationError};
use std::convert::TryFrom;
use std::cmp::Ordering;

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
                        Value::String(val, _loc) => val,
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
                            let _ = map.insert(fn_ref.0, array);
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
                map.insert(fn_ref.to_string(), Value::String(val, loc.clone()));
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
                map.insert(fn_ref.to_string(), Value::Null(loc.clone()));
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
                    index.insert(key, Value::try_from(each)?);
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
                    index.insert(key.to_string(), Value::try_from(each)?);
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

            Expr::Regex(regex) => {
                Ok(Value::Regex(regex.value.to_string(), regex.location.clone()))
            }

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

impl PartialEq<Expr> for Value {
    fn eq(&self, other: &Expr) -> bool {
        struct Comparator<'s> { value: &'s Value }
        impl<'s> Visitor<'_> for Comparator<'s> {
            type Value = bool;
            type Error = ();

            fn visit_array(self, _expr: &'_ Expr, value: &'_ ArrayExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::List(values, _) => {
                        let mut result = values.len() == value.elements.len();
                        if result {
                            for each_value in values {
                                result &= value.elements.iter().any(|e| each_value == e);
                                if !result { break }
                            }
                        }
                        result
                    },
                    _ => false
                })
            }

            fn visit_map(self, _expr: &'_ Expr, value: &'_ MapExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Map(map, _) => {
                        let mut result = value.entries.len() == map.len();
                        if result {
                            for (each_key, each_value) in map {
                                match value.entries.get(each_key) {
                                    Some(value) => {
                                        result &= each_value == value;
                                        if !result { break }
                                    },
                                    None => {
                                        result = false;
                                        break;
                                    }
                                }
                            }
                        }
                        result
                    },
                    _ => false
                })
            }

            fn visit_null(self, _expr: &'_ Expr, _value: &'_ Location) -> Result<Self::Value, Self::Error> {
                Ok(matches!(self.value, Value::Null(_)))
            }

            fn visit_string(self, _expr: &'_ Expr, value: &'_ StringExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::String(val, _) => &value.value == val,
                    _ => false
                })
            }

            fn visit_regex(self, _expr: &'_ Expr, value: &'_ RegexExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Regex(r, _) => r == &value.value,
                    Value::String(s, _) => {
                        match regex::Regex::new(&value.value) {
                            Ok(regex) => {
                                regex.is_match(s)
                            },
                            Err(_) => false
                        }
                    },
                    _ => false
                })
            }

            fn visit_char(self, _expr: &'_ Expr, value: &'_ CharExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Char(c, _) => *c == value.value,
                    _ => false
                })
            }

            fn visit_bool(self, _expr: &'_ Expr, value: &'_ BoolExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Bool(b, _) => *b == value.value,
                    _ => false,
                })
            }

            fn visit_int(self, _expr: &'_ Expr, value: &'_ IntExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Int(i, _) => *i == value.value,
                    Value::Float(f, _) => (*f as i64) == value.value,

                    _ => false
                })
            }

            fn visit_float(self, _expr: &'_ Expr, value: &'_ FloatExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Int(i, _) => match (*i as f64).partial_cmp(&value.value) {
                        Some(Ordering::Equal) => true,
                        _ => false,
                    },
                    Value::Float(f, _) => match f.partial_cmp(&value.value) {
                        Some(Ordering::Equal) => true,
                        _ => false,
                    },
                    _ => false
                })
            }

            fn visit_range_int(self, _expr: &'_ Expr, value: &'_ RangeIntExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Int(i, _) => i.is_within(&value.value),
                    _ => false,
                })
            }

            fn visit_range_float(self, _expr: &'_ Expr, value: &'_ RangeFloatExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Float(i, _) => i.is_within(&value.value),
                    _ => false,
                })
            }

            fn visit_any(self, expr: &'_ Expr) -> Result<Self::Value, Self::Error> {
                Ok(false)
            }
        }
        match other.accept(Comparator{value: self}) {
            Ok(eval) => eval,
            Err(_) => false
        }
    }
}

impl PartialOrd<Expr> for Value {
    fn partial_cmp(&self, other: &Expr) -> Option<Ordering> {
        struct Comparator<'s> { value: &'s Value }
        impl<'s> Visitor<'_> for Comparator<'s> {
            type Value = Option<Ordering>;
            type Error = ();

            fn visit_null(self, _expr: &'_ Expr, _value: &'_ Location) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Null(_) => Some(Ordering::Equal),
                    _ => None
                })
            }

            fn visit_string(self, _expr: &'_ Expr, value: &'_ StringExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::String(val, _) => value.value.partial_cmp(val),
                    _ => None
                })
            }

            fn visit_regex(self, _expr: &'_ Expr, value: &'_ RegexExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Regex(r, _) => value.value.partial_cmp(r),
                    _ => None,
                })
            }

            fn visit_char(self, _expr: &'_ Expr, value: &'_ CharExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Char(c, _) => value.value.partial_cmp(c),
                    _ => None
                })
            }

            fn visit_bool(self, _expr: &'_ Expr, value: &'_ BoolExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Bool(b, _) => value.value.partial_cmp(b),
                    _ => None
                })
            }

            fn visit_int(self, _expr: &'_ Expr, value: &'_ IntExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Int(i, _) => value.value.partial_cmp(i),
                    _ => None,
                })
            }

            fn visit_float(self, _expr: &'_ Expr, value: &'_ FloatExpr) -> Result<Self::Value, Self::Error> {
                Ok(match self.value {
                    Value::Int(i, _) => (*i as f64).partial_cmp(&value.value),
                    Value::Float(f, _) => f.partial_cmp(&value.value),
                    _ => None,
                })
            }

            fn visit_any(self, expr: &'_ Expr) -> Result<Self::Value, Self::Error> {
                Ok(None)
            }
        }
        match other.accept(Comparator{value: self}) {
            Ok(eval) => eval,
            _ => unreachable!()
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Null(_mine),
                Value::Null(_theirs)) => { Some(Ordering::Equal) },

            (Value::String(mine, _),
                Value::String(theirs, _)) => {
                mine.value.partial_cmp(&theirs.value)
            },

            (Value::Regex(mine, _),
                Value::Regex(theirs, _)) => {
                mine.value.partial_cmp(&theirs.value)
            },

            (Value::Char(mine, _),
                Value::Char(theirs, _)) => {
                mine.value.partial_cmp(&theirs.value)
            },

            (Value::Bool(mine, _),
                Value::Bool(theirs, _)) => {
                mine.value.partial_cmp(&theirs.value)
            },

            (Value::Int(mine, _),
                Value::Int(theirs, _)) => {
                mine.value.partial_cmp(&theirs.value)
            },

            (Value::Float(mine, _),
                Value::Float(theirs, _)) => {
                mine.value.partial_cmp(&theirs.value)
            },

            (Value::RangeInt(mine, _),
                Value::RangeInt(theirs, _)) => {
                mine.value.partial_cmp(&theirs.value)
            },

            (Value::RangeFloat(mine, _),
                Value::RangeFloat(theirs, _)) => {
                mine.value.partial_cmp(&theirs.value)
            },

            _ => None
        }
    }
}

#[cfg(test)]
mod partial_ord_tests;


#[cfg(test)]
mod yaml_parsing_tests;

#[cfg(test)]
mod json_parsing_tests;
