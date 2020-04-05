#[cfg(test)]
mod tests {
    use crate::http_parser;
    use crate::multipart;
    use std::str;

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
        assert_eq!(str::from_utf8(&http_parser::http_string(&req).unwrap()).unwrap(),"POST / HTTP/1.1\r\nHost: google.com\r\nUser-Agent: Bust/0.0.1\r\nConnection:Close\r\ncontent-type: application/json");
    }

    #[test]
    fn http_parser_string_without_headers() {
        let req: http::Request<Vec<u8>> = http::request::Builder::new()
            .method("POST")
            .uri("https://google.com")
            .body(vec![])
            .unwrap();
        assert_eq!(
            str::from_utf8(&http_parser::http_string(&req).unwrap()).unwrap(),
            "POST / HTTP/1.1\r\nHost: google.com\r\nUser-Agent: Bust/0.0.1\r\nConnection:Close"
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
        assert_eq!(str::from_utf8(&http_parser::http_string(&req).unwrap()).unwrap(),"GET / HTTP/1.1\r\nHost: google.com\r\nUser-Agent: Bust/0.0.1\r\nConnection:Close\r\ncontent-type: application/json");
    }

    #[test]
    fn http_parser_string_with_query() {
        let req: http::Request<Vec<u8>> = http::request::Builder::new()
            .method("POST")
            .header("content-type", "application/json")
            .uri("https://google.com?s=bust")
            .body(vec![])
            .unwrap();
        assert_eq!(str::from_utf8(&http_parser::http_string(&req).unwrap()).unwrap(),"POST /?s=bust HTTP/1.1\r\nHost: google.com\r\nUser-Agent: Bust/0.0.1\r\nConnection:Close\r\ncontent-type: application/json");
    }

    #[test]
    fn http_parser_string_with_localhost() {
        let req: http::Request<Vec<u8>> = http::request::Builder::new()
            .method("POST")
            .header("content-type", "application/json")
            .uri("https://localhost:4000")
            .body(vec![])
            .unwrap();
        assert_eq!(str::from_utf8(&http_parser::http_string(&req).unwrap()).unwrap(),"POST / HTTP/1.1\r\nHost: localhost\r\nUser-Agent: Bust/0.0.1\r\nConnection:Close\r\ncontent-type: application/json");
    }
}
