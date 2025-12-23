use server::{Server, configure_server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "127.0.0.1:8080".parse()?;
    let config = configure_server()?;

    let mut server = Server::bind(addr, config).await?;
    server.run().await
}
