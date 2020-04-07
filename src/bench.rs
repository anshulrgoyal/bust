extern crate test;

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_body_parsing(b: &mut Bencher) {
        b.iter(|| {
            let req: http::Request<Vec<u8>> = http::request::Builder::new()
                .method("POST")
                .header("content-type", "application/json")
                .uri("https://google.com")
                .body(vec![])
                .unwrap();
            crate::http_parser::http_string(&req, None).unwrap()
        })
    }
}
