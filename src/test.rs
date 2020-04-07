extern crate test;

#[cfg(test)]
mod tests {
    use crate::http_parser;
    use crate::multipart;
    use crate::request;
    use std::net::SocketAddr;

    use crate::args_parser::{Bust, Header, ValuePair};
    use argh::FromArgs;
    use std::str;
    use trust_dns_resolver::config::*;
    use trust_dns_resolver::TokioAsyncResolver;

    #[tokio::test]
    async fn multipart_body_test() {
        let data = multipart::get_file_as_parts("key", "files", "./test_file.txt")
            .await
            .unwrap();
        let st = str::from_utf8(&data.0).unwrap();
        let end = str::from_utf8(&data.2).unwrap();
        let file = str::from_utf8(&data.1).unwrap();
        assert_eq!(st,"--key\r\nContent-Disposition: form-data; name=\"files\"; filename=\"test_file.txt\"\r\nContent-Type: text/plain\r\n\r\n");
        assert_eq!(end, "\r\n--key--\r\n");
        assert_eq!(file, "hello there!");
    }

    #[test]
    fn http_parser_string() {
        let req: http::Request<Vec<u8>> = http::request::Builder::new()
            .method("POST")
            .header("content-type", "application/json")
            .uri("https://google.com")
            .body(vec![])
            .unwrap();
        assert_eq!(str::from_utf8(&http_parser::http_string(&req,None).unwrap()).unwrap(),"POST / HTTP/1.1\r\nHost: google.com\r\nUser-Agent: Bust/0.0.1\r\nConnection: Close\r\ncontent-type: application/json");
    }

    #[test]
    fn http_parser_string_without_headers() {
        let req: http::Request<Vec<u8>> = http::request::Builder::new()
            .method("POST")
            .uri("https://google.com")
            .body(vec![])
            .unwrap();
        assert_eq!(
            str::from_utf8(&http_parser::http_string(&req, None).unwrap()).unwrap(),
            "POST / HTTP/1.1\r\nHost: google.com\r\nUser-Agent: Bust/0.0.1\r\nConnection: Close"
        );
    }

    #[test]
    fn http_parser_string_with_get_method() {
        let req: http::Request<Vec<u8>> = http::request::Builder::new()
            .method("GET")
            .header("content-type", "application/json")
            .uri("https://google.com")
            .body(vec![])
            .unwrap();
        assert_eq!(str::from_utf8(&http_parser::http_string(&req,None).unwrap()).unwrap(),"GET / HTTP/1.1\r\nHost: google.com\r\nUser-Agent: Bust/0.0.1\r\nConnection: Close\r\ncontent-type: application/json");
    }

    #[test]
    fn http_parser_string_with_query() {
        let req: http::Request<Vec<u8>> = http::request::Builder::new()
            .method("POST")
            .header("content-type", "application/json")
            .uri("https://google.com?s=bust")
            .body(vec![])
            .unwrap();
        assert_eq!(str::from_utf8(&http_parser::http_string(&req,None).unwrap()).unwrap(),"POST /?s=bust HTTP/1.1\r\nHost: google.com\r\nUser-Agent: Bust/0.0.1\r\nConnection: Close\r\ncontent-type: application/json");
    }

    #[test]
    fn http_parser_string_with_localhost() {
        let req: http::Request<Vec<u8>> = http::request::Builder::new()
            .method("POST")
            .header("content-type", "application/json")
            .uri("https://localhost:4000")
            .body(vec![])
            .unwrap();
        assert_eq!(str::from_utf8(&http_parser::http_string(&req,None).unwrap()).unwrap(),"POST / HTTP/1.1\r\nHost: localhost\r\nUser-Agent: Bust/0.0.1\r\nConnection: Close\r\ncontent-type: application/json");
    }

