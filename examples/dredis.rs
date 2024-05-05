// dummy redis server

use std::{io::ErrorKind, net::SocketAddr};

use anyhow::Result;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tracing::{debug, info, warn};

const BUF_SIZE: usize = 4096;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "127.0.0.1:6379";
    let listener = TcpListener::bind(addr).await?;
    info!("listening on {addr}");

    loop {
        let (stream, addr) = listener.accept().await?;
        debug!("accepted connection from {addr}");
        tokio::spawn(async move {
            if let Err(e) = process_connection(stream, addr).await {
                warn!("connection error: {e}");
            }
        });
    }
}

async fn process_connection(mut stream: TcpStream, addr: SocketAddr) -> Result<()> {
    loop {
        stream.readable().await?;
        let mut buf = Vec::with_capacity(BUF_SIZE);
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                // process request
                debug!("received {n} bytes from {addr}");
                let line = String::from_utf8_lossy(&buf[..n]);
                debug!("received: {line}");
                stream.write_all(b"+OK\r\n").await?;
            }
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                warn!("read error: {e}");
                break;
            }
        }
    }
    warn!("connection closed: {addr}");
    Ok(())
}
