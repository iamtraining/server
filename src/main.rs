use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write, Result as IoResult};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Display, Formatter, Debug};
use std::fmt::Result as FmtResult;
use std::str;
use std::str::FromStr;
use std::collections::HashMap;
use std::env;
use std::fs;

fn main() {
    let default = format!("{}/view", env!("CARGO_MANIFEST_DIR"));
    let view_path = env::var("VIEW_PATH").unwrap_or(default);
    println!("view path: {}", view_path);
    let server = Server::new("127.0.0.1:3000".to_string());
    server.run(HttpHandler::new(view_path));
}
// GET /rawr?r=a&w=r&x=d HTTP/1.1\r\n ....

struct Server {
    addr: String,
}

impl Server {
    fn new(addr: String) -> Self {
        Self {
            addr,
        }
    }
    fn run(self, mut handler: impl Handler) {
        let listener = TcpListener::bind(&self.addr).unwrap();

        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0; 1024];
                    match stream.read(&mut buf) {
                        Ok(_) => {
                            println!("request received: \n{}", String::from_utf8_lossy(&buf));
                            //Request::try_from(&buf as &[u8]);
                            let response = match Request::try_from(&buf[..]) {
                                Ok(request) => {
                                    handler.handle(&request)
                                }
                                Err(err) => {
                                    handler.handle_bad_request(&err)
                                }
                            };
                            if let Err(err) = response.send(&mut stream) {
                                println!("sending responsa failure {}", err)
                            }
                        },
                        Err(err) => println!("cant read {}", err)

                    }
                }
                Err(err) => println!("cant establish a connaction{}", err)
            }
        }
    }
}

#[derive(Debug)]
enum HTTPMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl FromStr for HTTPMethod {
    type Err = MethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::GET),
            "HEAD" => Ok(Self::HEAD),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "CONNECT" => Ok(Self::CONNECT),
            "OPTIONS" => Ok(Self::OPTIONS),
            "TRACE" => Ok(Self::TRACE),
            "PATCH" => Ok(Self::PATCH),
            _ => Err(MethodError),
        }
    }
}

struct MethodError;

impl From<MethodError> for RequestError {
    fn from(_: MethodError) -> Self {
        Self::Method
    }
}

#[derive(Debug)]
pub struct Request<'buf> {
    method: HTTPMethod,
    path:  &'buf str,
    query_string: Option<QueryString<'buf>>,
}

impl<'buf> Request<'buf> {
    fn from_byte_to_struct(buf: &[u8]) -> Result<Self, String> {
        unimplemented!()
    }
    fn path(&self) -> &str {
        &self.path
    }
    fn method(&self) -> &HTTPMethod {
        &self.method
    }
    fn query_string(&self) -> Option<&QueryString> {
        self.query_string.as_ref()
    }
}

impl<'buf> TryFrom<&'buf [u8]> for Request<'buf> {
    type Error = RequestError;

    fn try_from(buf: &'buf [u8]) -> Result<Request<'buf>, Self::Error> {
        /*
        match str::from_utf8(&buf) {
            Ok(request) => {},
            Err(err) => println!("cannot convert to a string slice {}", err)
        }
        */

        /*
        match str::from_utf8(&buf).or(Err(RequestError::Encode)) {
            Ok(request) => {},
            Err(err) => return Err(err) ,
        }
        */

        let req = str::from_utf8(&buf).or(Err(RequestError::Encode))?;

        /*
        match next_word(&req) {
            Some((method, req)) => {},
            None => return Err(RequestError::Request),
        }
        */

        // GET /rawr?r=a&w=r&x=d HTTP/1.1\r\n ....
        let (method, req) = next_word(&req).ok_or(RequestError::Request)?;
        let (mut path, req) = next_word(&req).ok_or(RequestError::Request)?;
        let (protocol, _) = next_word(&req).ok_or(RequestError::Request)?; // посмотреть что будет с андерскором в дебаге

        if protocol != "HTTP/1.1" {
            return Err(RequestError::Protocol);
        }

        let method: HTTPMethod = method.parse()?;

        let mut q_str = None;

        if let Some(i) = path.find('?') {
            q_str = Some(QueryString::from((&path[i+1..]))); 
            path = &path[..i];
        }

        Ok(Request{
            method,
            path: path,
            query_string: q_str,
        })
    }
}

