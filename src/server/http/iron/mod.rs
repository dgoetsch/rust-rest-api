extern crate iron;
extern crate bodyparser;
extern crate persistent;
extern crate serde;
extern crate serde_json;

extern crate router;
extern crate simple_logger;

use self::serde_json::Serializer;
use self::serde_json::Value;
use self::serde::ser::{Serialize, SerializeStruct};
use self::iron::prelude::*;
use self::iron::status;
use self::iron::mime::Mime;
use self::iron::{typemap, AfterMiddleware, BeforeMiddleware};
use self::persistent::Read;
use self::router::Router;
use std::clone::Clone;
use std::collections::HashMap;

use super::super::super::RestApp;
use super::super::super::api;
use super::super::super::api::API;


struct ResponseTime;

impl AfterMiddleware for ResponseTime {
    fn after(&self, req: &mut Request, res: Response) -> IronResult<Response> {
        println!("after!");
        Ok(res)
    }
}

fn get_resource(app: RestApp, req: &mut Request) -> IronResult<Response> {
    let (host, port, path, params) = parse_url(req);
    
    let user = String::from("anon");
    let content_type = "application/json".parse::<Mime>().unwrap();
    app.get(path, user)
        .map(|body| Response::with((content_type, status::Ok, body.to_string())))
        .or_else(|_| Ok(Response::with(status::ImATeapot)))
}


fn to_key_value(string: String) -> Option<(String, String)> {
    let parts = string
        .split("=")
        .map(String::from)
        .collect::<Vec<String>>();

    parts
        .first()
        .map(|key: &String| (
            key.clone(), 
            parts[1..].to_vec().join("=")))
}

fn parse_url(req: &mut Request) -> (String, u16, Vec<String>, HashMap<String, String>) {
    (    
        String::from(format!("{}", req.url.host())),
        req.url.port(),
        req.url.path()
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>(),
        req.url.query()
            .map(String::from)
            .unwrap_or(String::from(""))
            .split("&")
            .map(String::from)
            .flat_map(to_key_value)
            .filter(|(k, _)| !k.is_empty())
            .collect::<HashMap<String, String>>()
    )
}


fn put_resource(app: RestApp, req: &mut Request) ->  IronResult<Response> {   
    let (host, port, path, params) = parse_url(req);

    let user = String::from("anon");
   

    let content_type = "application/json".parse::<Mime>().unwrap();
    req.get::<bodyparser::Json>()
        .map_err(api::APIErr::Deserialize)
        .and_then(|json_opt| json_opt.map(|j|Ok(j)).unwrap_or(Err(api::APIErr::EmptyRequest)))
        .and_then(|json| app.put(path, json, user))
        .map(|()| Response::with((status::Ok)))
        .or_else(|_| Ok(Response::with(status::ImATeapot)))
}

fn server() {
    let app = RestApp { storage_dir: String::from("rest-storage") };
    let mut routes = Router::new();
    let getApp = app.clone();
    let mut get_resource_chain = Chain::new(move |req: &mut Request| get_resource(getApp.clone(), req));
    let putApp = app.clone();
    let mut put_resource_chain = Chain::new(move |req: &mut Request| put_resource(putApp.clone(), req));
    routes.get("**", get_resource_chain, "get");
    routes.put("**", put_resource_chain, "put");
    
    Iron::new(routes).http("0.0.0.0:3000").unwrap();
}