    #[tokio::test]
    async fn http_request_test() {
        let req: http::Request<Vec<u8>> = http::request::Builder::new()
            .method("GET")
            .header("content-type", "application/json")
            .uri("http://www.google.com?s=bust")
            .body(vec![])
            .unwrap();
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::cloudflare(),
            ResolverOpts {
                cache_size: 0,
                use_hosts_file: false,
                ..ResolverOpts::default()
            },
        )
        .await
        .unwrap();
        let ip = match resolver
            .lookup_ip("www.google.com")
            .await
            .unwrap()
            .iter()
            .nth(0)
        {
            Some(ip) => ip,
            None => panic!("Error while making dns query"),
        };
        let socket = SocketAddr::new(ip, 80);
        request::make_http_request(
            &socket,
            &http_parser::http_string(&req, None).unwrap(),
            &request::Body::None,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn http_request_with_body_test() {
        let req: http::Request<Vec<u8>> = http::request::Builder::new()
            .method("POST")
            .header("content-type", "application/json")
            .uri("http://www.google.com?s=bust")
            .body(vec![])
            .unwrap();
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::cloudflare(),
            ResolverOpts {
                cache_size: 0,
                use_hosts_file: false,
                ..ResolverOpts::default()
            },
        )
        .await
        .unwrap();
        let ip = match resolver
            .lookup_ip("www.google.com")
            .await
            .unwrap()
            .iter()
            .nth(0)
        {
            Some(ip) => ip,
            None => panic!("Error while making dns query"),
        };
        let socket = SocketAddr::new(ip, 80);
        request::make_http_request(
            &socket,
            &http_parser::http_string(&req, None).unwrap(),
            &request::Body::Simple(b"test data".to_vec()),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn http_request_with_file_test() {
        let req: http::Request<Vec<u8>> = http::request::Builder::new()
            .method("POST")
            .header("content-type", "application/json")
            .uri("http://www.google.com?s=bust")
            .body(vec![])
            .unwrap();
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::cloudflare(),
            ResolverOpts {
                cache_size: 0,
                use_hosts_file: false,
                ..ResolverOpts::default()
            },
        )
        .await
        .unwrap();
        let ip = match resolver
            .lookup_ip("www.google.com")
            .await
            .unwrap()
            .iter()
            .nth(0)
        {
            Some(ip) => ip,
            None => panic!("Error while making dns query"),
        };
        let socket = SocketAddr::new(ip, 80);

        let data = multipart::get_file_as_parts("key", "files", "./test_file.txt")
            .await
            .unwrap();
        request::make_http_request(
            &socket,
            &http_parser::http_string(&req, None).unwrap(),
            &request::Body::File(data.0, data.1, data.2),
        )
        .await
        .unwrap();
    }

    #[test]
    fn args_parser() {
        let b = Bust::from_args(
            &["cmdname"],
            &["-n", "100", "-c", "100", "https://google.com"],
        )
        .expect("error while parsing");
        assert_eq!(
            b,
            Bust {
                auth: None,
                cookies: vec![],
                method: None,
                concurrency: 100,
                total_request: 100,
                headers: vec![],
                file: None,
                data: None,
                url: "https://google.com".to_owned()
            }
        );
    }

    #[test]
    fn args_parser_should_error_with_url() {
        let e = Bust::from_args(&["cmdname"], &["-n", "100", "-c", "100"])
            .expect_err("error while parsing");
        assert!(e.status.is_err());
    }

    #[test]
    fn args_parser_with_header() {
        let b = Bust::from_args(
            &["cmdname"],
            &[
                "-n",
                "100",
                "-c",
                "100",
                "https://google.com",
                "--headers",
                "content-type=application/json",
            ],
        )
        .expect("error while parsing");
        assert_eq!(
            b,
            Bust {
                auth: None,
                cookies: vec![],
                method: None,
                concurrency: 100,
                total_request: 100,
                headers: vec![Header {
                    key: http::header::HeaderName::from_bytes(b"content-type").unwrap(),
                    value: http::header::HeaderValue::from_str("application/json").unwrap()
                }],
                file: None,
                data: None,
                url: "https://google.com".to_owned()
            }
        );
    }

    #[test]
    fn args_parser_with_file() {
        let b = Bust::from_args(
            &["cmdname"],
            &[
                "-n",
                "100",
                "-c",
                "100",
                "https://google.com",
                "-f",
                "files=text.txt",
            ],
        )
        .expect("error while parsing");
        assert_eq!(
            b,
            Bust {
                auth: None,
                cookies: vec![],
                method: None,
                concurrency: 100,
                total_request: 100,
                headers: vec![],
                file: Some(ValuePair {
                    key: "files".to_owned(),
                    value: "text.txt".to_owned()
                }),
                data: None,
                url: "https://google.com".to_owned()
            }
        );
    }
}
