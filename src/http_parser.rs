use http::Request;

pub fn http_string<T>(req: &Request<T>, auth: Option<String>) -> anyhow::Result<Vec<u8>> {
    let mut headers = String::from("");
    let host = match req.uri().host() {
        Some(v) => match auth {
            Some(expr) => format!("{}@{}", expr, v),
            None => v.to_owned(),
        },
        None => {
            return Err(anyhow::anyhow!("host name not found"));
        }
    };
    // let body_length=req.body().len();
    let path = match req.uri().path_and_query() {
        Some(path) => path,
        None => {
            return Err(anyhow::anyhow!("no path in url found"));
        }
    };
    headers.push_str(
        format!(
            "Host: {}\r\nUser-Agent: Bust/0.0.1\r\nConnection: Close",
            host
        )
        .as_str(),
    );
    for (key, val) in req.headers() {
        headers.push_str(format!("\r\n{}: {}", key, val.to_str().unwrap()).as_str())
    }
    let stup = format!("{} {} HTTP/1.1\r\n{}", req.method(), path, headers);
    Ok(stup.into_bytes().to_vec())
}
