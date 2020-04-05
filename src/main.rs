use std::ops::Add;
use std::ops::Div;
use std::str::FromStr;
use std::net::SocketAddr;

use argh::FromArgs;
use tokio::prelude::*;
use prettytable::{Cell, Row, Table};
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;

mod http_parser;
mod multipart;

#[derive(Debug)]
struct ValuePair {
    key: String,
    value: String,
}

#[derive(Debug)]
struct Header {
    key: http::header::HeaderName,
    value: http::header::HeaderValue,
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
enum Method {
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

#[derive(FromArgs, Debug)]
#[argh(description = "A tool for Stress Testing")]
struct Bust {
    /// pass username  and password in for username:password
    #[argh(option, short = 'a')]
    auth: Option<String>,

    /// provide cookie for the request
    #[argh(option, short = 'C')]
    cookies: Vec<ValuePair>,

    /// custom http method
    #[argh(option, short = 'M')]
    method: Option<http::method::Method>,

    /// concurrency the number of concurrent request
    #[argh(option, short = 'c')]
    concurrency: u32,

    /// total number of request made
    #[argh(option, short = 'n')]
    total_request: u32,

    /// custom header for request
    #[argh(option, short = 'H')]
    headers: Vec<Header>,

    /// file path to upload the file
    #[argh(option, short = 'f')]
    file: Option<ValuePair>,

/// data to be sent in request
    #[argh(option,short = 'd')]
    data:Option<String>,

