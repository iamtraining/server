use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Display, Formatter, Debug};
use std::fmt::Result as FmtResult;
use std::str;
use std::str::FromStr;
use std::collections::HashMap;

fn main() {
    let server = Server::new("127.0.0.1:3000".to_string());
    server.run();
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
    fn run(self) {
        let listener = TcpListener::bind(&self.addr).unwrap();

        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0; 1024];
                    match stream.read(&mut buf) {
                        Ok(_) => {
                            println!("request received: \n{}", String::from_utf8_lossy(&buf));
                            //Request::try_from(&buf as &[u8]);
                            match Request::try_from(&buf[..]) {
                                Ok(request) => {
                                    dbg!(request);
                                },
                                Err(err) => println!("cant parse a request")
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
struct Request<'buf> {
    method: HTTPMethod,
    path:  &'buf str,
    query_string: Option<QueryString<'buf>>,
}

impl<'buf> Request<'buf> {
    fn from_byte_to_struct(buf: &[u8]) -> Result<Self, String> {
        unimplemented!()
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

enum RequestError {
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

        for s_s in s.split('&') {
            let mut k = s_s;
            let mut v = "";
            if let Some(i) = s.find('=') {
                k = &s_s[..i];
                v = &s_s[i+1..];
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