use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rustls::ServerConfig;
use rustls::pki_types::CertificateDer;
use rustls_pemfile::{certs, private_key};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tokio_rustls::TlsAcceptor;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Load the certificate chain from the PEM file
    let cert_file = File::open("/usr/local/etc/ssl/server.crt")?;
    let mut cert_reader = BufReader::new(cert_file);
    
    // Collect certificates into a vector
    let cert_chain: Vec<CertificateDer<'static>> = certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()?;

    // 2. Load the private key from the PEM file
    let key_file = File::open("/usr/local/etc/ssl/server.key")?;
    let mut key_reader = BufReader::new(key_file);
    
    // private_key() returns a PrivateKeyDer directly.
    // No conversion (like .to_vec()) is needed here.
    let raw_key = private_key(&mut key_reader)?
        .expect("The private key was not found in the file");

    // 3. Create TLS configuration
    // Use 'raw_key' directly as it matches the expected type.
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, raw_key)?;

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind("0.0.0.0:443").await?;
    println!("HTTPS server started on port 443");

    // 4. Server loop
    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            // Perform TLS handshake
            match acceptor.accept(stream).await {
                Ok(mut tls_stream) => {
                    println!("TLS handshake successful with {}", peer_addr);
                    
                    // Simple logic to handle the connection
                    let mut buffer = [0; 1024];
                    loop {
                        match tls_stream.read(&mut buffer).await {
                            Ok(0) => break, // Connection closed by client
                            Ok(_) => {
                                // Send a simple HTTP response
                                let response = "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\nHello, TLS!\n";
                                if tls_stream.write_all(response.as_bytes()).await.is_err() {
                                    eprintln!("Failed to send response to {}", peer_addr);
                                }
                                break;
                            }
                            Err(e) => {
                                eprintln!("Read error from {}: {}", peer_addr, e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => eprintln!("TLS handshake failed with {}: {}", peer_addr, e),
            }
        });
    }
}