    #[argh(positional)]
    url: String,
}

#[derive(Debug)]
struct Stats {
    connect: u128,
    handshake: u128,
    waiting: u128,
    writing: u128,
    compelete: u128,
    read: u128,
}

#[derive(Debug)]
enum Body {
    File(Vec<u8>,Vec<u8>,Vec<u8>),
    Simple(Vec<u8>),
    None,
}

async fn make_https_request(host: &str, ip: &SocketAddr, body: &[u8],extra:&Body) -> anyhow::Result<Stats> {
    let conn = native_tls::TlsConnector::new()?;
    let connector = tokio_tls::TlsConnector::from(conn);
    let start = std::time::Instant::now();
    let stream = tokio::net::TcpStream::connect(ip).await?;
    let connect = start.elapsed().as_millis();
    let mut con = connector.connect(host, stream).await?;
    let handshake = start.elapsed().as_millis() - connect;
    con.write(body).await?;
    match extra {
        Body::File(head,middle,end) => {
            con.write(format!("\r\ncontent-length: {}", &head.len()+&middle.len()+&end.len()).as_bytes()).await?;
            con.write(b"\r\n\r\n").await?;
            con.write(&head).await?;
            con.write(&middle).await?;
            con.write_all(&end).await?;
        },
        Body::Simple(main)=>{
            con.write(format!("\r\ncontent-length: {}", &main.len()).as_bytes()).await?;
            con.write(b"\r\n\r\n").await?;
            con.write_all(&main).await?;
        },
        Body::None => {
            con.write(b"\r\n\r\n").await?;
            con.write_all(b" \r\n").await?;
        },
    }
    let writing = start.elapsed().as_millis() - handshake - connect;
    let mut first: [u8; 1] = [0];
    con.read(&mut first).await?;
    let waiting = start.elapsed().as_millis() - handshake - connect - writing;
    let mut v = Vec::new();
    con.read_to_end(&mut v).await?;
    let compelete = start.elapsed().as_millis();
    return Ok(Stats {
        connect: connect,
        handshake: handshake,
        waiting: waiting,
        writing: writing,
        read: compelete - connect - waiting - handshake - writing,
        compelete: compelete,
    });
}

async fn make_http_request(ip: &SocketAddr, body: &[u8],extra:&Body) -> anyhow::Result<Stats> {
    let start = std::time::Instant::now();
    let mut stream = tokio::net::TcpStream::connect(ip).await?;
    let connect = start.elapsed().as_millis();
    stream.write(body).await?;
    match extra {
        Body::File(head,middle,end) => {
            stream.write(format!("\r\ncontent-length: {}", &head.len()+&middle.len()+&end.len()).as_bytes()).await?;
            stream.write(b"\r\n\r\n").await?;
            stream.write(&head).await?;
            stream.write(&middle).await?;
            stream.write_all(&end).await?;
            stream.write_all(b"\r\n").await?;
        },
        Body::Simple(main)=>{
            stream.write(format!("\r\ncontent-length: {}", &main.len()).as_bytes()).await?;
            stream.write(b"\r\n\r\n").await?;
            stream.write_all(&main).await?;
        },
        Body::None => {
            stream.write(b"\r\n\r\n").await?;
            stream.write_all(b" \r\n").await?;
        },
    }
    let writing = start.elapsed().as_millis() - connect;
    let mut first: [u8; 1] = [0];
    stream.read(&mut first).await?;
    let waiting = start.elapsed().as_millis() - connect - writing;
    let mut v = Vec::new();
    stream.read_to_end(&mut v).await?;
    let compelete = start.elapsed().as_millis();
    return Ok(Stats {
        connect: connect,
        handshake: 0,
        waiting: waiting,
        writing: writing,
        read: compelete - connect - waiting - writing,
        compelete: compelete,
    });
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let t = std::time::Instant::now();
    let args: Bust = argh::from_env();
    let mut table = Table::new();
    let method = match args.method {
        Some(method) => method,
        None => http::Method::GET,
    };
    let mut req: http::Request<Vec<u8>> = http::request::Builder::new()
        .method(method)
        .uri(args.url)
        .body(vec![])?;
    let heads = req.headers_mut();
    for v in args.headers {
        heads.insert(v.key, v.value);
    }
    let file=match args.file {
        Some(expr) => {
            heads.insert("content-type",http::header::HeaderValue::from_str("multipart/form-data; boundary=----------------123456789")?);
            let data=multipart::get_file_as_parts("----------------123456789",expr.key.as_str(),expr.value.as_str()).await?;
            Body::File(data.0,data.1,data.2)
        },
        None => match args.data {
            Some(expr) => Body::Simple(expr.bytes().collect()),
            None => Body::None,
        },
    };
    let schema = match req.uri().scheme_str() {
        Some(scheme) => scheme,
        None => return Err(anyhow::anyhow!("Error the protocol")),
    };
    let host = match req.uri().host() {
        Some(host) => host,
        None => return Err(anyhow::anyhow!("Host not provided")),
    };
    let body = http_parser::http_string(&req)?;
    let lookup = std::time::Instant::now();
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::cloudflare(),
        ResolverOpts {
            cache_size: 0,
            use_hosts_file: false,
            ..ResolverOpts::default()
        },
    )
    .await?;
    let ip = match resolver.lookup_ip(host).await?.iter().nth(0) {
        Some(ip) => ip,
        None => return Err(anyhow::anyhow!("Error while making dns query")),
    };
    let lookup_time = lookup.elapsed().as_millis();
    let socket = match &req.uri().port() {
        Some(port)=>SocketAddr::new(ip, port.as_u16()),
        None=>match schema {
        "https" => SocketAddr::new(ip, 443),
        "http" => SocketAddr::new(ip, 80),
        _ => return Err(anyhow::anyhow!("Error while creating ip")),
    }};
    let mut ac = Stats {
        connect: 0,
        handshake: 0,
        waiting: 0,
        writing: 0,
        read: 0,
        compelete: 0,
    };
    let mut max = Stats {
        connect: 0,
        handshake: 0,
        waiting: 0,
        writing: 0,
        read: 0,
        compelete: 0,
    };
    let mut min = Stats {
        connect: u128::max_value(),
        handshake: u128::max_value(),
        waiting: u128::max_value(),
        writing: u128::max_value(),
        read: u128::max_value(),
        compelete: u128::max_value(),
    };
    let cycles = args.total_request / args.concurrency;
    // let c=Vec::with_capacity((cycles+1) as usize);
    let mut compeleted = vec![];
    for _ in 0..cycles {
        match schema {
            "http" => {
                let mut v = Vec::with_capacity(args.concurrency as usize);
                for _ in 1..args.concurrency {
                    v.push(make_http_request(&socket, body.as_slice(),&file))
                }
                let s = futures::future::try_join_all(v).await?;
                s.iter().for_each(|c| {
                    compeleted.push(c.compelete);
                    min.connect = std::cmp::min(min.connect, c.connect);
                    min.handshake = std::cmp::min(min.handshake, c.handshake);
                    min.waiting = std::cmp::min(min.waiting, c.waiting);
                    min.writing = std::cmp::min(min.writing, c.writing);
                    min.read = std::cmp::min(min.read, c.read);
                    min.compelete = std::cmp::min(min.compelete, c.compelete);

                    max.connect = std::cmp::max(max.connect, c.connect);
                    max.handshake = std::cmp::max(max.handshake, c.handshake);
                    max.waiting = std::cmp::max(max.waiting, c.waiting);
                    max.writing = std::cmp::max(max.writing, c.writing);
                    max.read = std::cmp::max(max.read, c.read);
                    max.compelete = std::cmp::max(max.compelete, c.compelete);

                    ac.connect = ac.connect.add(c.connect);
                    ac.handshake = ac.handshake.add(c.handshake);
                    ac.waiting = ac.waiting.add(c.waiting);
                    ac.writing = ac.writing.add(c.writing);
                    ac.read = ac.read.add(c.read);
                    ac.compelete = ac.compelete.add(c.compelete);
                });
            }
            "https" => {
                let mut v = Vec::with_capacity(args.concurrency as usize);
                for _ in 1..args.concurrency {
                    v.push(make_https_request(host, &socket, body.as_slice(),&file));
                }
                let s = futures::future::try_join_all(v).await?;
                s.iter().for_each(|c| {
                    compeleted.push(c.compelete);
                    min.connect = std::cmp::min(min.connect, c.connect);
                    min.handshake = std::cmp::min(min.handshake, c.handshake);
                    min.waiting = std::cmp::min(min.waiting, c.waiting);
                    min.writing = std::cmp::min(min.writing, c.writing);
                    min.read = std::cmp::min(min.read, c.read);
                    min.compelete = std::cmp::min(min.compelete, c.compelete);

                    max.connect = std::cmp::max(max.connect, c.connect);
                    max.handshake = std::cmp::max(max.handshake, c.handshake);
                    max.waiting = std::cmp::max(max.waiting, c.waiting);
                    max.writing = std::cmp::max(max.writing, c.writing);
                    max.read = std::cmp::max(max.read, c.read);
                    max.compelete = std::cmp::max(max.compelete, c.compelete);

                    ac.connect = ac.connect.add(c.connect);
                    ac.handshake = ac.handshake.add(c.handshake);
                    ac.waiting = ac.waiting.add(c.waiting);
                    ac.writing = ac.writing.add(c.writing);
                    ac.read = ac.read.add(c.read);
                    ac.compelete = ac.compelete.add(c.compelete);
                });
            }
            _ => return Err(anyhow::anyhow!("Error with protocol")),
        }
    }
    println!(
        " Schema: {}\n Hostname: {}\n Path: {}\n",
        schema,
        host,
        &req.uri().path()
    );
    compeleted.sort();
    let total = compeleted.len();
    ac.connect = ac.connect.div(args.total_request as u128);
    ac.handshake = ac.handshake.div(args.total_request as u128);
    ac.waiting = ac.waiting.div(args.total_request as u128);
    ac.read = ac.read.div(args.total_request as u128);
    ac.compelete = ac.compelete.div(args.total_request as u128);
    ac.writing = ac.writing.div(args.total_request as u128);
    println!(
        "\nTime taken for bench Marking : {}s",
        t.elapsed().as_secs()
    );
    table.add_row(Row::new(vec![
        Cell::new("Task"),
        Cell::new("Min Time(milliseconds)"),
        Cell::new("Average Time(milliseconds)"),
        Cell::new("Max Time(milliseconds)"),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Dns Query"),
        Cell::new(lookup_time.to_string().as_str()),
        Cell::new(lookup_time.to_string().as_str()),
        Cell::new(lookup_time.to_string().as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Connection Time"),
        Cell::new(min.connect.to_string().as_str()),
        Cell::new(ac.connect.to_string().as_str()),
        Cell::new(max.connect.to_string().as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Tls Handshake Time"),
        Cell::new(min.handshake.to_string().as_str()),
        Cell::new(ac.handshake.to_string().as_str()),
        Cell::new(max.handshake.to_string().as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Waiting For Resposne"),
        Cell::new(min.waiting.to_string().as_str()),
        Cell::new(ac.waiting.to_string().as_str()),
        Cell::new(max.waiting.to_string().as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Writing the Request"),
        Cell::new(min.writing.to_string().as_str()),
        Cell::new(ac.writing.to_string().as_str()),
        Cell::new(max.writing.to_string().as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("Compelete"),
        Cell::new(min.compelete.to_string().as_str()),
        Cell::new(ac.compelete.to_string().as_str()),
        Cell::new(max.compelete.to_string().as_str()),
    ]));
    table.printstd();
    println!("\nApprox time Required to compelete % of request");

    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Percentage of Request"),
        Cell::new("Time(milliseconds)"),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("50%"),
        Cell::new(compeleted[total / 2].to_string().as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("75%"),
        Cell::new(compeleted[total * 3 / 4].to_string().as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("90%"),
        Cell::new(compeleted[total * 9 / 10].to_string().as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("95%"),
        Cell::new(compeleted[total * 95 / 100].to_string().as_str()),
    ]));
    table.add_row(Row::new(vec![
        Cell::new("100%"),
        Cell::new(compeleted[total - 1].to_string().as_str()),
    ]));
    table.printstd();
    Ok(())
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}