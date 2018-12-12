
extern crate serde_json;

use self::serde_json::Value;
use self::serde_json::Number;


use self::serde_json::Map;
use self::super::RestApp;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
const NULL_TYPE: &str = "NULL";
const BOOL_TYPE: &str = "BOOL";
const STRING_TYPE: &str = "STRING";
const NUMBER_TYPE: &str = "NUMBER";
const ARRAY_TYPE: &str = "ARRAY";
const OBJECT_TYPE: &str = "OBJECT";
const CLASS_TYPE: &str = "CLASS";
const CLASS_FILE_NAME: &str = "__class_declaration__";
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;
#[derive(Debug)]
pub enum APIErr {
    Aggregate(Vec<APIErr>),
    IO(std::io::Error),
    Deserialize(bodyparser::BodyError),
    EmptyRequest,
}

pub trait API {
    fn root_path(&self) -> Vec<String>;

    fn get(&self, path: Vec<String>, user: String) -> Result<Value, APIErr> {
        let mut path_buf = PathBuf::new();
        self.root_path().into_iter().for_each(|p| path_buf.push(p));
        path.into_iter().for_each(|p| path_buf.push(p));
        self.read_path(path_buf, user)
    }

    fn read_path(&self, path: PathBuf, user: String) -> Result<Value, APIErr> {
       fs::metadata(path.clone())
            .map_err(APIErr::IO)
            .map(|meta| meta.file_type().is_dir())
            .and_then(|is_dir| match is_dir {
                false => self.read_field(path.clone()),
                true => self.read_collection(path.clone(), user)
            }) 
    }


    fn put(&self, path: Vec<String>, value: Value, user: String) -> Result<(), APIErr> {
        let mut path_buf = PathBuf::new();
        self.root_path().into_iter().for_each(|p| path_buf.push(p));
        path.into_iter().for_each(|p| path_buf.push(p));
        self.write_path(path_buf, value, user)
    }

    fn write_path(&self, path: PathBuf, value: Value, user: String) -> Result<(), APIErr> {
        match value {
            Value::Null => self.write_field(path, String::from("null"), NULL_TYPE),
            Value::Bool(b) => self.write_field(path, b.to_string(), BOOL_TYPE),
            Value::String(s) =>         self.write_field(path, s, STRING_TYPE),
            Value::Number(n) =>  self.write_field(path, n.to_string(), NUMBER_TYPE),
            Value::Object(o) => {
               self.ensure_path(path.clone())
                    .and_then(|()| self.write_class_file(path.clone(), OBJECT_TYPE))
                    .map(|()| o.into_iter().collect())
                    .map(|m| self.write_collection_members(path.clone(), m, user))
                    .and_then(|errs| self.aggregate(errs))
            },
            Value::Array(a) => {
                self.ensure_path(path.clone())
                    .and_then(|()| self.write_class_file(path.clone(), ARRAY_TYPE))
                    .map(|()| a.into_iter()
                        .enumerate()
                        .map(|(i, v)| (i.to_string(), v))
                        .collect())
                    .map(|m| self.write_collection_members(path.clone(), m, user))
                    .and_then(|errs| self.aggregate(errs))
            },
        }
    }

    fn write_collection_members(&self, path: PathBuf, m: Vec<(String, Value)>, user: String) -> Vec<APIErr> {
            m.into_iter()
                .flat_map(|(k, v)| {
                    let mut p =  path.clone();
                    p.push(k);
                    self
                        .write_path(p, v, user.clone())
                        .err()
                })
                .collect::<Vec<APIErr>>()
    }

    fn write_class_file(&self, mut path: PathBuf, t: &str) -> Result<(), APIErr> {
        path.push(String::from(CLASS_FILE_NAME));
        self.write_field(path, String::from(t), CLASS_TYPE)
    }

    fn ensure_path(&self, path: PathBuf) -> Result<(), APIErr>;

    fn aggregate(&self, errors: Vec<APIErr>) -> Result<(), APIErr> {
        if errors.is_empty() {
            Ok(())
        } else {
            Err(APIErr::Aggregate(errors))
        }
    }

    fn aggregate_result<T,>(&self, results: Vec<T>, errs: Vec<APIErr>) -> Result<Vec<T>, APIErr> {
        if errs.is_empty() {
            Ok(results)
        } else {
            Err(APIErr::Aggregate(errs))
        }
    }


    fn write_field(&self, path: PathBuf, value: String, t: &str) -> Result<(), APIErr>;
    fn read_field(&self, path: std::path::PathBuf) -> Result<Value, APIErr>;
    fn read_collection(&self, path: PathBuf, user: String) -> Result<Value, APIErr> {
        self.read_collection_class(path.clone())
            .and_then(|class| 
                match class {
                    Class::Array => self.read_array(path.clone(), user.clone()),
                    _ => self.read_object(path.clone(), user.clone())
                })
    }
    fn read_object(&self, path: PathBuf, user: String) -> Result<Value, APIErr>;
    fn read_array(&self, path: PathBuf, user: String) -> Result<Value, APIErr>;
    fn read_collection_class(&self, path: PathBuf) -> Result<Class, APIErr>;
}



#[derive(PartialEq, Debug)]
pub enum Class {
    Null,
    Text,
    Number,
    Bool,
    Object,
    Array
}

