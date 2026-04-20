use std::{fs::File, io::BufReader, net::SocketAddr, sync::Arc};
use bytes::Bytes;
use http::Response;
use quinn::Endpoint;
use rustls::pki_types::CertificateDer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    
    let cert_chain = rustls_pemfile::certs(&mut BufReader::new(
        File::open("/usr/local/etc/ssl/server.crt")?,
    ))
    .collect::<Result<Vec<CertificateDer>, _>>()?;

    
    let key = rustls_pemfile::private_key(&mut BufReader::new(
        File::open("/usr/local/etc/ssl/server.key")?,
    ))?
    .expect("No private key");

    
    let mut tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)?;
    
    tls_config.alpn_protocols = vec![b"h3".to_vec()];

     
    let quic_config = quinn::crypto::rustls::QuicServerConfig::try_from(tls_config)?;
    let server_config = quinn::ServerConfig::with_crypto(Arc::new(quic_config));

     
    let addr: SocketAddr = "0.0.0.0:443".parse()?;
    let endpoint = Endpoint::server(server_config, addr)?;
    println!(" HTTP/3 server listening on https://localhost:443");

    
    while let Some(conn) = endpoint.accept().await {
        let fut = handle_connection(conn);
        tokio::spawn(async move {
            if let Err(e) = fut.await {
                eprintln!(" Connection error: {e}");
            }
        });
    }
    Ok(())
}

async fn handle_connection(conn: quinn::Incoming) -> Result<(), Box<dyn std::error::Error>> {
    let connection = conn.await?;
    println!(" New connection from {}", connection.remote_address());

    
    let mut h3_conn = h3::server::Connection::new(h3_quinn::Connection::new(connection)).await?;

    loop {
        match h3_conn.accept().await {
            Ok(Some(resolver)) => {
                let (req, mut stream) = resolver.resolve_request().await?;
                println!(" Request: {:?}", req);

                let response_body: &[u8] = b"Hello, HTTP/3!\n";
                
                
                let response = Response::builder()
                    .status(200)
                    .header("content-type", "text/plain; charset=utf-8")
                    .header("content-length", response_body.len().to_string())
                    .body(())?;

                stream.send_response(response).await?;
                stream.send_data(Bytes::from(response_body)).await?;
                stream.finish().await?;
            }
            Ok(None) => break, 
            Err(e) => {
                eprintln!(" Accept error: {e}");
                break;
            }
        }
    }
    Ok(())
}
