#![feature(test)]

use std::net::SocketAddr;
use std::ops::Div;

use spinners::{Spinner, Spinners};
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;

use crate::args_parser::Bust;
use crate::request::{make_http_request, make_https_request, Body, Stats};

mod args_parser;
mod bench;
mod calculate;
mod http_parser;
mod multipart;
mod request;
mod tables;
mod test;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sp = Spinner::new(Spinners::Dots12, "Running your benchmark".into());
    let t = std::time::Instant::now();
    let args: Bust = argh::from_env();
    let method = match args.method {
        Some(method) => method,
        None => http::Method::GET,
    };
    let mut req: http::Request<Vec<u8>> = http::request::Builder::new()
        .method(method)
        .uri(args.url)
        .body(vec![])?;
    let heads = req.headers_mut();
    for c in args.cookies {
        heads.insert(
            http::header::SET_COOKIE,
            http::header::HeaderValue::from_str(c.as_str())?,
        );
    }
    for v in args.headers {
        heads.insert(v.key, v.value);
    }
    let file = match args.file {
        Some(expr) => {
            heads.insert(
                "content-type",
                http::header::HeaderValue::from_str(
                    "multipart/form-data; boundary=----------------123456789",
                )?,
            );
            let data = multipart::get_file_as_parts(
                "----------------123456789",
                expr.key.as_str(),
                expr.value.as_str(),
            )
            .await?;
            Body::File(data.0, data.1, data.2)
        }
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
    let body = http_parser::http_string(&req, args.auth)?;
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
    let port = match &req.uri().port() {
        Some(port) => port.as_u16(),
        None => match schema {
            "https" => 443,
            "http" => 80,
            _ => return Err(anyhow::anyhow!("Error while creating ip")),
        },
    };
    let socket = SocketAddr::new(ip, port);
    let mut ac = Stats::default();
    let mut max = Stats::default();
    let mut min = Stats {
        connect: u128::max_value(),
        handshake: u128::max_value(),
        waiting: u128::max_value(),
        writing: u128::max_value(),
        read: u128::max_value(),
        compelete: u128::max_value(),
        length: usize::max_value(),
    };
    let mut len: usize = 0;
    let mut fail = 0;
    let mut success = 0;
    let cycles = args.total_request / args.concurrency;
    let mut compeleted = vec![];
    for _ in 0..cycles {
        match schema {
            "http" => {
                let mut v = Vec::with_capacity(args.concurrency as usize);
                for _ in 0..args.concurrency {
                    v.push(make_http_request(&socket, body.as_slice(), &file))
                }
                let s = futures::future::join_all(v).await;
                s.iter().for_each(|ele| match ele {
                    Ok(c) => {
                        success = success + 1;
                        len = c.length;
                        compeleted.push(c.compelete);
                        calculate::calculate_stats(&mut min, &mut max, &c, &mut ac)
                    }
                    Err(_) => {
                        fail = fail + 1;
                    }
                });
            }
            "https" => {
                let mut v = Vec::with_capacity(args.concurrency as usize);
                for _ in 0..args.concurrency {
                    v.push(make_https_request(host, &socket, body.as_slice(), &file));
                }
                let s = futures::future::join_all(v).await;
                s.iter().for_each(|ele| match ele {
                    Ok(c) => {
                        success = success + 1;
                        len = c.length;
                        compeleted.push(c.compelete);
                        calculate::calculate_stats(&mut min, &mut max, &c, &mut ac)
                    }
                    Err(_) => {
                        fail = fail + 1;
                    }
                });
            }
            _ => return Err(anyhow::anyhow!("Error with protocol")),
        }
    }

    sp.stop();
    print!("\r");
    if compeleted.len() < 3 {
        println!("{}", "Benchmark failed due to failure of request");
        return Ok(());
    }
    println!(
        " Schema          : {}\n Hostname        : {}\n Path            : {}\n Port            : {}\n Resposne-Length : {}\n",
        schema,
        host,
        &req.uri().path(),
        port,
        len
    );
    println!(" Number of Total Request     : {}",fail+success);
    println!(" Number of Sucessfull Request: {}",success);
    println!(" Number of Failed Request    : {}",fail);
    compeleted.sort();
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

    tables::create_task_table(&min, &max, &ac, lookup_time);
    println!("\nApprox time Required to compelete % of request");
    tables::create_percent_table(&compeleted);
    Ok(())
}