impl API for RestApp {
    fn root_path(&self) -> Vec<String> {
        self.storage_dir
            .split("/")
            .map(|p|p.to_string())
            .collect::<Vec<String>>()
    }
    fn ensure_path(&self, path: PathBuf) -> Result<(), APIErr> {
        fs::create_dir_all(path)
            .map_err(APIErr::IO)
    }

    fn write_field(&self, path: PathBuf, value: String, t: &str) -> Result<(), APIErr>{
        fs::File::create(path)
            .and_then(|mut f| f.write_all(format!("{}\n{}\n", t, value).as_bytes()))
            .map_err(APIErr::IO)
    } 

    fn read_field(&self, path: PathBuf) -> Result<Value, APIErr> {
        fs::File::open(path)
            .map_err(APIErr::IO)
            .map(BufReader::new)
            .map(move |mut b| {
                let mut class = String::new();
                let mut value = String::new();
                b.read_line(&mut class);
                b.read_line(&mut value);
                (String::from(class.trim()), String::from(value.trim()))
            })
            .map(|(class, value)|
                match class.clone().as_str() {
                    NULL_TYPE => Value::Null,
                    NUMBER_TYPE => 
                        Number::from_str(value.clone().as_str())
                            .map(Value::Number)
                            .unwrap_or(Value::String(value)),
                    BOOL_TYPE => 
                        <bool as FromStr>::from_str(value.clone().as_str())
                            .map(Value::Bool)
                            .unwrap_or(Value::String(value)),
                    _ =>
                        Value::String(value.to_string())
                }
            )
    }

    fn read_collection_class(&self, path: PathBuf) -> Result<Class, APIErr> {
        let mut path_clone = path.clone();
        path_clone.push(CLASS_FILE_NAME);
        fs::File::open(path_clone)
            .map_err(APIErr::IO)
            .map(BufReader::new)
            .map(move |mut b| {
                let mut line = String::new();
                b.read_line(&mut line);
                line = String::new();
                b.read_line(&mut line);
                String::from(line.trim())
            })
            .map(|class_str| {
                match class_str.as_str() {
                    OBJECT_TYPE => Class::Object,
                    ARRAY_TYPE => Class::Array, 
                    _ => Class::Object
                }
            })
    }

    fn read_object(&self, path: std::path::PathBuf, user: String) -> Result<Value, APIErr> {
        fs::read_dir(path.clone())
            .map(|read_dir| read_dir
                .into_iter()
                .map(|f| f
                    .map_err(|err| APIErr::IO(err))
                    .map(|dir_entry| {
                        let path_buf = dir_entry.path();
                        (path_buf.clone(), self.read_path(path_buf.clone(), user.clone()))
                    })
                    .and_then(|(path_buf, read_result)|
                        read_result.map(|value|
                            path_buf.file_name()
                                .and_then(|os_str| os_str.to_str())
                                .map(|file_name| (file_name.to_string(), value)))))
                .partition(|res| res.is_ok()))
            .map_err(|err| APIErr::IO(err))
            .and_then(|(results, errs): (Vec<Result<Option<(String, Value)>, APIErr>>, Vec<Result<Option<(String, Value)>, APIErr>>)| 
                self.aggregate_result(
                    results.into_iter()
                        .flat_map(|result| result.ok().and_then(|tuple_opt| tuple_opt))                        
                        .filter(|(name, value)| name.as_str() != CLASS_FILE_NAME)
                        .collect(), 
                    errs.into_iter().flat_map(|res| res.err()).collect()))
            .map(|results|
                    Value::Object(results
                        .into_iter()  
                        .collect::<Map<String, Value>>()))
}
    fn read_array(&self, path: PathBuf, user: String) -> Result<Value, APIErr> {
        fs::read_dir(path.clone())
            .map(|read_dir| read_dir
                .into_iter()
                .map(|f| f
                    .map_err(|err| APIErr::IO(err))
                    .map(|dir_entry| {
                        let path_buf = dir_entry.path();
                        (path_buf.clone(), self.read_path(path_buf.clone(), user.clone()))
                    })
                    .and_then(|(path_buf, read_result)|
                        read_result.map(|value|
                            path_buf.file_name()
                                .and_then(|os_str| os_str.to_str())
                                .map(|file_name| (file_name.to_string(), value)))))
                .partition(|res| res.is_ok()))
            .map_err(|err| APIErr::IO(err))
            .and_then(|(results, errs): (Vec<Result<Option<(String, Value)>, APIErr>>, Vec<Result<Option<(String, Value)>, APIErr>>)| 
                self.aggregate_result(
                    results.into_iter()
                        .flat_map(|result| result.ok().and_then(|tuple_opt| tuple_opt))
                        .filter(|(name, value)| name.as_str() != CLASS_FILE_NAME)
                        .collect(), 
                    errs.into_iter().flat_map(|res| res.err()).collect()))
            .map(|mut results|{
                results.sort_by(|(name, _), (name2, _)| name.cmp(name2));
                results
            })
            .map(|results|
                Value::Array(results
                    .into_iter()
                    .map(|(_, value)| value)
                    .collect::<Vec<Value>>()))
    }
}
