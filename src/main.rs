use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Display, Formatter, Debug};
use std::fmt::Result as FmtResult;
use std::str;
use std::str::FromStr;

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
                                Ok(request) => {},
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
            _ => Err(MethodError),
            "GET" => Ok(Self::GET),
            "HEAD" => Ok(Self::HEAD),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "CONNECT" => Ok(Self::CONNECT),
            "OPTIONS" => Ok(Self::OPTIONS),
            "TRACE" => Ok(Self::TRACE),
            "PATCH" => Ok(Self::PATCH),
        }
    }
}

struct MethodError;

impl From<MethodError> for RequestError {
    fn from(_: MethodError) -> Self {
        Self::Method
    }
}

struct Request {
    method: HTTPMethod,
    path:  &str,
    query_string: Option<&str>,
}

impl Request {
    fn from_byte_to_struct(buf: &[u8]) -> Result<Self, String> {
        unimplemented!()
    }
}

impl TryFrom<&[u8]> for Request {
    type Error = RequestError;

    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
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
            q_str = Some(&path[i+1..]);
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