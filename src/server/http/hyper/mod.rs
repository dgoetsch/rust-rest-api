extern crate hyper;
extern crate futures;
extern crate serde_json;
extern crate serde;
extern crate log;

use self::futures::future;
use self::futures::Stream;
use self::serde_json::{Value, Error};
use self::hyper::{Body, Request, Response, Server, Method, StatusCode, Chunk};
use self::hyper::rt::{self, Future};
use self::hyper::service::service_fn;
use super::super::super::RestApp;
use super::super::super::api::API;
use self::hyper::Uri;

const PHRASE: &str = "Hello, World!";

type BoxFut = Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>;

fn read_path(uri: &Uri) -> Vec<String> {
    uri.path()
        .clone()
        .split("/")
        .into_iter()
        .map(String::from)
        .collect::<Vec<String>>()
}

fn parse_json(body: Vec<u8>) -> Result<Value, ServerError> {
    serde_json::from_slice(body.as_slice())
        .map_err(ServerError::ParseJson)
}

fn collect_body(chunk: Chunk) -> Vec<u8> {
    chunk
        .iter()
        .cloned()
        .collect::<Vec<u8>>()
}

fn response(status: StatusCode, body: Body) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(body)
        .unwrap()
}

#[derive(Debug)]
enum ServerError {
    Api(super::super::super::api::APIErr),
    ParseJson(Error)
}

fn handle_request(req: Request<Body>) -> BoxFut {
    let app = RestApp { storage_dir: String::from("rest-storage")};
    match req.method() {
        &Method::PUT => {
            let path = read_path(req.uri());

            info!("PUT/{}", path.clone().join("/"));
            let response_fut = req
                .into_body()
                .concat2()
                .map(collect_body)
                .map(move |body| parse_json(body)
                    .and_then(|json| app.put(path, json, String::from("anon")).map_err(ServerError::Api))
                    .or_else(|err| {
                        error!("Service err: {:?}", err);
                        Err(err)
                    })
                    .map(|()| response(StatusCode::OK,Body::empty()))
                    .unwrap_or_else(|err| response(StatusCode::BAD_REQUEST, Body::from("Error"))));

            Box::new(response_fut)
        },
        &Method::GET => {
            let path = read_path(req.uri());

            info!("GET/{}", path.clone().join("/"));
            let response = app.get(path, String::from("anon"))
                .map_err(ServerError::Api)
                .or_else(|err| {
                    error!("Service err: {:?}", err);
                    Err(err)
                })
                .map(|json| response(StatusCode::OK, Body::from(json.to_string())))
                .unwrap_or_else(|err| response(StatusCode::INTERNAL_SERVER_ERROR, Body::from("Error")));

            Box::new(future::ok(response))
        },
        _ => {
            Box::new(future::ok(response(StatusCode::NOT_IMPLEMENTED, Body::empty())))
        },
    }
}


pub fn server() {
    let addr = ([0, 0, 0, 0], 3000).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn(handle_request))
        .map_err(|e| eprintln!("server error: {}", e));

    rt::run(server);
}