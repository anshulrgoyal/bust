use std::net::SocketAddr;
use std::ops::Add;

use tokio::prelude::*;

#[derive(Debug, Default)]
pub struct Stats {
    pub connect: u128,
    pub handshake: u128,
    pub waiting: u128,
    pub writing: u128,
    pub compelete: u128,
    pub read: u128,
    pub length: usize,
}

impl Add for Stats {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            connect: self.connect + other.connect,
            waiting: self.waiting + other.waiting,
            writing: self.writing + other.writing,
            handshake: self.handshake + other.handshake,
            compelete: self.compelete + other.compelete,
            read: self.read + other.read,
            length: self.length.max(other.length),
        }
    }
}

pub enum Body {
    File(Vec<u8>, Vec<u8>, Vec<u8>),
    Simple(Vec<u8>),
    None,
}

/// Make https request and bench mark performace of the request. This function uses native tls for https certs.
pub async fn make_https_request(
    host: &str,
    ip: &SocketAddr,
    body: &[u8],
    extra: &Body,
) -> anyhow::Result<Stats> {
    let conn = native_tls::TlsConnector::new()?;
    let connector = tokio_tls::TlsConnector::from(conn);
    let start = std::time::Instant::now();
    let stream = tokio::net::TcpStream::connect(ip).await?;
    let connect = start.elapsed().as_millis();
    let mut con = connector.connect(host, stream).await?;
    let handshake = start.elapsed().as_millis() - connect;
    con.write(body).await?;
    match extra {
        Body::File(head, middle, end) => {
            con.write(
                format!(
                    "\r\ncontent-length: {}",
                    &head.len() + &middle.len() + &end.len()
                )
                .as_bytes(),
            )
            .await?;
            con.write(b"\r\n\r\n").await?;
            con.write(&head).await?;
            con.write(&middle).await?;
            con.write_all(&end).await?;
        }
        Body::Simple(main) => {
            con.write(format!("\r\ncontent-length: {}", &main.len()).as_bytes())
                .await?;
            con.write(b"\r\n\r\n").await?;
            con.write_all(&main).await?;
        }
        Body::None => {
            con.write(b"\r\n\r\n").await?;
            con.write_all(b" \r\n").await?;
        }
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
        length: v.len(),
    });
}

pub async fn make_http_request(
    ip: &SocketAddr,
    body: &[u8],
    extra: &Body,
) -> anyhow::Result<Stats> {
    let start = std::time::Instant::now();
    let mut stream = tokio::net::TcpStream::connect(ip).await?;
    let connect = start.elapsed().as_millis();
    stream.write(body).await?;
    match extra {
        Body::File(head, middle, end) => {
            stream
                .write(
                    format!(
                        "\r\ncontent-length: {}",
                        &head.len() + &middle.len() + &end.len()
                    )
                    .as_bytes(),
                )
                .await?;
            stream.write(b"\r\n\r\n").await?;
            stream.write(&head).await?;
            stream.write(&middle).await?;
            stream.write_all(&end).await?;
            stream.write_all(b"\r\n").await?;
        }
        Body::Simple(main) => {
            stream
                .write(format!("\r\ncontent-length: {}", &main.len()).as_bytes())
                .await?;
            stream.write(b"\r\n\r\n").await?;
            stream.write_all(&main).await?;
        }
        Body::None => {
            stream.write(b"\r\n\r\n").await?;
            stream.write_all(b" \r\n").await?;
        }
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
        length: v.len(),
    });
}
