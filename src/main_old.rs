use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rustls::ServerConfig;
use rustls_pemfile::{certs, private_key};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio_rustls::TlsAcceptor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let certs = certs(&mut BufReader::new(File::open("/usr/local/etc/ssl/server.crt")?)).collect::<Result<Vec<_>, _>>()?;
    let key = private_key(&mut BufReader::new(File::open("/usr/local/etc/ssl/server.key")?))?.expect("No key");
    
    let acceptor = TlsAcceptor::from(Arc::new(ServerConfig::builder().with_no_client_auth().with_single_cert(certs, key)?));
    let listener = TcpListener::bind("0.0.0.0:443").await?;
    println!("HTTPS server on port 443");
    
    loop {
        let (stream, addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        tokio::spawn(async move {
            if let Ok(mut tls) = acceptor.accept(stream).await {
                println!("TLS handshake with {}", addr);
                let mut buf = [0; 1024];
                if tls.read(&mut buf).await.unwrap_or(0) > 0 {
                    let _ = tls.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello, TLS!\n").await;
                }
            }
        });
    }
}
