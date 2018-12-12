extern crate serde_json;
extern crate rand;
use std::fs;

#[macro_use]
use std::path::PathBuf;
use self::serde_json::json;
use self::serde_json::Value;
use super::super::api::API;
use super::super::api::Class;
use super::super::RestApp;
use self::rand::Rng;

fn random_string(len: usize) -> String {
    rand::thread_rng()
        .gen_ascii_chars()
        .take(len)
        .collect::<String>()
}

fn to_path_buf(path: Vec<String>) -> PathBuf {
        let mut path_buf = PathBuf::new();
        path.into_iter().for_each(|p| path_buf.push(p));
        path_buf
}

#[test]
fn test_read_collection_class() {
    let storage_dir = random_string(16) ;
    let api = RestApp { storage_dir: storage_dir.clone() };
    let json = json!({
        "name":"test_obj"
    });
    let user = String::from("tilda");
    
    let path = ["root".to_string(), "domain".to_string()].to_vec();
    let path_buf = to_path_buf(path.clone());


    let put_result = api.put(path.clone(), json, user);
    let put_ok = put_result
        .map_err(|e|{
            println!("put err: {:?}", e);
            e
        })
        .is_ok();
    assert!(put_ok);

    let full_path = [storage_dir.clone(), "root".to_string(), "domain".to_string()].to_vec();
    let full_path_buf = to_path_buf(full_path.clone());

    let class_result = api.read_collection_class(full_path_buf.clone());
    
    assert!(class_result
        .map(|v| assert_eq!(v,  Class::Object))
        .map_err(|e|{
            println!("get class err: {:?}", e);
            e
        })
    .is_ok());


    fs::remove_dir_all(storage_dir);
}

#[test]
fn test_read_field() {
    let storage_dir = random_string(16) ;
    let api = RestApp { storage_dir: storage_dir.clone() };

    let api = RestApp { storage_dir: storage_dir.clone() };
    let json = json!({
        "name":"test_obj"
    });
    let user = String::from("tilda");
    let path = ["root".to_string(), "domain".to_string()].to_vec();
    let path_buf = to_path_buf(path.clone());
    let put_result = api.put(path.clone(), json.clone(), user.clone());
    let put_ok = put_result.is_ok();
    assert!(put_ok);
    let full_path = [storage_dir.clone(), "root".to_string(), "domain".to_string(), "name".to_string()].to_vec();
    let full_path_buf = to_path_buf(full_path.clone());
    let object = api.read_path(full_path_buf.clone(), user.clone());
    assert!(object
        .map(|v| assert_eq!(v, Value::String("test_obj".to_string())))
        .map_err(|e|{
            println!("get err: {:?}", e);
            e
        })
        .is_ok());
    
    
    fs::remove_dir_all(storage_dir);
}

#[test]
fn test_read_object() {
    let storage_dir = random_string(16) ;
    let api = RestApp { storage_dir: storage_dir.clone() };

    let json = json!({
        "name":"test_obj",
        "age": 43,
        "days_past_end":-78,
        "lat":48.9999,
        "long":-88.594938,
        "cant_even":true,
        "conquests":[
            "birth",
            {"event":"learned","activity":1,"children":["frank", {"nodeId":1}]},
            ["data","is",true,"or",42]
        ]
    });
    let user = String::from("tilda");
    let path = ["root".to_string(), "domain".to_string()].to_vec();
    let path_buf = to_path_buf(path.clone());
    let put_result = api.put(path.clone(), json.clone(), user.clone());
    let put_ok = put_result.is_ok();
    assert!(put_ok);
    let full_path = [storage_dir.clone(), "root".to_string(), "domain".to_string()].to_vec();
    let full_path_buf = to_path_buf(full_path.clone());
    let object = api.read_path(full_path_buf.clone(), user.clone());
    assert!(object
        .map(|v| assert_eq!(v, json.clone()))
        .map_err(|e|{
            println!("get err: {:?}", e);
            e
        })
        .is_ok());
    
    
    fs::remove_dir_all(storage_dir);
}