use quinn::{ServerConfig, rustls::pki_types::PrivatePkcs8KeyDer};

use voxel_net::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:8080".parse()?;
    let config = configure_server()?;

    let server = Server::bind(addr, config).await?;
    server.run().await?;

    Ok(())
}

pub fn configure_server() -> anyhow::Result<ServerConfig> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let key = PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der());
    Ok(ServerConfig::with_single_cert(
        vec![cert.cert.into()],
        key.into(),
    )?)
}
