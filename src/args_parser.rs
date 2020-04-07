use std::str::FromStr;

use argh::FromArgs;

#[derive(Debug, PartialEq)]
pub struct ValuePair {
    pub key: String,
    pub value: String,
}

/// Header with key value pair
#[derive(Debug, PartialEq)]
pub struct Header {
    pub key: http::header::HeaderName,
    pub value: http::header::HeaderValue,
}

impl FromStr for Header {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = s.split("=").collect();
        if v.len() != 2 {
            return Err("invalid argument should be in  form of key=value".to_owned());
        }
        let head_name = match http::header::HeaderName::from_bytes(v[0].as_bytes()) {
            Ok(head_name) => head_name,
            Err(err) => return Err(err.to_string()),
        };
        let head_value = match http::header::HeaderValue::from_str(v[1]) {
            Ok(head_value) => head_value,
            Err(err) => return Err(err.to_string()),
        };
        return Ok(Header {
            key: head_name,
            value: head_value,
        });
    }
}

#[derive(Debug)]
pub enum Method {
    POST,
    GET,
    PUT,
    PATCH,
    DELETE,
    OPTION,
}

impl FromStr for Method {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "post" => Method::POST,
            "put" => Method::PUT,
            "delete" => Method::DELETE,
            "option" => Method::OPTION,
            "patch" => Method::PATCH,
            "get" => Method::GET,
            _ => Method::GET,
        })
    }
}

impl std::string::ToString for Method {
    fn to_string(&self) -> String {
        match &self {
            Method::PATCH => "PATCH",
            Method::POST => "POST",
            Method::DELETE => "DELETE",
            Method::PUT => "PUT",
            Method::OPTION => "OPTION",
            Method::GET => "GET",
        }
        .to_string()
    }
}

impl FromStr for ValuePair {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = s.split("=").collect();
        if v.len() != 2 {
            return Err("invalid argument should be in form of key=value".to_owned());
        }
        return Ok(ValuePair {
            key: v[0].to_owned(),
            value: v[1].to_owned(),
        });
    }
}

#[derive(FromArgs, Debug, PartialEq)]
#[argh(description = "A tool for Stress Testing")]
pub struct Bust {
    /// pass username  and password in for username:password
    #[argh(option, short = 'a')]
    pub auth: Option<String>,

    /// provide cookie for the request
    #[argh(option, short = 'C')]
    pub cookies: Vec<String>,

    /// custom http method
    #[argh(option, short = 'M')]
    pub method: Option<http::method::Method>,

    /// concurrency the number of concurrent request
    #[argh(option, short = 'c')]
    pub concurrency: u32,

    /// total number of request made
    #[argh(option, short = 'n')]
    pub total_request: u32,

    /// custom header for request
    #[argh(option, short = 'H')]
    pub headers: Vec<Header>,

    /// file path to upload the file
    #[argh(option, short = 'f')]
    pub file: Option<ValuePair>,

    /// data to be sent in request
    #[argh(option, short = 'd')]
    pub data: Option<String>,

    #[argh(positional)]
    pub url: String,
}