pub enum RequestError {
    Request,
    Protocol,
    Encode,
    Method,
}

impl RequestError {
    fn msg(&self) -> &str {
        match self {
            Self::Request => "invalid request",
            Self::Protocol =>"invalid protocol",
            Self::Encode => "invalid encoding",
            Self::Method => "invalid method",
        }
    }
}

impl Error for RequestError {}

impl Display for RequestError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}",self.msg())
    }
}

impl Debug for RequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.msg())
    }
}
fn next_word(str: &str) -> Option<(&str, &str)> {
    let mut iter = str.chars();
    
    for (i, v) in iter.enumerate() {
        if v == ' ' || v == '\r' {
            return Some((&str[..i], &str[i+1..]));
        }
    }
    
    None
}

#[derive(Debug)]

struct QueryString<'buf> {
    params: HashMap<&'buf str, Value<'buf>>
}

#[derive(Debug)]

enum Value<'buf> {
    Single(&'buf str),
    Multiple(Vec<&'buf str>),
}

impl <'buf> QueryString<'buf> {
    fn parse(&self, key:&str) -> Option<&Value> {
        self.params.get(key)
    }
}

// a=1&b=2&c&d=&e===&d=7&d=abc
impl<'buf> From<&'buf str> for QueryString<'buf> {
    fn from(s: &'buf str) -> Self {
        let mut map = HashMap::new();

        for sub_str in s.split('&') {
            let mut k = sub_str;
            let mut v = "";
            if let Some(i) = sub_str.find('=') {
                k = &sub_str[..i];
                v = &sub_str[i+1..];
            }

            map.entry(k)
            .and_modify(|cur: &mut Value| match cur {
                Value::Single(prev) => {
                    *cur = Value::Multiple(vec![prev, v])
                },
                Value::Multiple(vec) => vec.push(v)
            })
            .or_insert(Value::Single(v));
        }

        QueryString {
            params:map,
        }
    }
}
pub struct Response {
    status_code: StatusCode,
    body: Option<String>,
}

impl Response {
    fn new(code: StatusCode, body: Option<String>) -> Self {
        Response {
            status_code: code,
            body
        }
    }
    fn send(&self, stream: &mut impl Write) -> IoResult<()> {
        let body = match &self.body {
            Some(body) => body,
            None => "",
        };
        write!(stream, "HTTP/1.1 {} {}\r\n\r\n{}", self.status_code, self.status_code.msg(), body)
    }
}

#[derive(Clone, Copy)]
enum StatusCode {
    StatusOk = 200,
    StatusBadRequest = 400,
    StatusNotFound = 404,
}


impl StatusCode {
    fn msg(&self) -> &str {
        match self {
            Self::StatusOk => "ok",
            Self::StatusBadRequest => "bad request",
            Self::StatusNotFound => "not found"
        }
    }
}
impl Display for StatusCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", *self as u32)
    }
}

pub trait Handler {
    fn handle(&mut self, request: &Request) -> Response;
    fn handle_bad_request(&mut self, err: &RequestError)-> Response {
        println!("parse request failure {}", err);
        Response::new(StatusCode::StatusBadRequest, None)
    }
}

struct HttpHandler {
    view_path: String
}

impl HttpHandler {
    fn new(view_path: String) -> Self {
        Self {
            view_path
        }
    }
    fn render(&self, view_path: &str) -> Option<String> {
        let path = format!("{}/{}", self.view_path, view_path);
        fs::read_to_string(path).ok()
    }
}

impl Handler for HttpHandler {
    fn handle(&mut self, request: &Request) -> Response {
        match request.method() {
            HTTPMethod::GET => match request.path() {
                "/" => Response::new(StatusCode::StatusOk, self.render("main.html")),
                "/hello" => Response::new(StatusCode::StatusOk, self.render("hello.html")),
                _ => Response::new(StatusCode::StatusNotFound, None)
            }
            _ => Response::new(StatusCode::StatusNotFound, None)
        }
    }
}